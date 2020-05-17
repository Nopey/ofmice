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
use vgtk::lib::gio::{Cancellable, MemoryInputStream, Icon};
use vgtk::lib::glib::Bytes;
// use vgtk::lib::gtk::prelude::*;
use vgtk::{gtk, run, Component, UpdateAction, VNode};

#[derive(Clone, Debug, Default)]
struct Model {}

#[derive(Clone, Debug)]
enum Message {
    Noop,
    Exit,
}

fn load_bg() -> Pixbuf {
    static BG: &[u8] = include_bytes!("bg.png");
    let data_stream = MemoryInputStream::new_from_bytes(&Bytes::from_static(BG));
    Pixbuf::new_from_stream(&data_stream, None as Option<&Cancellable>).unwrap()
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
        }
    }

    fn view(&self) -> VNode<Model> {
        gtk! {
            <Application::new_unwrap(Some("fun.openfortress.ofmice"), ApplicationFlags::empty())>
                <Window app_paintable=true resizable=false on destroy=|_| Message::Exit>
                    // The Grid puts background behind the stack,
                    // That's my idea of a good hack.
                    <Grid>
                        <Stack>
                            // <Label label="Hello!" valign=Align::Center halign=Align::Center />
                            <Image
                                /*pixbuf=Pixbuf::new_from_resource("document-save").ok()*/
                                valign=Align::Center halign=Align::Center
                                // this isn't working
                                on configure_event=|i, _| {
                                    i.set_from_gicon(&Icon::new_for_string("process-working").unwrap(), IconSize::LargeToolbar);
                                    Message::Noop
                                }
                            />
                            <Box orientation=Orientation::Vertical spacing=12 valign=Align::Center>
                                /* Loading and Status */
                                <ProgressBar text="Progress Bar" show_text=true hexpand=true/>
                                <Button label="Start" halign=Align::Center />
                            </Box>
                        </Stack>
                        <Image pixbuf=Some(load_bg()) />
                    </Grid>
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
