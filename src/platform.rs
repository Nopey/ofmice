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
//NOTE: the client and server buckets are just assets, with no overlap.
// Client and server binary buckets both contain server.so, so only install the client.
#[cfg(target_os = "linux")]
pub fn bins() -> &'static [&'static str] {
    return &["client_linux", "client", "server"];
}

#[cfg(target_os = "windows")]
pub fn bins() -> &'static [&'static str] {
    return &["client_windows", "client", "server"];
}

#[cfg(target_os = "linux")]
pub fn ofmice_binary_name() -> &'static str {
    "ofmice"
}

#[cfg(target_os = "windows")]
pub fn ofmice_binary_name() -> &'static str {
    "ofmice.exe"
}