use std::{env::var, path::PathBuf};

use config::Config;
use serde::Deserialize;

const DEFAULT_CONFIG_PATH: &str = "/config/config.toml";

fn default_mod_path() -> PathBuf {
    PathBuf::from("/minecraft/mods")
}

fn default_loader() -> String {
    "fabric".to_string()
}

#[derive(Deserialize)]
pub struct Configuration {
    #[serde(default = "default_mod_path")]
    pub mod_path: PathBuf,
    pub version: String,
    #[serde(default = "default_loader")]
    pub loader: String,
    pub mod_ids: Vec<String>,
}

pub fn get_config() -> Configuration {
    let config_path: PathBuf = var("CONFIG_PATH")
        .unwrap_or_else(|err| {
            tracing::debug!(%err, "Using default CONFIG_PATH");
            DEFAULT_CONFIG_PATH.to_string().into()
        })
        .into();
    tracing::debug!(path=%config_path.display(), "found configuration path");

    let config_builder = Config::builder()
        .add_source(config::File::new(config_path.to_str().unwrap(), config::FileFormat::Toml).required(true))
        .add_source(config::Environment::default());

    config_builder.build().unwrap().try_deserialize().unwrap()
}
