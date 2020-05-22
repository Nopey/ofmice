//! Launcher resources that get glued into the binary
use gdk_pixbuf::Pixbuf;
use gio::{Cancellable, MemoryInputStream};
use glib::Bytes;

pub fn load_bg() -> Pixbuf {
    static BG: &[u8] = include_bytes!("bg.png");
    let data_stream = MemoryInputStream::new_from_bytes(&Bytes::from_static(BG));
    Pixbuf::new_from_stream(&data_stream, None as Option<&Cancellable>).unwrap()
}

pub fn load_logo() -> Pixbuf {
    static DATA: &[u8] = include_bytes!("logo.svg");
    let data_stream = MemoryInputStream::new_from_bytes(&Bytes::from_static(DATA));
    Pixbuf::new_from_stream(&data_stream, None as Option<&Cancellable>).unwrap()
}

pub fn load_play_icon() -> Pixbuf {
    static ICON: &[u8] = include_bytes!("play.png");
    let data_stream = MemoryInputStream::new_from_bytes(&Bytes::from_static(ICON));
    Pixbuf::new_from_stream(&data_stream, None as Option<&Cancellable>).unwrap()
}

pub fn load_config_icon() -> Pixbuf {
    static ICON: &[u8] = include_bytes!("config.png");
    let data_stream = MemoryInputStream::new_from_bytes(&Bytes::from_static(ICON));
    Pixbuf::new_from_stream(&data_stream, None as Option<&Cancellable>).unwrap()
}

pub fn load_close_icon() -> Pixbuf {
    static ICON: &[u8] = include_bytes!("close.png");
    let data_stream = MemoryInputStream::new_from_bytes(&Bytes::from_static(ICON));
    Pixbuf::new_from_stream(&data_stream, None as Option<&Cancellable>).unwrap()
}

const BUTTON_COUNT: i32 = 5;
pub fn load_button_pixbufs() -> [Pixbuf; BUTTON_COUNT as usize] {
    static DATA: &[u8] = include_bytes!("buttons.png");
    let data_stream = MemoryInputStream::new_from_bytes(&Bytes::from_static(DATA));
    let master = Pixbuf::new_from_stream(&data_stream, None as Option<&Cancellable>).unwrap();
    let height = master.get_height()/BUTTON_COUNT;
    let mut iter = (0..BUTTON_COUNT).map(|i| {
        let pixbuf = Pixbuf::new(
            master.get_colorspace(),
            master.get_has_alpha(),
            master.get_bits_per_sample(),
            master.get_width(),
            height
        ).unwrap();
        master.copy_area(0, height*i, master.get_width(), height, &pixbuf, 0, 0);
        pixbuf
    });
    // really ugly but i don't want to pull in a dep to make it better
    // and dont know how else to do it.
    [
        iter.next().unwrap(),
        iter.next().unwrap(),
        iter.next().unwrap(),
        iter.next().unwrap(),
        iter.next().unwrap()
    ]
}

#[cfg(debug_assertions)]
pub fn load_css() -> Vec<u8> {
    use std::io::Read;
    let mut content = vec![];
    std::fs::File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/src/res/main.css"))
        .unwrap().read_to_end(&mut content).unwrap();
    content
}
#[cfg(not(debug_assertions))]
pub fn load_css() -> &'static [u8] {
    include_bytes!("main.css")
}

#[cfg(debug_assertions)]
pub fn load_glade() -> String {
    use std::io::Read;
    let mut content = String::new();
    std::fs::File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/src/res/main.glade"))
        .unwrap().read_to_string(&mut content).unwrap();
    content
    
}
#[cfg(not(debug_assertions))]
pub fn load_glade() -> &'static str {
    include_str!("main.glade")
}
