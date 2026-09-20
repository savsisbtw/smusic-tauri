use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::state::Track;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub library_folders: Vec<String>,
    pub tracks: Vec<Track>,
    pub volume: f32,
    pub last_track_id: Option<String>,
    pub discord_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            library_folders: Vec::new(),
            tracks: Vec::new(),
            volume: 1.0,
            last_track_id: None,
            discord_enabled: true,
        }
    }
}

fn config_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("config.json")
}

pub fn load(app_data_dir: &Path) -> Config {
    let path = config_path(app_data_dir);
    match fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

pub fn save(app_data_dir: &Path, config: &Config) {
    let path = config_path(app_data_dir);
    if let Ok(text) = serde_json::to_string_pretty(config) {
        let _ = fs::write(path, text);
    }
}
