//! Handles extraction and tracking of bins.
//! Perhaps it could handle more of the bin logic, relating to downloads?
//! For now, it will just maintain a record of what's installed.
use crate::platform::{of_path, ssdk_exe};

use std::fs::File;
use std::collections::HashMap;
use std::path::PathBuf;
use std::ffi::{OsString, OsStr};
use std::process::Command;

use serde_derive::{Serialize, Deserialize};
use crate::WranglerError;
use std::collections::hash_map::DefaultHasher;
use std::io;
use std::io::Read;
use std::hash::{Hash, Hasher};

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

    #[cfg(feature = "steam_wrangler")]
    pub fn init_ssdk(&mut self) -> Result<(), WranglerError> {
        use crate::steam_wrangler::*;
        let r = wrangle_steam_and_get_ssdk_path()?;
        if self.ssdk_path.as_os_str().is_empty() {
            self.ssdk_path = r;
        }
        Ok(())
    }

    #[cfg(not(feature = "steam_wrangler"))]
    pub fn init_ssdk(&mut self) -> Result<(), WranglerError> {
        println!("No steam wrangler");
        Ok(())
    }
    /// Saves the installation to the file.
    /// Replaces it atomically with renaming.
    pub fn save_changes(&self) -> Result<(), &'static str> {
        let temp_name = of_path().join(".").join(TRACK_FILE);
        let real_name = of_path().join(TRACK_FILE);
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
        if self.ssdk_path.as_os_str().is_empty() {
            eprintln!("installation: cowardly not launching");
            return;
        }

        let mut args = self.get_launch_args();
        let ssdk_cmd = self.ssdk_path.join(ssdk_exe()).into_os_string();
        let cmd = if let Some(idx) = args.iter().position(|arg| arg == OsStr::new("%command%")) {
            args[idx] = ssdk_cmd;
            args.remove(0)
        } else {
            ssdk_cmd
        };

        let mut cmd = Command::new(&cmd);
        cmd.current_dir(&self.ssdk_path);
        if cfg!(target_os="linux") {
            cmd.env("LD_LIBRARY_PATH", self.ssdk_path.join("bin"));
        }
        cmd.args(args);
        cmd.spawn().expect("failed to launch game");
    }

    fn get_launch_args(&self) -> Vec<OsString> {
        let mut args = vec![];

        // I'm sure this is implemented somewhere, but I don't feel like finding it.
        // passing it to `sh` would work, but isn't portable.
        let mut arg = String::new();
        let mut escaped = false;
        let mut quote = ' ';// ' or " for yes
        for c in self.launch_options.chars() {
            if quote == '\'' {
                if c == '\'' {
                    quote = ' ';
                } else {
                    arg.push(c);
                }
            } else if escaped {
                escaped = false;
                arg.push(c);
            } else if c == '\\' {
                escaped = true;
            } else if quote == '\"' {
                if c == '\"' {
                    quote = ' ';
                } else {
                    arg.push(c);
                }
            } else if c == '\"' {
                quote = '\"';
            } else if c.is_whitespace() {
                if !arg.is_empty() {
                    args.push(arg.to_string().into());
                }
                arg.clear();
            } else {
                arg.push(c);
            }
        }
        if !arg.is_empty() {
            args.push(arg.into());
        }

        args.push("-game".to_string().into());
        args.push(of_path().join("open_fortress").into_os_string());

        dbg!(args)
    }

    pub fn are_paths_good(&self) -> bool {
        !self.ssdk_path.as_os_str().is_empty()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct InstalledBin {
    pub version: u32,
    pub files: Vec<InstalledFile>
}

impl InstalledBin {
    pub fn new() -> Self {
        InstalledBin {
            version: 0,
            files: vec![]
        }
    }

    pub fn is_not_modified(&self) -> bool {
        self.files.iter().all(|file| file.check().unwrap_or(false))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct InstalledFile {
    pub fullpath: PathBuf,
    checksum: u64,
}

impl InstalledFile {
    pub fn new(fullpath: PathBuf) -> Result<Self, io::Error> {
        let checksum = Self::hash_file(&fullpath)?;

        Ok(InstalledFile {
            fullpath,
            checksum
        })
    }

    pub fn check(&self) -> Result<bool, io::Error> {
        Ok(self.checksum == Self::hash_file(&self.fullpath)?)
    }

    fn hash_file(path: &PathBuf) -> Result<u64, io::Error> {
        let mut hasher = DefaultHasher::new();

        let mut file = File::open(path.as_path())?;
        let mut buf = [0; 64];

        while let Ok(n) = file.read(&mut buf) {
            Hash::hash(&buf[0..n], &mut hasher);
        }

        Ok(hasher.finish())
    }
}