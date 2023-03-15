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
        let title = self.client.get_title(&mod_id).await.expect("Couldn't find mod!");

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

        if self.should_download(&final_name, &file.hashes.sha512, &title).await {
            let download_path = self.mod_path.to_string() + &final_name;
            log::info!("Downloading {} to {}", file.filename, download_path);

            self.client
                .download_file(file.clone(), &download_path)
                .await
                .expect("Couldn't download file");
        }
    }

    // Deletes file if it should download!
    async fn should_download(&self, filename: &str, mod_hash: &str, mod_title: &str) -> bool {
        let fpath = PathBuf::from_str(&(self.mod_path.to_string() + filename)).unwrap();
        log::debug!("Testing if should download {}", fpath.to_string_lossy());
        if fpath.is_file() {
            let h = hash::async_hash_file(fpath.as_path()).await.unwrap();
            if h != mod_hash {
                tracing::debug!(hash_new = h, hash_old = mod_hash);
                std::fs::remove_file(fpath.as_path()).unwrap();
                true
            } else {
                tracing::info!("Skipping {mod_title}, newest version already downloaded.");
                tracing::debug!(filename, "skipped");
                false
            }
        } else {
            tracing::debug!("Mod {mod_title} not found at {filename}. Downloading now!");
            true
        }
    }
}
