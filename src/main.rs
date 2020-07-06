//! main is the gtk frontend of the code. Perhaps it should be moved into its own interface module?
mod res;

use ofmice::*;
use res::*;
use installation::Installation;
use progress::Progress;
use download::download;

use std::sync::Arc;
use arc_swap::ArcSwap;
use std::rc::Rc;
use std::cell::Cell;
use std::ops::Deref;
use std::path::Path;

use lazy_static::lazy_static;
use gtk::prelude::*;
use gio::prelude::*;
use gio::ApplicationFlags;
use glib::Continue;
use gtk::*;
use gdk_pixbuf::Pixbuf;


#[derive(Clone)]
struct ErrorDisplayer{
  pub window: Window,
}
impl ErrorDisplayer{
    fn display_error<S: AsRef<str>>(&self, text: S) {
        let md = MessageDialog::new(
            Some(&self.window),
            DialogFlags::MODAL|DialogFlags::DESTROY_WITH_PARENT,
            MessageType::Error, // TODO: maybe use Warning for some errors
            ButtonsType::Ok,
            text.as_ref()
        );
        md.run();
        // md.destroy();
    }

    fn display_wrangler_err(&self, e: WranglerError){
        self.display_error( match e {
                    WranglerError::SteamNotRunning => "Steam is not running, unable to set Source SDK 2013 or Team Fortress 2 paths",
                    WranglerError::TF2NotInstalled => "Team Fortress 2 is not installed",
                    WranglerError::SSDKNotInstalled => "Source SDK Base 2013 Multiplayer is not installed",
                })
    }
}

lazy_static!{
    /// the entire installation struct is serialized to ~/.of/INST.json
    /// ArcSwap'd so that the worker and main threads can share.
    static ref INST: ArcSwap<Installation> = ArcSwap::from(Arc::new(Installation::try_load().unwrap_or_default()));
}

#[derive(Clone, Copy)]
enum NeedsUpdateErr {
    Wait,
    Offline,
}

struct Model {
    // any other fields you add here won't be saved between runs of the launcher
    // hold some news-related info here?
    // a stream for communicating with the worker thread
    button_pixbufs: [Pixbuf; 5],
    // Errorbox
    //TODO: Maybe migrate everything off of the errordisplayer type onto Model?
    ed: ErrorDisplayer,
    /// What is the main button displaying?
    config_needs_attention: Cell<bool>,
    game_needs_update: Cell<Result<bool, NeedsUpdateErr>>,
    active: Rc<Cell<bool>>,
    play_button_image: Image,
    /// The switchy tabby thing
    home_screen: Notebook,
    /// config menu source sdk 2013 path box
    ssdk_path_box: Entry,
    /// config menu team fortress 2 path box
    tf2_path_box: Entry,
    progress_screen: Box,
    stack: Stack,
    progress_bar: ProgressBar,
    window: Window,
}

impl Model {
    fn config_updated(&self, inst: &Installation) {
        if inst.is_tf2_path_good() {
            self.tf2_path_box.set_widget_name("valid-path");
        } else {
            self.tf2_path_box.set_widget_name("invalid-path");
        }

        if inst.is_ssdk_path_good() {
            self.ssdk_path_box.set_widget_name("valid-path");
        } else {
            self.ssdk_path_box.set_widget_name("invalid-path");
        }

        self.config_needs_attention.set(!inst.can_launch());
        self.set_main_button_graphic();
    }

    fn save_install(&self){
        INST.load().save_changes().expect("TODO: FIXME: THIS SHOULD DISPLAY AN ERR TO USER");
    }

