use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use super::{ManifestInfo, ModInfo};

#[derive(Debug, Deserialize)]

pub struct ModScanner {
    mods_folder_path: PathBuf,
}

impl ModScanner {
    /// 允许动态设置mods目录
    pub fn set_mods_path(&mut self, mods_path: PathBuf) {
        self.mods_folder_path = mods_path;
    }
}

impl ModScanner {
    /// 使用默认的模组目录
    pub fn default() -> Self {
        ModScanner {
            mods_folder_path: PathBuf::from(
                "C:/Program Files (x86)/Steam/steamapps/common/Stardew Valley/Mods",
            ),
        }
    }

    /// 根据提供的星露谷模组路径构造
    ///
    /// # 参数
    /// - `sv_mods_path`: 包含所有模组的最低一级目录
    pub fn from(sv_mods_path: &str) -> Self {
        ModScanner {
            mods_folder_path: PathBuf::from(sv_mods_path),
        }
    }

    /// 返回 UniqueId 和 `ModInfo` 的哈希表
    /// 目前只处理第一层, 后续可以引入walkdir处理多层
    pub fn scan_mods(&self) -> HashMap<String, ModInfo> {
        let mut ans = HashMap::new();
        let entries = fs::read_dir(&self.mods_folder_path).expect("Can not read the folder.");
        for entry in entries {
            let entry = entry.expect("Can not read entry.");
            if entry.file_type().unwrap().is_file() {
                continue;
            }
            eprintln!("{:?}", entry);

            let mod_info = self.scan_single_mod(&entry.path());
            let mod_info = match mod_info {
                Ok(reslut) => reslut.unwrap(),
                Err(_) => {
                    continue;
                }
            };
            let unique_id = mod_info.manifest_info.UniqueId.clone();
            ans.insert(unique_id, mod_info);
        }

        ans
    }

    ///从单个模组的manifest.json文件中获取目标信息
    ///
    /// # 参数
    /// - `mod_folder_name`: 单个模组文件夹路径, 会在本函数中拼接manifest.json文件
    ///
    /// # 返回值
    /// Result<Option<ModsInfo>, String>, Option中Some是ModsInfo
    /// 如果该文件夹不是模组文件夹, 返回Err
    fn scan_single_mod(&self, mod_folder_path: &PathBuf) -> Result<Option<ModInfo>, String> {
        let manifest_path = mod_folder_path.join(format!("manifest.json"));

        //如果不存在, 就不是星露谷模组
        if !manifest_path.exists() {
            return Err(String::from("Mod does not exist"));
        }

        //读取并解析mainfest文件
        let manifest_content = fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;

        let manifest: ManifestInfo = match serde_json::from_str(&manifest_content) {
            Ok(m) => m,
            Err(e) => return Err(e.to_string()),
        };
        let mod_info = ModInfo {
            manifest_info: manifest,
            path: manifest_path.parent().unwrap().into(),
        };
        Ok(Some(mod_info))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_parse1() {
        let modScanner = ModScanner::default();
        let test_mod_path = modScanner.mods_folder_path.join("NPC不踢箱子");
        let mod_info = modScanner.scan_single_mod(&test_mod_path);
        let mod_info = match mod_info {
            Ok(op) => op.unwrap(),
            Err(e) => panic!("{}", e),
        };
        let manifest = mod_info.manifest_info;
        assert_eq!(
            mod_info.path,
            PathBuf::from(
                "C:/Program Files (x86)/Steam/steamapps/common/Stardew Valley/Mods/NPC不踢箱子"
            )
        );
        assert_eq!(manifest.Name, "NPC不踢箱子");
        assert_eq!(manifest.Version, "3.1.0");
        assert_eq!(
            manifest.Description,
            "NPCs no longer destroy placed objects in their paths. They would instead pass through them."
        );
        assert_eq!(manifest.UniqueId, "IamSaulC.NonDestructiveNPCs");
    }

    #[test]
    /// 原来是因为UniqueID和UniqueId的大小写问题
    fn test_json_parse2() {
        let modScanner = ModScanner::default();
        let test_mod_path = modScanner.mods_folder_path.join("ConsoleCommands");
        let mod_info = modScanner.scan_single_mod(&test_mod_path).unwrap().unwrap();
        let manifest = mod_info.manifest_info;
        assert_eq!(
            mod_info.path,
            PathBuf::from(
                "C:/Program Files (x86)/Steam/steamapps/common/Stardew Valley/Mods/ConsoleCommands"
            )
        );
        assert_eq!(manifest.Name, "Console Commands");
        assert_eq!(manifest.Version, "4.3.2");
        assert_eq!(
            manifest.Description,
            "Adds SMAPI console commands that let you manipulate the game."
        );
        assert_eq!(manifest.UniqueId, "SMAPI.ConsoleCommands");
    }

    #[test]
    fn test_scan_mods() {
        let mod_scanner = ModScanner::default();
        let mod_table = mod_scanner.scan_mods();
        assert_eq!(mod_table.len(), 3);
        let g_mod_info = mod_table.get("SilcentHonestFarmer.GoBackHome").unwrap();
        let s_mod_info = mod_table.get("SMAPI.SaveBackup").unwrap();
        let c_mod_info = mod_table.get("SMAPI.ConsoleCommands").unwrap();
        assert_eq!(
            g_mod_info.manifest_info.Description,
            "After the player presses Q, he/she will go back home immediately."
        );
        assert_eq!(
            s_mod_info.path,
            PathBuf::from(
                "C:/Program Files (x86)/Steam/steamapps/common/Stardew Valley/Mods/SaveBackup"
            )
        );
        assert_eq!(c_mod_info.manifest_info.Version, "4.3.2");
    }
}
