use std::fs::File;
use std::io::{copy, Result};
use std::path::Path;
use std::time::Instant;

use hex::encode;
use sha2::{Digest, Sha512};

use tokio::sync::oneshot;

pub fn hash_file(path: impl AsRef<Path>) -> Result<String> {
    let start = Instant::now();
    let mut file = File::open(path)?;

    let mut sha = Sha512::new();
    copy(&mut file, &mut sha)?;
    let hash = sha.finalize();

    tracing::debug!(time_millis = start.elapsed().as_millis(), "Hashed data!");
    Ok(encode(hash))
}

pub async fn async_hash_file(path: impl AsRef<Path>) -> Result<String> {
    let owned = path.as_ref().to_owned();
    let (tx, rx) = oneshot::channel();

    rayon::spawn(|| {
        tx.send(hash_file(owned)).expect("Couldn't send back async hash!");
    });

    rx.await.expect("Couldn't get async hash!")
}