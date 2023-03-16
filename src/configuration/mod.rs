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

pub async fn get_config() -> Configuration {
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
    let file = read(config_path).await.expect("Couldn't open /config/config.toml");
    toml::from_slice(file.as_slice()).expect("TOML configuration INVALID!")
}
