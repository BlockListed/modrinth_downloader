use std::io::{Result, Read};
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
    // Creating a fixed sized buffer to reduce memory size.
    let mut buf: Vec<u8> = vec![0u8; 2usize.pow(2*10)];
    loop {
        let bytes_read = file.read(&mut buf)?;
        if bytes_read == 0 {
            log::trace!("Fully read file, {}!", path.display());
            break
        }
        log::trace!("Read {} bytes from file, {}¡", bytes_read, path.display());
        sha.update(&buf[0..bytes_read]);
    }
    let hash = sha.finalize();

    log::debug!("Took {} millis to hash!", start.elapsed().as_millis());
    Ok(encode(hash))
}