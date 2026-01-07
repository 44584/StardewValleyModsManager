//! StardewModsManager 集成测试

use std::fs;
use std::path::PathBuf;

use StardewModsManager::mods_manager::ModInfo;
use StardewModsManager::total_manager::Manager;

fn setup_test_mods_dir() -> PathBuf {
    //模组所在目录
    PathBuf::from("C:/Program Files (x86)/Steam/steamapps/common/Stardew Valley/Mods")
}

#[test]
fn test_manager_integration_flow() {
    let mods_dir = setup_test_mods_dir();
    let mut manager = Manager::default();
    manager.set_scanner_mods_path(mods_dir);

    manager.register_all_mods();

    let profile_name = "test_profile";
    let description = "测试用配置";
    manager.create_empty_profile(profile_name, description);
    let profiles = manager.get_all_profiles();
    assert!(profiles.iter().any(|p| p.name == profile_name));

    let all_mods = manager.get_registered_mods();
    assert!(all_mods.len() == 3, "应有3个模组被注册");

    // 选中部分模组加入 profile
    let selected_mods: Vec<ModInfo> = all_mods
        .to_vec()
        .into_iter()
        .filter(|m| {
            m.manifest_info.Name == "Console Commands" || m.manifest_info.Name == "GoBackHome"
        })
        .collect();
    assert_eq!(selected_mods.len(), 2);
    manager.insert_mods_to_profile(selected_mods.clone(), profile_name);
    let mods_in_profile = manager.get_mods_from_profile(profile_name);
    assert_eq!(mods_in_profile.len(), 2);

    // 测试增删改查
    let save_backup_mod = all_mods
        .clone()
        .into_iter()
        .find(|m| m.manifest_info.Name == "Save Backup")
        .unwrap();
    manager.insert_mods_to_profile(vec![save_backup_mod.clone()], profile_name);
    let mods_in_profile = manager.get_mods_from_profile(profile_name);
    assert_eq!(mods_in_profile.len(), 3);

    // 移除 ConsoleCommands
    let cc_mod = mods_in_profile
        .iter()
        .find(|m| m.manifest_info.Name == "Console Commands")
        .unwrap()
        .clone();
    manager.remove_mod_from_profile(cc_mod, profile_name);
    let mods_in_profile = manager.get_mods_from_profile(profile_name);
    assert_eq!(mods_in_profile.len(), 2);

    // 获取 profile
    let profiles = manager.get_all_profiles();
    assert!(profiles.iter().any(|p| p.name == profile_name));

    // 启动游戏
    manager.launch_stardew_valley(profile_name);

    // 这里的的启动函数和删除不是顺序执行的, 导致先删除后启动, debug好久...
    // manager.remove_profile(profile_name);
    // let profiles = manager.get_all_profiles();
    // assert!(!profiles.iter().any(|p| p.name == profile_name));
}
