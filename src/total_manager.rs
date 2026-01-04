use crate::link_manager::LinkManager;
use crate::mods_manager;
use crate::mods_manager::mods_info_storage::ModManagerDb;
use crate::mods_manager::mods_scanner::ModScanner;

use std::fs;
use std::path::PathBuf;

//应该让ModInfo和Profile (的成员) 成为通用的统一数据, 这样能使多个接口保持统一

struct Manager {
    scanner: ModScanner,
    database_manager: ModManagerDb,
    link_manager: LinkManager,
}

impl Manager {
    pub fn default() -> Self {
        Manager {
            scanner: ModScanner::default(),
            database_manager: ModManagerDb::new(PathBuf::from("./mod_manager.db")).unwrap(),
            link_manager: LinkManager::default(),
        }
    }

    /// 本地所有的模组注册进入数据库
    /// - 如果已存在, 不处理
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
        //Todo: 删除所有指向这个模组的链接目录
        // 首先获取profiles文件夹下的所有profile_name
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
}
