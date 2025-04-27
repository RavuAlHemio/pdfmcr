use std::path::PathBuf;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::error;


pub(crate) static CONFIG_PATH: OnceLock<PathBuf> = OnceLock::new();
pub(crate) static CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();


#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Config {
    pub state_file_path: String,
    pub image_dir: String,
}


pub(crate) fn load_config() -> Option<Config> {
    let config_path = CONFIG_PATH.get()
        .expect("CONFIG_PATH not set?!");
    let config_string = match std::fs::read_to_string(config_path) {
        Ok(cs) => cs,
        Err(e) => {
            error!("failed to read config from {}: {}", config_path.display(), e);
            return None;
        }
    };
    let config: Config = match toml::from_str(&config_string) {
        Ok(c) => c,
        Err(e) => {
            error!("failed to parse config from {}: {}", config_path.display(), e);
            return None;
        },
    };
    Some(config)
}
