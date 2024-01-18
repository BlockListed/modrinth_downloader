use std::path::{Path, PathBuf};

use color_eyre::eyre::WrapErr;

use crate::modrinth::Client;

use crate::hash;

pub struct Downloader {
    mod_path: PathBuf,
    version: String,
    loader: String,
    client: Client,
}

impl Downloader {
    pub fn new(mod_path: PathBuf, version: String, loader: String) -> Self {
        let client = Client::new();

        if !mod_path.exists() {
            std::fs::create_dir(&mod_path).wrap_err("Couldn't create mod folder").unwrap();
            tracing::info!("Mod path didn't exist created directory!");
        }

        assert!(
            mod_path.is_dir(),
            "mod_path, {}, is not a directory",
            mod_path.display()
        );

        Self {
            mod_path,
            version,
            loader,
            client,
        }
    }

    pub fn download(&self, mod_id: String) {
        let title = match self.client.get_title(&mod_id) {
            Ok(x) => x,
            Err(error) => {
                tracing::error!(%error, mod_id, "Couldn't get title of mod!");
                return;
            }
        };

        let version = match self
            .client
            .get_newest_version(&mod_id, &self.version, &self.loader)
        {
            Ok(x) => x,
            Err(error) => {
                tracing::error!(%error, "Couldn't get mod version!");
                return;
            }
        };

        // TODO: Should probably be inside modrinth client
        let file = match version.files.iter().find(|x| x.primary) {
            Some(x) => x,
            // This is desired behaviour, as described in https://github.com/modrinth/labrinth/issues/559
            None => &version.files[0],
        };

        let final_name = mod_id.to_string() + ".jar";
        let mut final_path = self.mod_path.clone();
        final_path.push(final_name);

        let should_download = match self
            .should_download(&final_path, &file.hashes.sha512, &title)
        {
            Ok(x) => x,
            Err(error) => {
                tracing::error!(%error, file=%final_path.display(), "Couldn't perform update checking/deletion of old file!");
                return;
            }
        };

        if should_download {
            tracing::info!(file=file.filename, path=%final_path.display(), "Downloading mod");

            if let Err(e) = self.client.download_file(file.clone(), &final_path) {
                tracing::error!(%e, "Couldn't download file!");
            }
        }
    }

    // Deletes file if it should download!
    fn should_download(
        &self,
        filepath: impl AsRef<Path> + Send,
        mod_hash: &str,
        mod_title: &str,
    ) -> std::io::Result<bool> {
        let fpath = filepath.as_ref();
        tracing::debug!("Testing if should download {}", fpath.to_string_lossy());
        if fpath.is_file() {
            let h = hash::hash_file(fpath).unwrap();
            if h == mod_hash {
                tracing::info!("Skipping {mod_title}, newest version already downloaded.");
                tracing::debug!(filepath=%fpath.display(), "skipped");
                Ok(false)
            } else {
                tracing::debug!(hash_new = h, hash_old = mod_hash);
                std::fs::remove_file(fpath)?;
                tracing::info!("Updating {mod_title}.");
                Ok(true)
            }
        } else {
            // TODO: This log should probably be in download()
            tracing::debug!(mod_title, filepath=%fpath.display(), "Mod not found. Downloading now!");
            Ok(true)
        }
    }
}
