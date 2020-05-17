use crate::platform;
use crate::platform::of_path;
use crate::installation::*;

use std::collections::HashMap;
use std::ffi::OsStr;
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
    //TODO: Patch newtype.
    pub patch_tail: u64,
}

const BASEURL: &'static str = "https://larsenml.ignorelist.com:8443/of/mice/";

pub async fn is_update_available(installation: &Installation) -> Result<bool, DownloadError> {
    use DownloadError::*;
    //TODO: replace some of these unwraps with BadResponse masks
    //TODO: make a function that eprintln!'s the error before returning BadResponse.
    // (maybe report the error, too)

    // Install our self signed certificate
    // easier than ensurnig letsencrypts is trusted by reqwest, and hasn't expired.
    let cert = Certificate::from_pem(include_bytes!("of.ssl.cert")).unwrap();
    let client = Client::builder()
        .add_root_certificate(cert).build().unwrap();

    //TODO: Launcher self update detection
    // Download the index that describes what's available
    let index: Index = {
        let response = client.get(BASEURL).send().await;
        response.map_err(|_| ConnectionFailure)?
            .json().await.map_err(|e| {eprintln!("Response err msg: {:?}", e); BadResponse})?
    };

    // each bin is a set of files needed for the game.
    // Ideally, we'd have platform-specific binary ones, a barebones server assets one, and the textures&audio.
    let bins = platform::bins().iter();

    for &bin in bins {
        let bindex = index.bindices.get(bin).expect("All valid bins must be in the index!");
        let binst = match installation.bins.get(bin){
            Some(b) => b,
            None => {return Ok(true);} // update available
        };
        let oldversion = binst.version;
        let delta_dist = bindex.version - oldversion;
        //TODO: Signature checking
        if delta_dist!=0 {
            // up-to-date
            return Ok(true);
        }
    }
    Ok(false)
}

//TODO: Stream the download through xz and tar
/// Downloads the latest update
pub async fn download(installation: &mut Installation) -> Result<(), DownloadError> {
    use DownloadError::*;
    //TODO: replace some of these unwraps with BadResponse masks
    //TODO: make a function that eprintln!'s the error before returning BadResponse.
    // (maybe report the error, too)

    // Install our self signed certificate
    // easier than ensurnig letsencrypts is trusted by reqwest, and hasn't expired.
    let cert = Certificate::from_pem(include_bytes!("of.ssl.cert")).unwrap();
    let client = Client::builder()
        .add_root_certificate(cert).build().unwrap();

    //TODO: Launcher self update detection
    // Download the index that describes what's available
    let index: Index = {
        let response = client.get(BASEURL).send().await;
        response.map_err(|_| ConnectionFailure)?
            .json().await.map_err(|e| {eprintln!("Response err msg: {:?}", e); BadResponse})?
    };

    // each bin is a set of files needed for the game.
    // Ideally, we'd have platform-specific binary ones, a barebones server assets one, and the textures&audio.
    let bins = platform::bins().iter();

    for &bin in bins {
        println!("[download] considering bin {}", bin);
        let bindex = index.bindices.get(bin).expect("All valid bins must be in the index!");
        let mut binst = &mut installation.bins.entry(bin.to_owned()).or_insert_with(|| {println!("clearing a bin");InstalledBin::new()});
        let oldversion = binst.version;
        println!("installed ver:\t{}\nindex ver:\t{}", oldversion, bindex.version);
        let delta_dist = bindex.version - oldversion;
        //TODO: Signature checking
        if delta_dist==0 {
            // up-to-date
            println!("up-to-date");
        }else if oldversion!=0 && delta_dist <= bindex.patch_tail /* && signatures match */{
            println!("patchable");

            // mark uninstalled (if we're interrupted or fail, don't attempt to patch)
            binst.version = 0;
            drop(binst);
            installation.save_changes().map_err(|_| WriteErr)?;

            for patch_id in oldversion..bindex.version{
                let url = format!("{}{}-patch{}.tar.xz", BASEURL, bin, patch_id);
                let dottarxz = client.get(&url).send().await
                    .map_err(|_| ConnectionFailure)?.bytes().await
                    .map_err(|_| ConnectionFailure)?;
                let dottar = read::XzDecoder::new(dottarxz.as_ref());
                
                let mut ar = Archive::new(dottar);
                for file in ar.entries().unwrap() {
                    let mut f = file.map_err(|_| BadResponse)?;
                    let path = f.path().unwrap();
                    match path.extension().and_then(OsStr::to_str) {
                        Some("del") => std::fs::remove_file(
                                of_path().join(path.parent().unwrap()).join(path.file_stem().unwrap())
                            ).map_err(|_| WriteErr)?,
                        Some("dif") => {// okay, stay calm. we know how to do this.
                            // get real filename
                            let real_filename = of_path()
                                .join(path.parent().unwrap())
                                .join(path.file_stem().unwrap());
                            // the temporary version is named .dif
                            let temp_filename = of_path().join(path);

                            // apply the dif
                            // chunked alleviates the 2.14GB restriction of ddelta
                            let mut outfile = File::create( &temp_filename ).map_err(|_| WriteErr)?;
                            //NOTE: once we check checksums, realfile won't fail because of missing input.
                            let mut realfile = File::open( &real_filename ).map_err(|_| WriteErr)?;
                            ddelta::apply_chunked(&mut realfile, &mut outfile, &mut f)
                                .map_err(|_| BadResponse)?;
                            drop(realfile);
                            drop(outfile);
                            std::fs::rename(&temp_filename, &real_filename).map_err(|_| WriteErr)?;

                            //TODO: Signature recording
                        },
                        _ => {
                            let outpath = of_path().join(f.path().map_err(|_| BadResponse)?);
                            std::fs::create_dir_all(outpath.parent().unwrap()).map_err(|_| BadResponse)?;
                            f.unpack_in(of_path()).map_err(|_| WriteErr)?;

                            //TODO: Signature recording
                        }
                    }
                }
            }
            todo!()
        }else{
            println!("full-download");
            // mark uninstalled
            binst.version = 0;
            drop(binst);
            installation.save_changes().map_err(|_| WriteErr)?;
            
            // Delete every previously installed file
            for file in &installation.bins[bin].files {
                // if it fails, we don't really care.
                std::fs::remove_file(file).ok();
            }

            // Must download from scratch
            let url = format!("{}{}.tar.xz", BASEURL, bin);
            let dottarxz = client.get(&url).send().await
                .map_err(|_| ConnectionFailure)?.bytes().await
                .map_err(|_| ConnectionFailure)?;
            let dottar = read::XzDecoder::new(dottarxz.as_ref());
            
            let mut ar = Archive::new(dottar);
            for file in ar.entries().unwrap() {
                let mut f = file.map_err(|_| BadResponse)?;
                let outpath = of_path().join(f.path().map_err(|_| BadResponse)?);
                std::fs::create_dir_all(outpath.parent().unwrap()).map_err(|_| BadResponse)?;
                f.unpack_in(of_path()).map_err(|_| WriteErr)?;
                installation.bins.get_mut(bin).unwrap().files.push(outpath);

                //TODO: Signature recording
            }
            installation.bins.get_mut(bin).unwrap().version = bindex.version;
            installation.save_changes().map_err(|_| WriteErr)?;
        }
    }
    Ok(())
}
