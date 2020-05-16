//! Platform specific code including:
//! * Choosing what binaries to install
//! * Choosing where to install
//! * Running SSDK2013
//!
//! Hopefully these will be dissolved into proper cross-platform code.

use crate::steam_wrangler::*;

use std::path::PathBuf;
use std::ffi::OsStr;
use std::env;
use std::process::Command;

pub fn run_ssdk_2013<S, I> (args: I) -> Result<(), WranglerError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let ssdk_path = wrangle_steam_and_get_ssdk_path()?;
    let mut cmd = Command::new(ssdk_exe());
    cmd.current_dir(&ssdk_path);
    if cfg!(linux){
        cmd.env("LD_LIBRARY_PATH", ssdk_path.join("bin"));
    }
    //TODO: set args like -game
    cmd.spawn().unwrap();

    Ok(())
}


#[cfg(target_os = "linux")]
pub fn ssdk_exe() -> &'static str {
    "hl2_linux"
}
#[cfg(target_os = "windows")]
pub fn ssdk_exe() -> &'static str {
    "hl2.exe"
}


#[cfg(target_os = "linux")]
pub fn of_path() -> PathBuf {
    PathBuf::from(env::var("HOME").unwrap()).join(".of")
}
#[cfg(target_os = "windows")]
pub fn of_path() -> PathBuf {
    PathBuf::from(env::var("APPDATA").unwrap()).join("of")
}

//TODO: I'm uncomfortable with these bins functions, Too bad!
#[cfg(target_os = "linux")]
pub fn bins() -> &'static [&'static str] {
    return &["bin_linux_client", "bin_linux_server", "content_client", "content_server"];
}

#[cfg(target_os = "windows")]
pub fn bins() -> &'static [&'static str] {
    return &["bin_windows_client", "bin_windows_server", "content_client", "content_server"];
}

pub fn all_valid_bins() -> &'static [&'static str] {
    //TODO: bin_xxx_xxx_dbg?
    return &["bin_linux_client", "bin_linux_server", "bin_windows_client", "bin_windows_server", "content_client", "content_server"];
}
