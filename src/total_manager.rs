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
    /// 允许在测试时设置scanner的mods路径
    pub fn set_scanner_mods_path(&mut self, mods_path: PathBuf) {
        self.scanner.set_mods_path(mods_path);
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

        Manager {
            smapi_path: PathBuf::from(
                "C:/Program Files (x86)/Steam/steamapps/common/Stardew Valley/StardewModdingAPI.exe",
            ),
            scanner,
            database_manager: ModManagerDb::new(db_path).unwrap(),
            link_manager: LinkManager::default(),
        }
    }

    /// 本地所有的模组注册进入数据库
    /// - 如果模组已存在, 则更新模组信息
    pub fn register_all_mods(&self) {
        let all_mods = self.scanner.scan_mods();
        let all_mods: Vec<mods_manager::ModInfo> = all_mods.into_values().collect();
        self.database_manager.insert_mods(&all_mods);
    }

    /// 移除一个模组, 实际上这个模组文件夹不被删除, 但是指向它的链接需要删除
    /// # 参数
    /// - `mod_unique_id` 模组的UniqueId
    pub fn remove_mod(&self, mod_unique_id: &str) {
        self.database_manager.remove_mod(mod_unique_id);
        // 首先获取rofiles文件夹下的所有profile_name
        let profiles = &self.link_manager.link_parent_path;
        let profile_name_s: Vec<String> = fs::read_dir(profiles)
            .unwrap()
            .filter_map(|entry| entry.ok()) // 过滤掉错误条目，保留Ok值
            .filter(|entry| entry.file_type().unwrap().is_dir()) // 过滤出目录类型的条目
            .map(|entry| entry.file_name().to_string_lossy().into_owned()) // 映射为目录名称字符串
            .collect();

        // 再获取模组名(要绝对可靠, 所以统一使用数据库)
        let mod_name = &self.database_manager.get_modname_by_uniqueid(mod_unique_id);
        for pn in &profile_name_s {
            self.link_manager.remove_mod_from_profile(pn, mod_name);
        }
    }

    /// 返回所有模组的信息
    pub fn get_registered_mods(&self) -> Vec<mods_manager::ModInfo> {
        self.database_manager.get_mods().unwrap()
    }

    /// 创建一个空的profile
    /// # 参数
    /// - `name`: 配置名
    /// - `description`: 配置描述
    pub fn create_empty_profile(&self, name: &str, description: &str) {
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
    pub fn remove_profile(&self, name: &str) {
        self.database_manager.remove_profile(name);
        self.link_manager.remove_profile(name).unwrap();
    }

    /// 返回所有的profile
    pub fn get_all_profiles(&self) -> Vec<mods_manager::Profile> {
        self.database_manager.get_profiles()
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
        let mod_name = mod_info.manifest_info.Name.clone();
        self.link_manager
            .remove_mod_from_profile(profile_name, &mod_name);
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
