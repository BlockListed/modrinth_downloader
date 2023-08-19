use std::fs::File;
use std::io::{copy, Result};
use std::path::Path;
use std::time::Instant;

use hex::encode;
use sha2::{Digest, Sha512};

#[allow(clippy::module_name_repetitions)]
pub fn hash_file(path: impl AsRef<Path> + Send) -> Result<String> {
    let start = Instant::now();
    let mut file = File::open(path)?;

    let mut sha = Sha512::new();
    copy(&mut file, &mut sha)?;
    let hash = sha.finalize();

    tracing::debug!(time_millis = start.elapsed().as_millis(), "Hashed data!");
    Ok(encode(hash))
}
