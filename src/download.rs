use crate::platform;
use crate::platform::of_path;
use crate::installation::*;

use std::collections::HashMap;
use std::io::copy;
use std::fs::File;

use reqwest::{Client, Certificate};
use serde_derive::{Serialize, Deserialize};
use xz2::read;
use tar::Archive;

/// A user-friendly **actionable** error
#[derive(Debug, Clone, Copy)]
pub enum DownloadError{
    ConnectionFailure,
    WriteErr,
    BadResponse, // we effed up on the serverside :c
}

/// a list of patches and other downloads
#[derive(Debug, Deserialize)]
struct Index{
    bindices: HashMap<String, Bindex>,
}

/// The name is a pun of Bin and index.
#[derive(Debug, Serialize, Deserialize)]
pub struct Bindex {
    pub version: u64,
    pub patch_tail: Vec<u8>, //TODO: Patch type.
    /// A tar.xz that just contains the bin, with no funny business.
    pub complete_download: String, // TODO: this can probably be deduced from the bindex's name.
}

const BASEURL: &'static str = "https://larsenml.ignorelist.com:8443/of/mice/";

/// Downloads the latest update
pub async fn download(installation: &mut Installation) -> Result<(), DownloadError> {
    use DownloadError::*;

    //TODO: Check what's installed.

    // Install our self signed certificate
    let cert = Certificate::from_pem(include_bytes!("of.ssl.cert")).unwrap();
    let client = Client::builder()
        .add_root_certificate(cert).build().unwrap();

    // Download the index that describes what's available
    let index: Index = {
        let response = client.get(BASEURL).send().await;
        response.map_err(|e| {eprintln!("Response err msg: {:?}", e); ConnectionFailure})?
            .json().await.map_err(|_| BadResponse)?
    };

    // each bin is a set of files needed for the game.
    // Ideally, we'd have platform-specific binary ones, a barebones server assets one, and the textures&audio.
    //TODO: Maybe remove the bin function in favor of a std lib platform string
    let bins = platform::bins().iter();

    for &bin in bins {
        println!("[download] considering bin {}", bin);
        let bindex = index.bindices.get(bin).expect("All valid bins must be in the index!");
        let mut binst = &mut installation.bins.entry(bin.to_owned()).or_insert_with(|| {println!("clearing a bin");InstalledBin::new()});
        println!("installed ver:\t{}\nindex ver:\t{}", binst.version, bindex.version);
        let delta_dist = bindex.version - binst.version;
        //TODO: Signature checking
        if delta_dist==0 {
            // up-to-date
            println!("up-to-date");
        }else if delta_dist <= bindex.patch_tail.len() as u64 /* && signatures match */{
            // Patchable..?
            println!("patchable");
            todo!()
        }else{
            println!("full-download");
            // Delete every previously installed file
            binst.version = 0; // mark uninstalled
            drop(binst);
            installation.save_changes()
                .map_err(|_| WriteErr)?; // save uninstallation
            for file in &installation.bins[bin].files {
                std::fs::remove_file(of_path().join(file))
                    .map_err(|_| WriteErr)?;
            }

            // Must download from scratch
            let mut url = BASEURL.to_owned();
            url.push_str(bin);
            url.push_str(".tar.xz");
            let dottarxz = client.get(&url).send().await
                .map_err(|_| ConnectionFailure)?.bytes().await
                .map_err(|_| ConnectionFailure)?;
            let dottar = read::XzDecoder::new(dottarxz.as_ref());
            
            let mut ar = Archive::new(dottar);
            for file in ar.entries().unwrap() {
                let mut f = file.map_err(|_| BadResponse)?;
                //TODO: Signature recording
                let outpath = of_path().join(f.path().map_err(|_| BadResponse)?);
                std::fs::create_dir_all(outpath.parent().unwrap()).map_err(|_| BadResponse)?;
                f.unpack_in(of_path()).map_err(|_| WriteErr)?;
                // let mut outfile = File::create( outpath ).map_err(|_| WriteErr)?;
                // copy(&mut f, &mut outfile).unwrap();
            }
            installation.bins.get_mut(bin).unwrap().version = bindex.version;
            installation.save_changes().map_err(|_| WriteErr)?;
        }
    }
    Ok(())
}
