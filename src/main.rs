//! main is the vgtk frontend of the code. Perhaps it should be moved into its own interface module?
mod res;

use ofmice::*;
use res::*;

use installation::Installation;
use std::sync::{Arc, RwLock};
use std::rc::Rc;
use std::cell::Cell;
use std::time::Duration;
use std::thread;
use std::path::Path;

use gtk::prelude::*;
use gio::prelude::*;
use gio::ApplicationFlags;
use glib::clone;
use gtk::*;

use crate::platform::ssdk_exe;

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
        md.destroy();
    }

    fn display_wrangler_err(&self, e: WranglerError){
        self.display_error( match e {
                    WranglerError::SteamNotRunning => "Steam is not running, unable to set Source SDK 2013 or Team Fortress 2 paths",
                    WranglerError::TF2NotInstalled => "Team Fortress 2 is not installed",
                    WranglerError::SSDKNotInstalled => "Source SDK Base 2013 Multiplayer is not installed",
                })
    }
}

struct Model {
    /// the entire installation struct is serialized to ~/.of/installation.json
    /// RWLocked so that the worker and main threads can share.
    pub installation: Arc<RwLock<Installation>>,
    // any other fields you add here won't be saved between runs of the launcher
    // hold some news-related info here?
    // a stream for communicating with the worker thread
}

impl Model {
    fn new() -> Self {
        let installation = Installation::try_load().unwrap_or_default();
        Model{
            installation: Arc::new(RwLock::new(
                installation
            ))
        }
    }
}

fn build_ui(application: &gtk::Application) {
    // Build our UI from ze XML
    let builder = Builder::new_from_string(load_glade().as_ref());

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

    // Errorbox setup goes here

    let model = Rc::new(Model::new());

    // window needs application
    let window: Window = builder.get_object("window").unwrap();
    window.set_application(Some(application));

    // transparent hooks
    set_visual(&window, None);
    window.connect_draw(draw);
    window.connect_screen_changed(set_visual);

    // Set the background image
    let background: Image = builder.get_object("background").unwrap();
    background.set_from_pixbuf(Some(&load_bg()));

    // Set the tab's icons
    let play_tabicon: Image = builder.get_object("play-tab").unwrap();
    play_tabicon.set_from_pixbuf(Some(&load_play_icon()));
    let config_tabicon: Image = builder.get_object("config-tab").unwrap();
    config_tabicon.set_from_pixbuf(Some(&load_config_icon()));

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
            let md = MessageDialog::new(
                Some(&window),
                DialogFlags::MODAL|DialogFlags::DESTROY_WITH_PARENT,
                MessageType::Info,
                ButtonsType::Ok,
                &credits
            );
            md.run();
            md.destroy();
            Inhibit(true)
        });
    }

    // Save the config when the config tab is navigated away from
    let home_screen: Notebook = builder.get_object("home_screen").unwrap();
    {
        let model = model.clone();
        home_screen.connect_switch_page(move |home_screen, _page, _page_num| {
            if home_screen.get_current_page()==Some(1) {
                model.installation.read().unwrap().save_changes().expect("TODO: FIXME: THIS SHOULD DISPLAY AN ERR TO USER");
            }
        });
    }

    let ssdk_path: Entry = builder.get_object("ssdk_path").unwrap();
    {
        let model = model.clone();
        ssdk_path.connect_focus_out_event(move |_widget, _event| {
            let inst = &mut model.installation.write().unwrap();
            let t = _widget.get_text().unwrap();
            let p = Path::new(t.as_str());

            if p.join(ssdk_exe()).exists() {
                _widget.set_widget_name("valid-path");
                inst.ssdk_path = p.to_path_buf();
            } else {
                _widget.set_widget_name("invalid-path");
            }
            // println!("Out of focus");
            Inhibit(false)
        });
    }

    connect_progress(&builder, &model);

    window.show_all();

    let ed = ErrorDisplayer {window};

    model.installation.write().unwrap().init_ssdk().unwrap_or_else(|e| ed.display_wrangler_err(e));
}

fn connect_progress(builder: &Builder, model: &Rc<Model>){
    // Play button does things
    let play_button: Button = builder.get_object("play-button").unwrap();
    
    let home_screen: Notebook = builder.get_object("home_screen").unwrap();
    let progress_screen: Box = builder.get_object("progress_screen").unwrap();
    let stack: Stack = builder.get_object("stack").unwrap();
    let progress_bar: ProgressBar = builder.get_object("progress_bar").unwrap();
    
    let widgets = Rc::new((
        stack,
        home_screen,
        progress_screen,
        progress_bar,
    ));

    let active = Rc::new(Cell::new(false));
    play_button.connect_clicked( clone!( @weak model => move |_| {
        if active.get() {
            return;
        }

        active.set(true);

        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        thread::spawn(move || {
            for v in 1..=25 {
                let _ = tx.send(Some(v));
                thread::sleep(Duration::from_millis(200));
            }
            let _ = tx.send(None);
        });

        widgets.0
            .set_visible_child(&widgets.2);

        let active = active.clone();
        let widgets = widgets.clone();
        let model = model.clone();
        rx.attach(None, move |value| match value {
            Some(value) => {
                widgets.3
                    .set_fraction(f64::from(value) / 25.0);

                if value == 25 {
                    let widgets = widgets.clone();
                    model.installation.read().unwrap().launch();
                    gtk::timeout_add(1000, move || {
                        widgets.3.set_fraction(0.0);
                        widgets.0
                            .set_visible_child(&widgets.1);
                        glib::Continue(false)
                    });
                }

                glib::Continue(true)
            }
            None => {
                active.set(false);
                glib::Continue(false)
            }
        });
    }));
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

    let old = installation::Installation::try_load().unwrap_or_default();
    // let mut new = old.clone();
    // download::download(&mut new).await.unwrap();
    println!("update available: {:?}", download::is_update_available(&old).await);
    // new.save_changes().unwrap();

    let uiapp = gtk::Application::new(Some("fun.openfortress.ofmice"),
                    ApplicationFlags::FLAGS_NONE)
                .expect("Application::new failed");

    uiapp.connect_activate(build_ui);

    uiapp.run(&std::env::args().collect::<Vec<_>>());
}
