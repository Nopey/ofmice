//! Handles extraction and tracking of bins.
//! Perhaps it could handle more of the bin logic, relating to downloads?
//! For now, it will just maintain a record of what's installed.
use crate::platform::{of_path, ssdk_exe};

use std::fs::File;
use std::collections::HashMap;
use std::path::PathBuf;
use std::ffi::OsString;
use std::process::Command;

use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Installation {
    //TODO: Privatize this, make some getters
    pub bins: HashMap<String, InstalledBin>,
    pub launch_options: String,
    pub ssdk_path: PathBuf,
    pub tf2_path: PathBuf,
}

const TRACK_FILE: &'static str = "installation.json";
impl Installation {
    /// Try to load preexisting installation details
    pub fn try_load() -> Result<Self, ()> {
        let name = of_path().join(TRACK_FILE);
        let file = File::open(name).map_err(|_| ())?;
        serde_json::from_reader(file).map_err(|_| ())
    }
    /// Saves the installation to the file.
    /// Replaces it atomically with renaming.
    pub fn save_changes( &self ) -> Result<(),&'static str> {
        let temp_name = of_path().join(".").join(TRACK_FILE);
        let real_name = of_path()          .join(TRACK_FILE);
        std::fs::create_dir_all(of_path())
            .map_err(|_| "couldn't create installation dir")?;
        let temp = File::create(&temp_name)
            .map_err(|_| "couldn't create file in installation")?;
        // serde_json::to_writer_pretty(temp, self)
        serde_json::to_writer(temp, self)
            .map_err(|_| "couldn't serialize installation.json")?;
        std::fs::rename(&temp_name, &real_name)
            .map_err(|_| "couldn't overwrite previous installation record")?;
        Ok(())
    }
    /// run game
    /// very little checking, just goes for it
    pub fn launch(&self) {
        let mut cmd = Command::new(ssdk_exe());
        cmd.current_dir(&self.ssdk_path);
        if cfg!(linux){
            cmd.env("LD_LIBRARY_PATH", self.ssdk_path.join("bin"));
        }
        //TODO: set args like -game
        cmd.args(self.get_launch_args());
        cmd.spawn().unwrap();
    }

    fn get_launch_args(&self) -> Vec<OsString> {
        let mut args = vec![];

        args.push("-game".to_string().into());
        args.push(of_path().join("open_fortress").into_os_string());

        // I'm sure this is implemented somewhere, but I don't feel like finding it.
        // passing it to `sh` would work, but isn't portable.
        let mut arg = String::new();
        let mut escaped = false;
        let mut quote = ' ';// ' or " for yes
        for c in self.launch_options.chars(){
            if quote=='\'' {
                if c=='\'' {
                    quote = ' ';
                }else{
                    arg.push(c);
                }
            }else if escaped {
                escaped = false;
                arg.push(c);
            }else if c=='\\' {
                escaped = true;
            }else if quote=='\"' {
                if c=='\"'{
                    quote = ' ';
                }else{
                    arg.push(c);
                }
            }else if c=='\"'{
                quote = '\"';
            }else if c.is_whitespace() {
                if !arg.is_empty(){
                    args.push(arg.to_string().into());
                }
                arg.clear();
            }else{
                arg.push(c);
            }
        }
        if !arg.is_empty(){
            args.push(arg.into());
        }
        args
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct InstalledBin {
    pub version: u64,
    pub files: Vec<PathBuf>,
    //TODO: integrity checking belongs here
}

impl InstalledBin{
    pub fn new() -> Self {
        InstalledBin{
            version: 0,
            files: vec![],
        }
    }
}