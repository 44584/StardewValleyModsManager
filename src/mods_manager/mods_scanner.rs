use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ManifestInfo {
    pub Name: String,
    pub Version: String,
    pub Description: String,
}

struct ModScanner {
    mods_folder_path: PathBuf,
}

impl ModScanner {
    ///使用默认的模组目录
    pub fn default() -> Self {
        ModScanner {
            mods_folder_path: PathBuf::from(
                "C:/Program Files (x86)/Steam/steamapps/common/Stardew Valley/Mods",
            ),
        }
    }

    ///从单个模组的manifest.json文件中获取目标信息
    /// mod_name是单个模组文件夹名, 在本函数中拼接
    fn scan_single_mod(&self, mod_name: &str) -> Result<Option<ManifestInfo>, String> {
        let manifest_path = self
            .mods_folder_path
            .join(format!("{}/manifest.json", mod_name));

        //如果不存在, 就不是星露谷模组
        if !manifest_path.exists() {
            return Err(String::from("Mod does not exist"));
        }

        //读取并解析mainfest文件
        let manifest_content = fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;

        let manifest: ManifestInfo = serde_json::from_str(&manifest_content).unwrap();

        Ok(Some(manifest))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_parse() {
        let modScanner = ModScanner::default();
        let mainfest = modScanner.scan_single_mod("GoBackHome").unwrap().unwrap();
        assert_eq!(mainfest.Name, "GoBackHome");
        assert_eq!(mainfest.Version, "1.0.0");
        assert_eq!(
            mainfest.Description,
            "After the player presses Q, he/she will go back home immediately."
        );
    }
}
