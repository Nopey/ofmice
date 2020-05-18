//TODO: HACK: i think we should make a lib.rs with everything in it.

#[path = "../platform.rs"]
pub mod platform;
#[path = "../installation.rs"]
pub mod installation;

// so things like this don't get copypasted
#[derive(Debug, Clone, Copy)]
pub enum WranglerError{
    SteamNotRunning,
    SSDKNotInstalled,
    TF2NotInstalled,
}

fn main(){
    println!("TODO: ofpatchtool");
    // find all the tarballs
    // trim stale patches
    // save index
    platform::all_valid_bins();
}