mod mods_info_storage;
mod mods_scanner;

use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct ManifestInfo {
    pub Name: String,
    pub Version: String,
    pub Description: String,
    pub UniqueId: String,
}

pub struct ModInfo {
    pub manifest_info: ManifestInfo,
    pub path: PathBuf,
}

pub struct Profile {
    pub name: String,
    pub description: String,
    pub create_time: String,
}
