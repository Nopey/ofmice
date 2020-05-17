//! Platform specific code including:
//! * Choosing what binaries to install
//! * Choosing where to install
//! * Running SSDK2013
//!
//! Hopefully these will be dissolved into proper cross-platform code.


use std::path::PathBuf;
use std::env;


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
    PathBuf::from(env::var("HOME").unwrap()).join(".local/share/of")
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
