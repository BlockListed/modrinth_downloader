use std::{env::var_os, path::PathBuf};

use serde::Deserialize;
use tokio::fs::read;

const DEFAULT_CONFIG_PATH: &str = "/config/config.toml";

#[derive(Deserialize)]
pub struct Configuration {
    pub mod_path: PathBuf,
    pub version: String,
    pub loader: String,
    pub mod_ids: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Couldn't read configuration from `{path}` because `{error}`. Path can be changed using `CONFIG_PATH` env.")]
    IOError {
        error: std::io::Error,
        path: String,
    },
    #[error("Couldn't parse configuration at `{path}` because `{error}`.")]
    TOMLError {
        error: toml::de::Error,
        path: String,
    },
}

impl From<(std::io::Error, String)> for ConfigError {
    fn from(value: (std::io::Error, String)) -> Self {
        Self::IOError { error: value.0, path: value.1 }
    }
}

impl From<(toml::de::Error, String)> for ConfigError {
    fn from(value: (toml::de::Error, String)) -> Self {
        Self::TOMLError { error: value.0, path: value.1 }
    }
}

macro_rules! handle_error {
    ($e:expr, $p:expr) => {
        match $e {
            Ok(x) => x,
            Err(e) => return Err(ConfigError::from((e, $p.to_string_lossy().into_owned())))
        }
    };
}

pub async fn get_config() ->Result<Configuration, ConfigError> {
    let config_path: PathBuf = var_os("CONFIG_PATH").unwrap_or_else(|| {
        tracing::debug!("Using default CONFIG_PATH, because enviroment is not set.");
        DEFAULT_CONFIG_PATH.to_string().into()
    }).into();
    tracing::debug!(path=%config_path.display(), "Getting configration!");
    let file = handle_error!(read(&config_path).await, config_path);
    Ok(handle_error!(toml::from_slice(file.as_slice()), config_path))
}
