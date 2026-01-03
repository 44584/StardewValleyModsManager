use std::os::windows;
use std::path::PathBuf;

// 符号链接在现有的扫描中不会被识别成模组

/// 创建 目录符号链接
/// - 需要cmd的管理员权限,
/// - 或者 系统>开发者选项>开发人员模式 打开
/// # 参数
/// - `original_dir_path`：模组实际存放的物理路径;
/// - `link_dir_path`：命令执行后, 会创建的目录链接;
pub fn create_link(original_dir_path: &PathBuf, link_dir_path: &PathBuf) -> std::io::Result<()> {
    windows::fs::symlink_dir(original_dir_path, link_dir_path)?;
    eprintln!(
        "{:?} -> {:?} 符号链接创建成功.",
        link_dir_path, original_dir_path
    );
    Ok(())
}

/// 创建 多个目录符号链接
/// - 需要cmd的管理员权限,
/// - 或者 系统>开发者选项>开发人员模式 打开
/// # 参数
/// - `original_dir_paths`：模组实际存放的物理路径的数组;
/// - `mods_folder_path`：配置文件的目录;
/// - `profile_name`: 配置名称
pub fn create_links(
    original_dir_paths: &Vec<PathBuf>,
    mods_folder_path: &PathBuf,
    profile_name: &str,
) -> std::io::Result<()> {
    //首先为profile创建目录
    let profile_path = mods_folder_path.join(profile_name);
    std::fs::create_dir(&profile_path)?;

    //接下来为参数数组中的每个模组创建目录链接
    for odp in original_dir_paths {
        let mod_name = odp.file_name().unwrap().to_str().unwrap();
        let _ = create_link(odp, &profile_path.join(mod_name));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_link() {
        let original_dir_path = PathBuf::from(
            "C:/Program Files (x86)/Steam/steamapps/common/Stardew Valley/Mods_simple/GoBackHome",
        );
        let link_dir_path =
            PathBuf::from("C:/Program Files (x86)/Steam/steamapps/common/Stardew Valley/Mods/gbh");

        let _ = create_link(&original_dir_path, &link_dir_path);

        let link_dir_path =
            PathBuf::from("C:/Program Files (x86)/Steam/steamapps/common/Stardew Valley/Mods/gbh");
        assert!(link_dir_path.is_dir());
    }

    #[test]
    fn test_create_links() {
        let mods_folder_path =
            PathBuf::from("C:/Program Files (x86)/Steam/steamapps/common/Stardew Valley/Mods");
        let profile_name = "test_profile";
        let mods_path = vec![
            PathBuf::from(
                "C:/Program Files (x86)/Steam/steamapps/common/Stardew Valley/Mods_simple/GoBackHome",
            ),
            PathBuf::from(
                "C:/Program Files (x86)/Steam/steamapps/common/Stardew Valley/Mods_simple/ConsoleCommands",
            ),
        ];
        let _ = create_links(&mods_path, &mods_folder_path, profile_name);

        for mod_name in mods_path {
            let mod_name = mod_name.file_name().unwrap().to_str().unwrap();
            let link_path = mods_folder_path.join(format!("{}/{}", profile_name, mod_name));
            assert!(link_path.is_dir());
        }
    }
}
