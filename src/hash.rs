use std::fs::File;
use std::io::{copy, Result};
use std::path::Path;
use std::time::Instant;

use hex::encode;
use sha2::{Digest, Sha512};
use tracing_unwrap::ResultExt;

pub fn hash_file(path: impl AsRef<Path> + std::marker::Copy) -> Result<String> {
    let start = Instant::now();
    let mut file = File::open(path).unwrap_or_log();

    let mut sha = Sha512::new();
    copy(&mut file, &mut sha)?;
    let hash = sha.finalize();

    log::debug!("Took {} millis to hash!", start.elapsed().as_millis());
    Ok(encode(hash))
}
