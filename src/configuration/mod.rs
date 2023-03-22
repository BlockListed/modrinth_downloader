use std::env::{var, VarError};

use serde::Deserialize;
use tokio::fs::read;

const DEFAULT_CONFIG_PATH: &str = "/config/config.toml";

#[derive(Deserialize)]
pub struct Configuration {
    pub mod_path: String,
    pub version: String,
    pub loader: String,
    pub mod_ids: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigurationError {
    #[error("Couldn't read configuration from `{path}` because `{error}`. Path can be changed using `CONFIG_PATH` env.")]
    IOError {
        error: std::io::Error,
        path: String,
    },
    #[error("Couldn't parse configuration at `{path}` because `{error}`")]
    TOMLError {
        error: toml::de::Error,
        path: String,
    },
}

impl From<(std::io::Error, String)> for ConfigurationError {
    fn from(value: (std::io::Error, String)) -> Self {
        Self::IOError { error: value.0, path: value.1 }
    }
}

impl From<(toml::de::Error, String)> for ConfigurationError {
    fn from(value: (toml::de::Error, String)) -> Self {
        Self::TOMLError { error: value.0, path: value.1 }
    }
}

macro_rules! handle_error {
    ($e:expr, $p:expr) => {
        match $e {
            Ok(x) => x,
            Err(e) => return Err(ConfigurationError::from((e, $p)))
        }
    };
}

pub async fn get_config() ->Result<Configuration, ConfigurationError> {
    let config_path = var("CONFIG_PATH").unwrap_or_else(|err| {
        match err {
            VarError::NotUnicode(x) => {
                tracing::warn!(invalid_str=?x, "CONFIG_PATH is not valid Unicode!");
            },
            _ => {
                tracing::debug!("Using default CONFIG_PATH, because enviroment is not set.");
            }
        }
        DEFAULT_CONFIG_PATH.to_string()
    });
    tracing::debug!(path=config_path, "Getting configration!");
    let file = handle_error!(read(&config_path).await, config_path);
    Ok(handle_error!(toml::from_slice(file.as_slice()), config_path))
}
