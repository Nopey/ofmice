//! Handles extraction and tracking of bins.
//! Perhaps it could handle more of the bin logic, relating to downloads?
//! For now, it will just maintain a record of what's installed.
use crate::platform::of_path;

use std::fs::File;
use std::collections::HashMap;
use std::path::PathBuf;

use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Installation {
    //TODO: Privatize this, make some getters
    pub bins: HashMap<String, InstalledBin>,
    //TODO: Launch options
    //TODO: Cache TF2 and SSDK location
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
    pub fn save_changes( &self ) -> Result<(),&'static str>{
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