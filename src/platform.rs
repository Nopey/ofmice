//!
//! Platform specific code including:
//! * Choosing what binaries to install
//! * Choosing where to install
//! * Running SSDK2013
//!
use std::path::PathBuf;
use std::ffi::OsStr;
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

pub fn of_path() -> PathBuf {
    // Linux:   ~/.of
    // Windows: %APPDATA%/of
    // with the install happening inside an "open_fortress" subfolder.
    unimplemented!()
}

pub fn bins() -> &[str] {
    // something like:
    return &["base", "client", "linux"];
}