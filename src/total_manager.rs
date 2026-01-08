use crate::link_manager::LinkManager;
use crate::mods_manager;
use crate::mods_manager::mods_info_storage::ModManagerDb;
use crate::mods_manager::mods_scanner::ModScanner;
use std::process::Command;

use std::fs;
use std::path::PathBuf;

//应该让ModInfo和Profile (的成员) 成为通用的统一数据, 这样能使多个接口保持统一

pub struct Manager {
    smapi_path: PathBuf,
    scanner: ModScanner,
    database_manager: ModManagerDb,
    link_manager: LinkManager,
}

impl Manager {
    /// 允许设置scanner的mods路径
    pub fn set_scanner_mods_path(&mut self, mods_path: PathBuf) {
        self.scanner.set_mods_path(mods_path);
    }

    /// 允许设置SMAPI位置
    pub fn set_smapi_path(&mut self, smapi_path: PathBuf) {
        self.link_manager.link_parent_path = smapi_path;
    }

    /// 支持重置, 然后重新输入SMAPI与mods的路径
    pub fn reset(&self) {
        let setting_path = dirs::data_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap())
            .join("StardewModsManager")
            .join("setting.toml");

        fs::remove_file(setting_path);
    }
}

impl Manager {
    pub fn default() -> Self {
        // 获取用户数据以及配置文件夹
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap())
            .join("StardewModsManager");
        // 若不存在,创建
        fs::create_dir_all(&data_dir).unwrap_or_else(|e| {
            eprintln!("无法创建数据目录 {:?}: {}", data_dir, e);
        });
        let db_path = data_dir.join("mod_manager.db");
        let config_path = data_dir.join("setting.toml");

        // 如果配置文件存在, 说明已经配置模组目录;
        // 否则为首次使用, 用户可以输入自定义模组目录
        let scanner = if config_path.exists() {
            if let Some(cfg) = crate::config::AppConfig::load_from_file(&config_path) {
                ModScanner::from(&cfg.mods_folder_path)
            } else {
                ModScanner::default()
            }
        } else {
            ModScanner::default()
        };

        // 如果配置文件存在, 说明已经配置SMAPI目录;
        // 否则为首次使用, 用户可以输入SMAPI目录
        let mut smapi_path = PathBuf::from(
            "C:/Program Files (x86)/Steam/steamapps/common/Stardew Valley/StardewModdingAPI.exe",
        );
        if config_path.exists() {
            if let Some(cfg) = crate::config::AppConfig::load_from_file(&config_path) {
                smapi_path = PathBuf::from(cfg.smapi_path.clone());
            }
        }

        Manager {
            smapi_path,
            scanner,
            database_manager: ModManagerDb::new(db_path).unwrap(),
            link_manager: LinkManager::default(),
        }
    }

    /// 本地所有的模组注册进入数据库
    /// - 如果模组已存在, 则更新模组信息
    pub fn register_all_mods(&mut self) {
        let all_mods = self.scanner.scan_mods();
        let all_mods: Vec<mods_manager::ModInfo> = all_mods.into_values().collect();
        self.database_manager.insert_mods(&all_mods);
    }

    /// 移除一个模组, 实际上这个模组文件夹不被删除, 但是指向它的链接需要删除
    /// # 参数
    /// - `mod_unique_id` 模组的UniqueId
    ///
    /// Todo: 该函数的移除链接应该交给底层实现
    pub fn remove_mod(&mut self, mod_unique_id: &str) {}

    /// 返回所有模组的信息
    pub fn get_registered_mods(&self) -> &[mods_manager::ModInfo] {
        self.database_manager.get_cached_mods()
    }

    /// 创建一个空的profile
    /// # 参数
    /// - `name`: 配置名
    /// - `description`: 配置描述
    pub fn create_empty_profile(&mut self, name: &str, description: &str) {
        let result = self.database_manager.create_profile(name, description);
        match result {
            Ok(o) => eprintln!("OK"),
            Err(e) => eprintln!("{}", e.to_string()),
        }

        self.link_manager.create_links(&Vec::new(), name);
    }

    /// 删除一个配置
    /// # 参数
    /// - `name`: 配置名
    /// # 返回值
    /// 剩余的profile数量
    pub fn remove_profile(&mut self, name: &str) -> Result<u16, rusqlite::Error> {
        let num_profiles = self.database_manager.remove_profile(name);
        self.link_manager.remove_profile(name).unwrap();
        num_profiles
    }

    /// 返回所有的profile
    pub fn get_all_profiles(&self) -> &[mods_manager::Profile] {
        self.database_manager.get_cached_profiles()
    }

    /// 返回一个profile中启用的mod
    /// # 参数
    /// - `profile_name`: profile名
    pub fn get_mods_from_profile(&self, profile_name: &str) -> Vec<mods_manager::ModInfo> {
        self.database_manager.get_mods_from_profile(profile_name)
    }

    /// 在指定profile中加入一些模组
    /// # 参数
    /// -  `mods`: 模组的数组, Vec<ModInfo>
    /// - `profile_name`: 配置名
    pub fn insert_mods_to_profile(&self, mods: Vec<mods_manager::ModInfo>, profile_name: &str) {
        self.database_manager
            .insert_mod_to_profile(profile_name, &mods);
        let mod_path_vec = mods.into_iter().map(|mi| mi.path).collect();
        self.link_manager.create_links(&mod_path_vec, profile_name);
    }

    /// 从指定配置中移除某个模组
    pub fn remove_mod_from_profile(&self, mod_info: mods_manager::ModInfo, profile_name: &str) {
        self.database_manager
            .remove_mod_from_profile(profile_name, mod_info.clone());
        let mod_path = mod_info.path.clone();
        match self
            .link_manager
            .remove_mod_from_profile(profile_name, mod_path)
        {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e)
            }
        }
    }

    pub fn launch_stardew_valley(&self, profile_name: &str) {
        let child = Command::new(&self.smapi_path)
            .arg("--mods-path")
            .arg(self.link_manager.link_parent_path.join(profile_name))
            .spawn()
            .unwrap();
        eprintln!("{}已启动", child.id());
    }
}
