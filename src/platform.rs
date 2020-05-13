//! Platform specific code including:
//! * Choosing what binaries to install
//! * Choosing where to install
//! * Running SSDK2013
//!
//! Hopefully these will be dissolved into proper cross-platform code.

use std::path::PathBuf;
use std::ffi::OsStr;
use std::env;
use crate::steam_wrangler::*;

pub fn run_ssdk_2013<S, I> (args: I) -> Result<(), WranglerError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let ssdk_path = wrangle_steam_and_get_ssdk_path()?;
    //TODO: on linux, set LD_LIBRARY_PATH
    //TODO: impl run_ssdk_2013
    // https://doc.rust-lang.org/std/process/struct.Command.html
    unimplemented!()
}


#[cfg(target_os = "linux")]
pub fn of_path() -> PathBuf {
    PathBuf::from(env::var("HOME").unwrap()).join(".of")
}
#[cfg(target_os = "windows")]
pub fn of_path() -> PathBuf {
    PathBuf::from(env::var("APPDATA").unwrap()).join("of")
}

//TODO: HACK: This bins function needs to be refractored.
//TODO: all_valid_bins() which lists both windows or linux bins
pub fn bins() -> &'static [&'static str] {
    // something like:
    return &["bin_linux_client", "bin_linux_server", "content_client", "content_server"];
}
