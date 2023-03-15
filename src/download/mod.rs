use std::{
    path::PathBuf,
    str::FromStr,
};

use crate::modrinth::Client;

use crate::hash;

pub struct Downloader {
    mod_path: String,
    version: String,
    loader: String,
    client: Client,
}

impl Downloader {
    pub fn new(mut mod_path: String, version: String, loader: String) -> Self {
        let client = Client::new();

        if !mod_path.ends_with('/') {
            tracing::warn!("mod_path doesn't include trailing slash. Correcting");
            mod_path += "/";
        }

        Self {
            mod_path,
            version,
            loader,
            client,
        }
    }

    pub async fn download(&self, mod_id: String) {
        let version = self
            .client
            .get_newest_version(&mod_id, &self.version, &self.loader)
            .await
            .expect("Couldn't get version");

        let file = version
            .files
            .iter()
            .find(|x| x.primary)
            .expect("Newest version has no files!");

        let final_name = mod_id.to_string() + ".jar";

        if self.should_download(&version.name, &final_name, &file.hashes.sha512) {
            let download_path = self.mod_path.to_string() + &final_name;
            log::info!("Downloading {} to {}", file.filename, download_path);

            self.client
                .download_file(file.clone(), &download_path)
                .await
                .expect("Couldn't download file");
        }
    }

    // Deletes file if it should download!
    fn should_download(&self, filename: &str, mod_hash: &str, mod_name: &str) -> bool {
        let fpath = PathBuf::from_str(&(self.mod_path.to_string() + filename)).unwrap();
        log::debug!("Testing if should download {}", fpath.to_string_lossy());
        if fpath.is_file() {
            let h = hash::hash_file(fpath.as_path()).unwrap();
            if h != mod_hash {
                std::fs::remove_file(fpath.as_path()).unwrap();
                return true;
            } else {
                tracing::info!("Skipping {}, newest version already downloaded.", mod_name);
                return false;
            }
        } else {
            true
        }
    }
}
