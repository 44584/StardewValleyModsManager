use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub mods_folder_path: String,
}

impl AppConfig {
    pub fn load_from_file(path: &PathBuf) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        toml::from_str(&content).ok()
    }

    pub fn save_to_file(&self, path: &PathBuf) -> std::io::Result<()> {
        let toml_str =
            toml::to_string(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, toml_str)
    }
}
