use std::io::Write;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use chrono::Local;
use color_eyre::Report;
use color_eyre::Result;

use dashmap::DashMap;
use serde::Deserialize;
use ureq::AgentBuilder;
use url::Url;
use ureq::Agent;
use ureq::Response;

#[derive(Deserialize, Clone, Debug)]
pub struct Version {
    pub name: String,
    pub files: Vec<ModFile>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ModFile {
    pub hashes: Hashes,
    pub url: String,
    pub filename: String,
    pub primary: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Hashes {
    pub sha512: String,
}

#[derive(Deserialize, Debug)]
pub struct ProjectInformation {
    pub title: String,
}

pub struct Client {
    client: Agent,
    endpoint: Url,
    title_cache: DashMap<String, String>,
}

impl Client {
    pub fn new() -> Self {
        let client = AgentBuilder::new()
            .user_agent(&format!(
                "BlockListed/modrinth_downloader/{} (idvg4u3ea@mozmail.com)",
                env!("CARGO_PKG_VERSION")
            ))
            .build();

        Self {
            client,
            endpoint: Url::parse("https://api.modrinth.com/v2").unwrap(),
            title_cache: DashMap::new(),
        }
    }

    pub fn get_title(&self, mod_id_or_slug: &str) -> Result<String> {
        if let Some(title) = self.title_cache.get(mod_id_or_slug) {
            return Ok(title.value().to_string());
        }
        let mut url = self.endpoint.clone();

        {
            let mut segments = url.path_segments_mut().unwrap();
            segments.push("project");
            segments.push(mod_id_or_slug);
        }

        tracing::debug!(%url, "Getting title information!");
        let resp: ProjectInformation = self.client.get(url.as_str()).call()?.into_json()?;

        self.title_cache
            .insert(mod_id_or_slug.to_string(), resp.title.clone());

        Ok(resp.title)
    }

    pub fn get_versions(
        &self,
        mod_id_or_slug: &str,
        game_version: &str,
        loader: &str,
    ) -> Result<Vec<Version>> {
        let mut url = self.endpoint.clone();

        {
            let mut segments = url.path_segments_mut().unwrap();
            segments.push("project");
            segments.push(mod_id_or_slug);
            segments.push("version");
        }
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("loaders", &format!("[\"{loader}\"]"));
            query.append_pair("game_versions", &format!("[\"{game_version}\"]"));
        }

        tracing::debug!(%url, "Getting version information");

        let resp = self.client.get(url.as_str()).call()?;

        let versions: Vec<Version> = resp.into_json()?;

        Ok(versions)
    }

    pub fn get_newest_version(
        &self,
        mod_id_or_slug: &str,
        game_version: &str,
        loader: &str,
    ) -> Result<Version> {
        let title = self.get_title(mod_id_or_slug)?;
        Ok(self
            .get_versions(mod_id_or_slug, game_version, loader)?
            .get(0)
            .ok_or_else(|| color_eyre::eyre::eyre!("No version of {title} exists for {game_version}-{loader}"))?
            .clone())
    }

    pub fn download_file(
        &self,
        file: ModFile,
        destination: impl AsRef<Path> + Send,
    ) -> Result<()> {
        let path = destination.as_ref();
        tracing::debug!(downloading = file.url);
        let resp = self.client.get(&file.url).call()?;

        let status = resp.status();

        if status != 200 {
            let timestamp = Local::now();

            let mut log_file = PathBuf::from_str("/minecraft/mods/").unwrap();
            log_file.push(format!(
                "download_{}_{}.log",
                file.filename,
                timestamp.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
            ));

            tracing::error!(code=%status , log=%log_file.display(), "Download failed! Saving log file.");

            let status = resp.status();

            let mut out = File::create(log_file)?;
            save_resp(resp, &mut out)?;

            return Err(Report::msg(format!(
                "Download failed with code - {}",
                status
            )));
        }

        let mut out = File::create(path).map_err(|x| {
            std::io::Error::new(
                x.kind(),
                format!("Couldn't create file {}.", path.display()),
            )
        })?;

        save_resp(resp, &mut out)?;

        // This is supposed to be read from disk to detect corruption.
        // DO NOT OPTIMISE THIS AS A READ FROM MEMORY, SINCE THAT'S FUCKING STUPID.
        if crate::hash::hash_file(path)? == file.hashes.sha512 {
            tracing::debug!(dest = ?path, file.hashes.sha512, "Correct shasum for downloaded file!");
        } else {
            panic!(
                "CORRUPTION WHILE CHECKING DOWNLOADED FILE! {} - {}",
                file.filename,
                status
            );
        }

        Ok(())
    }
}

pub fn save_resp(
    resp: Response,
    out: &mut impl Write,
) -> Result<()> {
    // Chunk to reduce memory usage
    std::io::copy(&mut resp.into_reader(), out)?;

    Ok(())
}
