use anyhow::Result;
use reqwest::Client as ReqwestClient;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Clone)]
pub struct ModrinthVersion {
    pub name: String,
    pub files: Vec<ModrinthFile>,
}

#[derive(Deserialize, Clone)]
pub struct ModrinthFile {
    pub hashes: ModrinthHashes,
    pub url: String,
    pub filename: String,
    pub primary: bool,
}

#[derive(Deserialize, Clone)]
pub struct ModrinthHashes {
    pub sha512: String,
}

#[derive(Deserialize)]
pub struct ProjectInformation {
    pub title: String,
}

pub struct Client {
    client: ReqwestClient,
    endpoint: &'static str,
}

impl Client {
    pub fn new() -> Self {
        let client = ReqwestClient::builder()
            .user_agent(format!(
                "BlockListed/modrinth_downloader/{} (idvg4u3ea@mozmail.com)",
                env!("CARGO_PKG_VERSION")
            ))
            .build()
            .unwrap();
        Self {
            client,
            endpoint: "https://api.modrinth.com/v2",
        }
    }

    pub async fn get_title(&self, mod_id_or_slug: &str) -> Result<String> {
        let uri = self.endpoint.to_string() + &format!("/project/{mod_id_or_slug}");
        let resp: ProjectInformation = self.client.get(uri).send().await?.json().await?;
        Ok(resp.title)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_versions(
        &self,
        mod_id_or_slug: &str,
        game_version: &str,
        loader: &str,
    ) -> Result<Vec<ModrinthVersion>> {
        let uri = self.endpoint.to_string() + &format!("/project/{mod_id_or_slug}/version?loaders=[\"{loader}\"]&game_versions=[\"{game_version}\"]");
        let resp = self.client.get(&uri).send().await?;

        tracing::debug!(uri, "Getting version information");

        let versions: Vec<ModrinthVersion> = resp.json().await?;

        Ok(versions)
    }

    pub async fn get_newest_version(
        &self,
        mod_id_or_slug: &str,
        game_version: &str,
        loader: &str,
    ) -> Result<ModrinthVersion> {
        Ok(self
            .get_versions(mod_id_or_slug, game_version, loader)
            .await?.get(0).ok_or(anyhow::Error::msg(format!("No version of {mod_id_or_slug} exists for {game_version}-{loader}")))?
            .clone())
    }

    pub async fn download_file(
        &self,
        file: ModrinthFile,
        destination: impl AsRef<Path>,
    ) -> Result<()> {
        use tokio::io::copy;
        use std::io::Cursor;

        let path = destination.as_ref();
        tracing::debug!(downloading = file.url);
        let resp = self.client.get(file.url).send().await?;

        let mut out = tokio::fs::File::create(path).await?;

        copy(&mut Cursor::new(resp.bytes().await?), &mut out).await?;

        if crate::hash::async_hash_file(path).await? != file.hashes.sha512 {
            panic!("CORRUPTION WHILE DOWNLOADING FILE! {}", file.filename);
        } else {
            tracing::debug!(dest = ?path, file.hashes.sha512, "Correct shasum for downloaded file!");
        }

        Ok(())
    }
}