    /// play game, no update
    fn action_play(&self){
        //TODO: change main button from UPDATE to PLAY
        INST.load().launch();
        self.window.close();
    }
    /// update game, no play
    /// (change button to play once done, if no err)
    fn action_update(self: &Rc<Self>){
        /*

        // new fields for Model:
            stack,
            home_screen,
            progress_screen,
            progress_bar,
        
        // active: Rc::new(Cell::new(false)),
*/
        let model = self.as_ref();
        if model.active.get() {
            return;
        }

        model.active.set(true);

        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let (err_tx, err_rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        {
            // thread::spawn(move || {
            let progress = Progress::new(tx);
            tokio::spawn((move || async move{
                let mut inst = INST.load().deref().deref().clone();

                download(&mut inst, progress).await.map_err(|e| err_tx.send(e).ok()).ok();
                INST.store(Arc::new(inst));
            })());
        }

        {
            let model = self.clone();
            err_rx.attach(None, move |e: download::DownloadError| {
                eprintln!("DownloadError: {:?}", e);
                model.ed.display_error("TODO: actually handle DownloadErrors properly");
                Continue(false)
            });
        }

        model.stack.set_visible_child(&model.progress_screen);

        let active = self.active.clone();
        let model = self.clone();
        rx.attach(None, move |value| match value {
            Some((value, message)) => {
                model.progress_bar.set_fraction(value);
                model.progress_bar.set_text(Some(&message));
                Continue(true)
            }
            None => {
                let model = model.clone();
                model.stack.set_visible_child(&model.home_screen);
                active.set(false);
                Continue(false)
            }
        });
    }
    /// go to config tab
    /// because a configured path is invalid.
    fn action_config(&self){
        // XXX: HACKHACK: Hardcoding tab 1 is config
        self.home_screen.set_current_page(Some(1)); 
    }
    /// Decides what action should happen
    /// when the main button is pressed
    fn action_main_button(self: &Rc<Self>){
        if self.config_needs_attention.get() {
            self.action_config()
        }else { match self.game_needs_update.get() {
            Ok(true) => self.action_update(), // UPDATE
            Ok(false) => self.action_play(), // PLAY
            Err(NeedsUpdateErr::Offline) => (), // TODO: Should OFFLINE run the game anyways?
            Err(NeedsUpdateErr::Wait) => (),
        }};
    }
    fn set_main_button_graphic(&self){
        let state = if self.config_needs_attention.get() {
            3 // CONFIG
        }else { match self.game_needs_update.get() {
            Ok(true) => 0, // UPDATE
            Ok(false) => 1, // PLAY
            Err(NeedsUpdateErr::Offline) => 2,
            Err(NeedsUpdateErr::Wait) => 4,
        }};
        self.play_button_image.set_from_pixbuf(Some(&self.button_pixbufs[state]));
    }

    fn build_ui(application: &gtk::Application) {
        // Build our UI from ze XML
        let builder = Builder::from_string(load_glade().as_ref());

        // Apply the CSS to ze XML
        let provider = CssProvider::new();
        provider
            .load_from_data(load_css().as_ref())
            .expect("Failed to load CSS");
        StyleContext::add_provider_for_screen(
            &gdk::Screen::get_default().expect("Error initializing gtk css provider."),
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // window needs application
        let window: Window = builder.get_object("window").unwrap();
        window.set_application(Some(application));

        // Load play button pixbufs
        let button_pixbufs = load_button_pixbufs();

        // Check what the play button will do
        let play_button_image: Image = builder.get_object("play_button_image").unwrap();

        let home_screen: Notebook = builder.get_object("home_screen").unwrap();

        let ssdk_path_box: Entry = builder.get_object("ssdk_path").unwrap();
        let tf2_path_box: Entry = builder.get_object("tf2_path").unwrap();

        let progress_screen: Box = builder.get_object("progress_screen").unwrap();
        let stack: Stack = builder.get_object("stack").unwrap();
        let progress_bar: ProgressBar = builder.get_object("progress_bar").unwrap();


        // For the UI model
        //TODO: update main button state as config is edited
        let model = Rc::new(Model{
            button_pixbufs,
            ed: ErrorDisplayer{window: window.clone()},
            config_needs_attention: Cell::new(!INST.load().can_launch()),
            game_needs_update: Cell::new(Err(NeedsUpdateErr::Wait)),
            play_button_image: play_button_image.clone(),
            home_screen: home_screen.clone(),
            ssdk_path_box: ssdk_path_box.clone(),
            tf2_path_box: tf2_path_box.clone(),
            active: Rc::new(Cell::new(false)),
            progress_bar,
            stack,
            progress_screen,
            window: window.clone(),
        });

        // steam_wrangler if needed
        let mut inst = INST.load().deref().deref().clone();
        model.config_updated(&inst);
        inst.init_ssdk().unwrap_or_else(|e| model.ed.display_wrangler_err(e));
        INST.store(Arc::new(inst));

        // initialize state
        {
            let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

            //TODO: occasionally check for updates if idle?
            tokio::spawn( ( move || { async move {
                //TODO: maybe display this error if we screw up
                tx.send(download::is_update_available(&INST.load()).await.map_err(|_| NeedsUpdateErr::Offline) ).unwrap();
            }})());

            let model = model.clone();
            rx.attach(None, move |value| { model.game_needs_update.set(value); model.set_main_button_graphic(); Continue(false)});
        }

        // transparent hooks
        set_visual(&window, None);
        window.connect_draw(draw);
        window.connect_screen_changed(set_visual);

        // Set the background image
        let background: Image = builder.get_object("background").unwrap();
        background.set_from_pixbuf(Some(&load_bg()));

        // Set the tab's icons
        let play_tabicon: Image = builder.get_object("play_tab").unwrap();
        play_tabicon.set_from_pixbuf(Some(&load_play_icon()));
        let config_tabicon: Image = builder.get_object("config_tab").unwrap();
        config_tabicon.set_from_pixbuf(Some(&load_config_icon()));

        // close button icon
        let close_image: Image = builder.get_object("close_image").unwrap();
        close_image.set_from_pixbuf(Some(&load_close_icon()));
        // and action
        let close_eventbox: EventBox = builder.get_object("close_eventbox").unwrap();
        {
            let window = window.clone();
            let model = model.clone();
            close_eventbox.connect_button_press_event(move |_,_|{
                model.save_install();
                window.close();
                Inhibit(false)
            });
        }

        // Set the main logo
        let logo: Image = builder.get_object("logo").unwrap();
        logo.set_from_pixbuf(Some(&load_logo()));

        // Set the version
        let version: Label = builder.get_object("version").unwrap();
        version.set_label(&format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")));
        {
            let credits: EventBox = builder.get_object("credits_event").unwrap();
            let window = window.clone();
            credits.connect_button_press_event(move |_, _|{
                let credits = format!("Launcher written by:\n{}\nTODO: List crates used and stuff, especially the MIT APACHE and BSD licensed ones.\nMaybe have a series of credits boxes", env!("CARGO_PKG_AUTHORS").replace(':', "\n"));
                //TODO: use the about dialog
                let md = MessageDialog::new(
                    Some(&window),
                    DialogFlags::MODAL|DialogFlags::DESTROY_WITH_PARENT,
                    MessageType::Info,
                    ButtonsType::Ok,
                    &credits
                );
                md.run();
                // md.destroy();
                Inhibit(true)
            });
        }

        // Save the config when the config tab is navigated away from
        {
            let model = model.clone();
            home_screen.connect_switch_page(move |home_screen, _page, _page_num| {
                if home_screen.get_current_page()==Some(1) {
                    model.save_install();
                }
            });
        }
        // or when the close button is pressed, but that's handled elsewhere.

        {
            let model = model.clone();
            ssdk_path_box.set_text(&INST.load().ssdk_path.to_string_lossy());
            ssdk_path_box.connect_focus_out_event(move |widget, _event| {
                let t = widget.get_text();
                let p = Path::new(t.as_str());

                let mut inst = INST.load().deref().deref().clone();
                inst.ssdk_path = p.to_path_buf();
                model.config_updated(&inst);
                INST.store(Arc::new(inst));
                // println!("Out of focus");
                Inhibit(false)
            });
        }

        {
            let model = model.clone();
            tf2_path_box.set_text(&INST.load().tf2_path.to_string_lossy());
            tf2_path_box.connect_focus_out_event(move |widget, _event| {
                let t = widget.get_text();
                let p = Path::new(t.as_str());

                let mut inst = INST.load().deref().deref().clone();
                inst.tf2_path = p.to_path_buf();
                model.config_updated(&inst);
                INST.store(Arc::new(inst));
                // println!("Out of focus");
                Inhibit(false)
            });
        }

        {
            let launch_opts: Entry = builder.get_object("launch_opts").unwrap();
            launch_opts.set_text(&INST.load().launch_options);
            launch_opts.connect_focus_out_event(move |widget, _event| {
                let t = widget.get_text().as_str().to_owned();
                let mut inst = INST.load().deref().deref().clone();
                inst.launch_options = t;
                INST.store(Arc::new(inst));
                Inhibit(false)
            });
        }

        Self::connect_progress(&builder, model.clone());

        // space bar for play
        window.connect_key_press_event( move |_, e| {
            if e.get_keyval() == gdk::keys::constants::space
            && model.home_screen.get_current_page()==Some(0) {
                model.action_main_button();
            }
            Inhibit(false)
        });

        window.show_all();
    }

    fn connect_progress(builder: &Builder, model: Rc<Model>){
        // Play button does things
        let play_button: EventBox = builder.get_object("play_button").unwrap();

        {
            let model = model.clone();
            play_button.connect_button_press_event( move |_,_| {
                model.action_main_button();
                Inhibit(false)
            });
        }
    }
}


fn set_visual(window: &Window, _screen: Option<&gdk::Screen>) {
    if let Some(screen) = window.get_screen() {
        if let Some(ref visual) = screen.get_rgba_visual() {
            window.set_visual(Some(visual)); // crucial for transparency
        }
    }
}

fn draw(_window: &Window, ctx: &cairo::Context) -> Inhibit {
    // crucial for transparency
    ctx.set_source_rgba(0.0, 0.0, 0.0, 0.0);
    ctx.set_operator(cairo::Operator::Screen);
    ctx.paint();
    Inhibit(false)
}
#[tokio::main]
async fn main() {
    // pretty_env_logger::init();

    // let old = installation::Installation::try_load().unwrap_or_default();
    // let mut new = old.clone();
    // download::download(&mut new).await.unwrap();
    // println!("update available: {:?}", download::is_update_available(&old).await);
    // new.save_changes().unwrap();

    let uiapp = gtk::Application::new(Some("fun.openfortress.ofmice"),
                    ApplicationFlags::FLAGS_NONE)
                .expect("Application::new failed");

    uiapp.connect_activate(Model::build_ui);

    uiapp.run(&std::env::args().collect::<Vec<_>>());
}
