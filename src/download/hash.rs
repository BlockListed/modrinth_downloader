use std::io::Result;
use std::fs::read;
use std::path;

use sha2::{Digest, Sha512};
use hex::encode;

pub fn hash_file(path: &path::Path) -> Result<String> {
    let file = match read(path) {
        Ok(x) => x,
        Err(x) => {
            log::error!("Couldn't read path: {}, because {}", path.to_string_lossy(), x);
            return Err(x)
        }
    };

    let mut sha = Sha512::new();
    sha.update(file);
    let hash = sha.finalize();
    Ok(encode(hash))
}