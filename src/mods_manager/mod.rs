pub mod mods_info_storage;
pub mod mods_scanner;

use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct ManifestInfo {
    pub Name: String,
    pub Version: String,
    pub Description: String,
    #[serde(alias = "UniqueID")] // 支持 UniqueID 和 UniqueId 两种字段名, 支持不同的manifest文件
    pub UniqueId: String,
}

#[derive(Clone)]
pub struct ModInfo {
    pub manifest_info: ManifestInfo,
    pub path: PathBuf,
}

//Todo: 添加路径属性
pub struct Profile {
    pub name: String,
    pub description: String,
    pub create_time: String,
}
