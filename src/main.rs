//! main is the vgtk frontend of the code. Perhaps it should be moved into its own interface module?
#![deny(clippy::all)]
#![recursion_limit="1024"]
mod steam_wrangler;
mod platform;
mod download;
mod installation;

use vgtk::ext::*;
use vgtk::lib::gio::ApplicationFlags;
use vgtk::lib::gtk::*;
use vgtk::lib::gdk_pixbuf::Pixbuf;
use vgtk::lib::gio::{Cancellable, MemoryInputStream};
use vgtk::lib::glib::Bytes;
// use vgtk::lib::gtk::prelude::*;
use vgtk::{gtk, run, Component, UpdateAction, VNode};

use crate::steam_wrangler::*;
use crate::platform::*;

#[derive(Clone, Debug, Default)]
struct Model {}

#[derive(Clone, Debug)]
enum Message {
    // Noop,
    Exit,
    Start
}

fn load_bg() -> Pixbuf {
    static BG: &[u8] = include_bytes!("bg.png");
    let data_stream = MemoryInputStream::new_from_bytes(&Bytes::from_static(BG));
    Pixbuf::new_from_stream(&data_stream, None as Option<&Cancellable>).unwrap()
}

fn get_path() -> String {
    let p = wrangle_steam_and_get_ssdk_path();
    assert_eq!(p.is_ok(), true);


    p.unwrap().into_os_string().into_string().unwrap()
}

impl Component for Model {
    type Message = Message;
    type Properties = ();

    fn update(&mut self, msg: Self::Message) -> UpdateAction<Self> {
        match msg {
            // Message::Noop => UpdateAction::None,
            Message::Exit => {
                vgtk::quit();
                UpdateAction::None
            },

            Message::Start => {
                println!("Starting the game!");
                run_ssdk_2013(std::env::args());
                UpdateAction::None
            },
        }
    }

    fn view(&self) -> VNode<Model> {
        gtk! {
            <Application::new_unwrap(Some("fun.openfortress.ofmice"), ApplicationFlags::empty())>
                <Window title="Open Fortress Launcher" app_paintable=true resizable=false on destroy=|_| Message::Exit>
                    // The Grid puts background behind the stack,
                    // That's my idea of a good hack.

                    <Box orientation=Orientation::Vertical>
                        <Label label=get_path()/>
                        <Button label="Start" on clicked=|_| Message::Start/>
                        <ProgressBar text="Progress Bar" show_text=true hexpand=true/>
                    </Box>
                    // <Grid>
                        // <Stack>
                            // <Grid row_spacing=12 vexpand=true hexpand=true border_width=6>
                                /* Loading and Status */
                            // </Grid>
                        // </Stack>
                        // <Image pixbuf=Some(load_bg()) />
                    // </Grid>
                </Window>
            </Application>
        }
    }
}

#[tokio::main]
async fn main() {
    let old = installation::Installation::try_load().unwrap_or_default();
    let mut new = old.clone();
    download::download(&mut new).await.unwrap();
    new.save_changes().unwrap();
    pretty_env_logger::init();
    std::process::exit(run::<Model>());
}
