use color_eyre::Result;
use reqwest::Client as ReqwestClient;
use serde::Deserialize;
use std::path::Path;
use dashmap::DashMap;

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
            endpoint: "https://api.modrinth.com/v2",
            title_cache: DashMap::new(),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_title(&self, mod_id_or_slug: &str) -> Result<String> {
        if let Some(title) = self.title_cache.get(mod_id_or_slug) {
            Ok(title.value().to_string())
        } else {
            let uri = self.endpoint.to_string() + &format!("/project/{mod_id_or_slug}");
            tracing::debug!(uri, "Getting title information!");
            let resp: ProjectInformation = self.client.get(uri).send().await?.json().await?;
            self.title_cache.insert(mod_id_or_slug.to_string(), resp.title.clone());
            Ok(resp.title)
        }
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
        let title = self.get_title(mod_id_or_slug).await?;
        Ok(self
            .get_versions(mod_id_or_slug, game_version, loader)
            .await?.get(0).ok_or(color_eyre::eyre::Error::msg(format!("No version of {title} exists for {game_version}-{loader}")))?
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

        let mut out = tokio::fs::File::create(path).await.map_err(|x| {
            std::io::Error::new(x.kind(), format!("Couldn't create file {}.", path.display()))
        })?;

        copy(&mut Cursor::new(resp.bytes().await?), &mut out).await?;

        // This is supposed to be read from disk to detect corruption.
        // DO NOT OPTIMISE THIS AS A READ FROM MEMORY, SINCE THAT'S FUCKING STUPID.
        // However this could be turned into an optional step, since https should
        // protect the data during download and if a filesystem corrupts data while
        // downloading, that should probably not be my problem.
        if crate::hash::async_hash_file(path).await? != file.hashes.sha512 {
            panic!("CORRUPTION WHILE DOWNLOADING FILE! {}", file.filename);
        } else {
            tracing::debug!(dest = ?path, file.hashes.sha512, "Correct shasum for downloaded file!");
        }

        Ok(())
    }
}
