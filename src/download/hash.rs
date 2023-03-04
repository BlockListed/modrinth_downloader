use std::io::{Result, copy};
use std::fs::File;
use std::time::Instant;
use std::path;

use sha2::{Digest, Sha512};
use hex::encode;

pub fn hash_file(path: &path::Path) -> Result<String> {
    let start = Instant::now();
    let mut file = match File::open(path) {
        Ok(x) => x,
        Err(x) => {
            log::error!("Couldn't open path: {}, because {}", path.to_string_lossy(), x);
            return Err(x)
        }
    };

    let mut sha = Sha512::new();
    copy(&mut file, &mut sha)?;
    let hash = sha.finalize();

    log::debug!("Took {} millis to hash!", start.elapsed().as_millis());
    Ok(encode(hash))
}