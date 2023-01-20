use std::{fs::File, str::FromStr, path::{Path, PathBuf}};

use ureq::{AgentBuilder, Agent, Error};
use serde::Deserialize;

mod hash;

#[derive(Deserialize, Clone)]
struct ModRinthVersion<'a> {
    name: &'a str,
    files: Vec<ModRinthFile<'a>>,
}

#[derive(Deserialize, Clone)]
struct ModRinthFile<'a> {
    hashes: ModRinthHashes<'a>,
    url: &'a str,
    filename: &'a str,
    primary: bool,
}

#[derive(Deserialize, Clone)]
struct ModRinthHashes<'a> {
    sha512: &'a str,
}

pub struct Downloader<'a> {
    mod_path: String,
    version: &'a str,
    loader: &'a str,
    agent: Agent,
}

impl<'a> Downloader<'a> {
    pub fn new(mut mod_path: String, version: &'a str, loader: &'a str) -> Self {
        let agent = AgentBuilder::new()
            .user_agent(format!("github.com/BlockListed/modrinth_downloader/{}", env!("CARGO_PKG_VERSION")).as_str())
            .build();

        if !mod_path.ends_with('/') {
            log::warn!("mod_path doesn't include trailing slash. Correcting");
            mod_path += "/";
        }

        Self {
            mod_path,
            version,
            loader,
            agent,
        }
    }

    pub fn download(&self, mod_id: &str) -> Result<(), Box<Error>> {
        let versions = self.agent.get(format!("https://api.modrinth.com/v2/project/{mod_id}/version?game_versions=[\"{}\"]&loaders=[\"{}\"]", self.version, self.loader).as_str()).call()?.into_string().unwrap();
    
        let data: Vec<ModRinthVersion> = serde_json::from_str(&versions).unwrap();
        let version = data[0].clone();
        let file = version.files.iter().find(|x| x.primary).unwrap().clone();

        if self.should_download(&data[0].name, &(mod_id.to_string() + ".jar"), &file.hashes.sha512) {
            let mut mod_download = self.agent.get(&file.url).call()?.into_reader();
            let download_path = self.mod_path.to_string() + &mod_id + ".jar";
            log::info!("Downloading {} to {}", file.filename, download_path);
            let mut download_file = File::create(&download_path).unwrap();

            std::io::copy(&mut mod_download, &mut download_file).unwrap();

            if hash::hash_file(Path::new(&download_path)).unwrap() != file.hashes.sha512 {
                panic!("HASH NOT THE SAME FOR {}", download_path);
            }
        }
    
        Ok(())
    }

    // Delete file if should download!
    fn should_download(&self, mod_name: &str, filename: &str, mod_hash: &str) -> bool {
            let fpath = PathBuf::from_str(&(self.mod_path.clone() + filename)).unwrap();
            log::debug!("Testing if should download {}", fpath.to_string_lossy());
            if fpath.is_file() {
                let h = hash::hash_file(fpath.as_path()).unwrap();
                if h != mod_hash {
                    std::fs::remove_file(fpath.as_path()).unwrap();
                    return true
                } else {
                    log::info!("Skipping {}, newest version already downloaded.", mod_name);
                    return false
                }
            } else {
                true
            }
    }
}



