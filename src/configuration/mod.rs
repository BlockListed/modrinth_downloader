use serde::Deserialize;
use std::fs::read;

const CONFIG_PATH: &str = "/config/config.toml";

#[derive(Deserialize)]
pub struct Configuration {
    pub mod_path: String,
    pub version: String,
    pub loader: String,
    pub mod_ids: Vec<String>,
}

pub fn get_config() -> Configuration {
    let file = String::from_utf8(read(CONFIG_PATH).expect("Couldn't open /config/config.toml")).unwrap();
    toml::from_str(&file).unwrap()
}