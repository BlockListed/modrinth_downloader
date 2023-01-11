use std::{fs::File, str::FromStr};

use ureq::{AgentBuilder, Agent, Error};
use serde::Deserialize;

mod hash;

#[derive(Deserialize, Clone)]
struct ModRinthVersion {
    files: Vec<ModRinthFile>,
}

#[derive(Deserialize, Clone)]
struct ModRinthFile {
    hashes: ModRinthHashes,
    url: String,
    filename: String,
    primary: bool,
}

#[derive(Deserialize, Clone)]
struct ModRinthHashes {
    sha512: String,
}

pub struct Downloader {
    mod_path: String,
    version: String,
    loader: String,
    agent: Agent,
}

impl Downloader {
    pub fn new(mut mod_path: String, version: String, loader: String) -> Self {
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

    pub fn download(&self, mod_id: String) -> Result<(), Box<Error>> {
        let versions = self.agent.get(format!("https://api.modrinth.com/v2/project/{mod_id}/version?game_versions=[\"{}\"]&loaders=[\"{}\"]", self.version, self.loader).as_str()).call()?;
    
        let data: Vec<ModRinthVersion> = serde_json::from_reader(versions.into_reader()).unwrap();
        let version = data[0].clone();
        let file = version.files.iter().find(|x| x.primary).unwrap().clone();

        if self.should_download(file.filename.clone(), &file.hashes.sha512) {
            log::info!("Downloading {}", file.filename);
            let mut mod_download = self.agent.get(&file.url).call()?.into_reader();
            let download_path = self.mod_path.to_string() + file.filename.as_str();
            let mut download_file = File::create(&download_path).unwrap();

            std::io::copy(&mut mod_download, &mut download_file).unwrap();

            if hash::hash_file(std::path::PathBuf::from_str(&download_path).unwrap()).unwrap() != file.hashes.sha512 {
                panic!("HASH NOT THE SAME FOR {}", download_path);
            }
        }
    
        Ok(())
    }

    // Delete file if should download!
    fn should_download(&self, filename: String, mod_hash: &str) -> bool {
        // lithium-fabric-mc1.19.3-0.10.4.jar
        let name= filename.split('-').next().unwrap();

        for i in std::fs::read_dir(&self.mod_path).unwrap() {
            let d = i.unwrap();
            let useless =  d.file_name();
            let fname = useless.to_str().unwrap();
            if fname.starts_with(name) && fname.ends_with(".jar") {
                let h = hash::hash_file(d.path()).unwrap();
                if h != mod_hash {
                    std::fs::remove_file(d.path()).unwrap();
                    return true
                } else {
                    log::info!("Skipping {}, newest version already downloaded.", d.file_name().to_str().unwrap());
                    return false
                }
            }
        }

        true
    }
}



