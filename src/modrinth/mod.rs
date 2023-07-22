use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use chrono::Local;
use color_eyre::Result;
use color_eyre::Report;

use reqwest::Client as ReqwestClient;
use reqwest::Response;
use tokio::fs::File;
use serde::Deserialize;
use dashmap::DashMap;
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;
use url::Url;

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
    client: ReqwestClient,
    endpoint: Url,
    title_cache: DashMap<String, String>,
}

impl Client {
    pub fn new() -> Self {
        let client = ReqwestClient::builder()
            .user_agent(format!(
                "BlockListed/modrinth_downloader/{} (idvg4u3ea@mozmail.com)",
                env!("CARGO_PKG_VERSION")
            ))
            .build()
            .expect("Couldn't create client!");
        Self {
            client,
            endpoint: Url::parse("https://api.modrinth.com/v2").unwrap(),
            title_cache: DashMap::new(),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_title(&self, mod_id_or_slug: &str) -> Result<String> {
        if let Some(title) = self.title_cache.get(mod_id_or_slug) {
            return Ok(title.value().to_string())
        }
        let mut url = self.endpoint.clone();

        {
            let mut segments = url.path_segments_mut().unwrap();
            segments.push("project");
            segments.push(mod_id_or_slug);
        }

        tracing::debug!(%url, "Getting title information!");
        let resp: ProjectInformation = self.client.get(url).send().await?.json().await?;
        self.title_cache.insert(mod_id_or_slug.to_string(), resp.title.clone());
        Ok(resp.title)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_versions(
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

        let resp = self.client.get(url.as_str()).send().await?;

        tracing::debug!(%url, "Getting version information");

        let versions: Vec<Version> = resp.json().await?;

        Ok(versions)
    }

    pub async fn get_newest_version(
        &self,
        mod_id_or_slug: &str,
        game_version: &str,
        loader: &str,
    ) -> Result<Version> {
        let title = self.get_title(mod_id_or_slug).await?;
        Ok(self
            .get_versions(mod_id_or_slug, game_version, loader)
            .await?.get(0).ok_or_else(|| color_eyre::eyre::Error::msg(format!("No version of {title} exists for {game_version}-{loader}")))?
            .clone())
    }

    pub async fn download_file(
        &self,
        file: ModFile,
        destination: impl AsRef<Path> + Send,
    ) -> Result<()> {
        let path = destination.as_ref();
        tracing::debug!(downloading = file.url);
        let mut resp = self.client.get(file.url).send().await?;

        if !resp.status().is_success() {
            let timestamp = Local::now();

            let mut log_file = PathBuf::from_str("/minecraft/mods/").unwrap();
            log_file.push(format!("download_{}_{}.log", file.filename, timestamp.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)));

            tracing::error!(code=%resp.status() , log=%log_file.display(), "Download failed! Saving log file.");

            let mut out = File::create(log_file).await?;
            save_resp(&mut resp, &mut out).await?;

            return Err(Report::msg(format!("Download failed with code - {}", resp.status())))
        }

        let mut out = File::create(path).await.map_err(|x| {
            std::io::Error::new(x.kind(), format!("Couldn't create file {}.", path.display()))
        })?;

        save_resp(&mut resp, &mut out).await?;

        // This is supposed to be read from disk to detect corruption.
        // DO NOT OPTIMISE THIS AS A READ FROM MEMORY, SINCE THAT'S FUCKING STUPID.
        if crate::hash::async_hash_file(path).await? == file.hashes.sha512 {
            tracing::debug!(dest = ?path, file.hashes.sha512, "Correct shasum for downloaded file!");
        } else {
            panic!("CORRUPTION WHILE CHECKING DOWNLOADED FILE! {} - {}", file.filename, resp.status());
        }

        Ok(())
    }
}

pub async fn save_resp(resp: &mut Response, out: &mut (impl AsyncWrite + std::marker::Unpin + Send)) -> Result<()> {
    // Chunk to reduce memory usage
    while let Some(data) = resp.chunk().await? {
        out.write_all(&data).await?;
    }

    Ok(())
}