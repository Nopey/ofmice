//! main is the vgtk frontend of the code. Perhaps it should be moved into its own interface module?
#![deny(clippy::all)]
mod steam_wrangler;
mod platform;
mod download;
mod installation;

use gtk::prelude::*;
use gio::prelude::*;
use gdk_pixbuf::Pixbuf;
use gio::{ApplicationFlags, Cancellable, MemoryInputStream};
use glib::Bytes;

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

fn build_ui(application: &gtk::Application) {
    use gtk::*;

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

    // window needs application
    let window: ApplicationWindow = builder.get_object("window").unwrap();
    window.set_application(Some(application));

    // Set the background image
    let background: Image = builder.get_object("background").unwrap();
    background.set_from_pixbuf(Some(&load_bg()));

    // Overlay the stack onto the background
    let overlay: Overlay = builder.get_object("overlay").unwrap();
    let stack: Stack = builder.get_object("stack").unwrap();
    overlay.add_overlay(&stack);

    // Set the tab's icons
    let play_tabicon: Image = builder.get_object("play-tab").unwrap();
    play_tabicon.set_from_pixbuf(Some(&load_play_icon()));
    let config_tabicon: Image = builder.get_object("config-tab").unwrap();
    config_tabicon.set_from_pixbuf(Some(&load_config_icon()));

    // Set the main logo
    let logo: Image = builder.get_object("logo").unwrap();
    logo.set_from_pixbuf(Some(&load_logo()));

    window.show_all();
}

#[tokio::main]
async fn main() {
    // pretty_env_logger::init();

    let old = installation::Installation::try_load().unwrap_or_default();
    let mut new = old.clone();
    download::download(&mut new).await.unwrap();
    new.save_changes().unwrap();


    let uiapp = gtk::Application::new(Some("fun.openfortress.ofmice"),
                    ApplicationFlags::FLAGS_NONE)
                .expect("Application::new failed");

    uiapp.connect_activate(build_ui);

    uiapp.run(&std::env::args().collect::<Vec<_>>());
}
