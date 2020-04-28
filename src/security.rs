//!
//! Uses PGP signatures to ensure that the software being
//! downloaded from the internet is actually
//! coming from us, and not some random hijacker.
//!

use pgp::{Deserializable, StandaloneSignature, types::KeyTrait};
use std::io::Cursor;

const OFMICE_PUBKEY_ASC: &'static [u8] = include_bytes!("ofmice.pgp.ascii");

// Loosely based off of
// https://github.com/rust-lang/rustup/pull/2077/commits/597953ef16f77213884850f21fed297b64c42a80
pub fn verify_signature(
    content_buf: &[u8],
    signature: &str,
) -> bool {
    if let Ok((signature, _)) = StandaloneSignature::from_string(signature){
        if let Ok((key, _)) = pgp::SignedPublicKey::from_armor_single(Cursor::new(OFMICE_PUBKEY_ASC)){
            if key.is_signing_key() && signature.verify(&key, &content_buf).is_ok() {
                return true;
            }
        }
    }
    false
}