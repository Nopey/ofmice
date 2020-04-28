use crate::platform;
use reqwest::Client;

/// Does everything the game needs to
pub async fn launch(){
    // 1. Get index and index.asc
    // 2. Check signature index.asc against index
    // 3. Parse index
    // 4. For each bin that's gonna be installed
    for bin in platform::bins(){
        // 4.1 If the bin is installed
        // check what revision it is at
        // If the revision is up to date, do nothing
        // If the revision is patchable, 4A
        // 4A.1 for each patch from installed revision to latest
        //TODO:
        // else, 4B
        // 4B.1 Find the bin url from the index
        // 4B.2 Download the bin
        // 4B.3 Check the bin against the sig contained in the index
        // 4B.4 Call the code that applies the bin
    }
    unimplemented!()
}
