//! main is the vgtk frontend of the code. Perhaps it should be moved into its own interface module?
#![deny(clippy::all)]
mod platform;
mod download;
mod installation;
#[cfg(feature = "steam_wrangler")]
mod steam_wrangler;

use crate::installation::Installation;
use std::sync::{Arc, RwLock};
use std::rc::Rc;
use std::cell::Cell;
use std::time::Duration;
use std::thread;
use std::path::Path;

use gtk::prelude::*;
use gio::prelude::*;
use gdk_pixbuf::Pixbuf;
use gio::{ApplicationFlags, Cancellable, MemoryInputStream};
use glib::{Bytes, clone};
use gtk::*;
use lazy_static::lazy_static;

use crate::platform::ssdk_exe;

#[derive(Debug, Clone, Copy)]
pub enum WranglerError{
    SteamNotRunning,
    SSDKNotInstalled,
    TF2NotInstalled,
}

fn load_bg() -> Pixbuf {
    static BG: &[u8] = include_bytes!("res/bg.png");
    let data_stream = MemoryInputStream::new_from_bytes(&Bytes::from_static(BG));
    Pixbuf::new_from_stream(&data_stream, None as Option<&Cancellable>).unwrap()
}

fn load_logo() -> Pixbuf {
    static DATA: &[u8] = include_bytes!("res/logo.svg");
    let data_stream = MemoryInputStream::new_from_bytes(&Bytes::from_static(DATA));
    Pixbuf::new_from_stream(&data_stream, None as Option<&Cancellable>).unwrap()
}

fn load_play_icon() -> Pixbuf {
    static ICON: &[u8] = include_bytes!("res/play.png");
    let data_stream = MemoryInputStream::new_from_bytes(&Bytes::from_static(ICON));
    Pixbuf::new_from_stream(&data_stream, None as Option<&Cancellable>).unwrap()
}

fn load_config_icon() -> Pixbuf {
    static ICON: &[u8] = include_bytes!("res/config.png");
    let data_stream = MemoryInputStream::new_from_bytes(&Bytes::from_static(ICON));
    Pixbuf::new_from_stream(&data_stream, None as Option<&Cancellable>).unwrap()
}

fn load_css() -> &'static [u8] {
    include_bytes!("res/main.css")
}

fn load_glade() -> &'static str {
    include_str!("res/main.glade")
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
        let mut installation = Installation::try_load().unwrap_or_default();
        installation.init_ssdk().unwrap();
        Model{
            installation: Arc::new(RwLock::new(
                installation
            ))
        }
    }
}

fn build_ui(application: &gtk::Application) {
    // Build our UI from ze XML
    let builder = Builder::new_from_string(load_glade());

    // Apply the CSS to ze XML
    let provider = CssProvider::new();
    provider
        .load_from_data(load_css())
        .expect("Failed to load CSS");
    StyleContext::add_provider_for_screen(
        &gdk::Screen::get_default().expect("Error initializing gtk css provider."),
        &provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Errorbox setup goes here

    let model = Rc::New(Model::new());

    // window needs application
    let window: ApplicationWindow = builder.get_object("window").unwrap();
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

    // Save the config when the config tab is navigated away from
    let home_screen: Notebook = builder.get_object("home_screen").unwrap();
    home_screen.connect_switch_page(clone!( @weak model => |homescreen, _page, _page_num| {
        if homescreen.get_current_page()==Some(1) {
            model.installation.read().unwrap().save_changes().expect("TODO: FIXME: THIS SHOULD DISPLAY AN ERR TO USER");
        }
    }));

    let ssdk_path: Entry = builder.get_object("ssdk_path").unwrap();
    ssdk_path.connect_focus_out_event(clone!(@weak model => move |_widget, _event| {
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
    }));

    connect_progress(&builder, model);

    window.show_all();
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
        rx.attach(None, clone!(@weak model => move |value| match value {
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
        }));
    }));
}


fn set_visual(window: &ApplicationWindow, _screen: Option<&gdk::Screen>) {
    if let Some(screen) = window.get_screen() {
        if let Some(ref visual) = screen.get_rgba_visual() {
            window.set_visual(Some(visual)); // crucial for transparency
        }
    }
}

fn draw(_window: &ApplicationWindow, ctx: &cairo::Context) -> Inhibit {
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
