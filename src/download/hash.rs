use std::io::Result;
use std::fs::read;
use std::path;

use sha2::{Digest, Sha512};
use hex::encode;

pub fn hash_file(path: path::PathBuf) -> Result<String> {
    let file = read(path)?;

    let mut sha = Sha512::new();
    sha.update(file);
    let hash = sha.finalize();
    Ok(encode(hash))
}