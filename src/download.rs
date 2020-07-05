use crate::platform;
use crate::platform::of_path;
use crate::installation::*;
use crate::progress::Progress;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::time::Duration;

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
#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
    pub bindices: HashMap<String, Bindex>,
}

impl Index {
    async fn get(client: &Client) -> Result<Self, DownloadError> {
        let response = client.get(&format!("{}{}", BASEURL, "index.json")).send().await;
        response.map_err(connection_failure)?
            .json().await.map_err(bad_response)
    }
}

fn bad_response<E: std::fmt::Debug + Sized>(e: E) -> DownloadError {
    eprintln!("BadResponse: {:?}", e);
    DownloadError::BadResponse
}

fn connection_failure<E: std::fmt::Debug + Sized>(e: E) -> DownloadError {
    eprintln!("ConnectionFailure: {:?}", e);
    DownloadError::ConnectionFailure
}

/// The name is a pun of Bin and index.
#[derive(Debug, Serialize, Deserialize)]
pub struct Bindex {
    pub version: u32,
    //TODO: Patch newtype.
    pub patch_tail: u32,
}

fn make_client() -> Client{
    // Install our self signed certificate
    // easier than ensurnig letsencrypts is trusted by reqwest, and hasn't expired.
    // let cert = Certificate::from_pem(include_bytes!("of-ca.cert")).unwrap();
    // baltimore root cert, useful for cloudflare
    let cert = Certificate::from_pem(include_bytes!("baltimore-root.cert")).unwrap();
    let client = Client::builder()
        .timeout(Duration::new(8,0)) // 8 second timeout
        .add_root_certificate(cert)
        .build().unwrap();
    client
}

// const BASEURL: &'static str = "https://larsenml.ignorelist.com:8443/of/mice/";
const BASEURL: &'static str = "https://mice.openfortress.fun/of/mice/";

pub async fn self_update() -> Result<(), DownloadError> {
    let client = make_client();
    let new_version = client.get(&format!("{}{}", BASEURL, "launcher_version.txt")).send()
        .await.map_err(bad_response)?.text().await.map_err(bad_response)?;
    if new_version!=env!("CARGO_PKG_VERSION"){
        // ooga booga we updating boys
        todo!()
    }
    Ok(())
}

pub async fn is_update_available(installation: &Installation) -> Result<bool, DownloadError> {
    //TODO: replace some of these unwraps with BadResponse masks
    //TODO: make a function that eprintln!'s the error before returning BadResponse.
    // (maybe report the error, too)

    let client = make_client();

    //TODO: Launcher self update detection
    // Download the index that describes what's available
    let index = Index::get(&client).await?;

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

//TODO: Stream the download through xz and tar?
/// Downloads the latest update
pub async fn download(inst: &mut Installation, progress: Progress<'_>) -> Result<(), DownloadError>{
    use DownloadError::*;
    //TODO: replace some of these unwraps with BadResponse masks
    //TODO: make a function that eprintln!'s the error before returning BadResponse.
    // (maybe report the error, too)
    progress.send(0f64, "Downloading index");

    let client = make_client();

    //TODO: Launcher self update detection
    // Download the index that describes what's available
    let index = Index::get(&client).await?;

    // each bin is a set of files needed for the game.
    // Ideally, we'd have platform-specific binary ones, a barebones server assets one, and the textures&audio.
    let bins = platform::bins().iter();

    for (progress, &bin) in progress.over(bins, "Checking Bin") {
        println!("[download] considering bin {}", bin);
        progress.send(0f64, &format!("{}: Checking Signature", bin));
        let bindex = index.bindices.get(bin).expect("All valid bins must be in the index!");
        let mut binst = &mut inst.bins.entry(bin.to_owned()).or_insert_with(|| {println!("clearing a bin");InstalledBin::new()});
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
            binst.files.clear();
            drop(binst);
            inst.save_changes().map_err(|_| WriteErr)?;

            for (progress, patch_id) in progress.over(oldversion..bindex.version, "Applying Patch") {
                progress.send(0f64, "Downloading");
                let url = format!("{}{}-patch{}.tar.xz", BASEURL, bin, patch_id);
                let dottarxz = client.get(&url).send().await
                    .map_err(|_| ConnectionFailure)?.bytes().await
                    .map_err(|_| ConnectionFailure)?;
                let dottar = read::XzDecoder::new(dottarxz.as_ref());

                progress.send(0.5f64, "Applying");
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
            let files = std::mem::take(&mut binst.files);
            drop(binst);
            inst.save_changes().map_err(|_| WriteErr)?;
            
            // Delete every previously installed file
            for file in files {
                // if it fails, we don't really care.
                std::fs::remove_file(file).ok();
            }

            // I split the iter here so that I can
            // split it again for the Download.
            let mut piter = progress.divide(2, "Clean Install");
            let progress = piter.next().unwrap();
            //TODO: Download progress within the file
            progress.send(0f64, "Downloading");
            // Must download from scratch
            let url = format!("{}{}.tar.xz", BASEURL, bin);
            let dottarxz = client.get(&url).send().await
                .map_err(|_| ConnectionFailure)?.bytes().await
                .map_err(|_| ConnectionFailure)?;
            let dottar = read::XzDecoder::new(dottarxz.as_ref());
            
            let progress = piter.next().unwrap();
            progress.send(0f64, "Installing");
            let mut ar = Archive::new(dottar);
            for file in ar.entries().unwrap() {
                let mut f = file.map_err(|_| BadResponse)?;
                let outpath = of_path().join(f.path().map_err(|_| BadResponse)?);
                std::fs::create_dir_all(outpath.parent().unwrap()).map_err(|_| BadResponse)?;
                f.unpack_in(of_path()).map_err(|_| WriteErr)?;
                inst.bins.get_mut(bin).unwrap().files.push(outpath);

                //TODO: Signature recording
            }

            inst.bins.get_mut(bin).unwrap().version = bindex.version;
            inst.save_changes().map_err(|_| WriteErr)?;
        }
    }
    progress.finish();
    Ok(())
}
