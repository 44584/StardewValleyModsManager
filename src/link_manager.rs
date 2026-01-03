use std::os::windows::fs;
use std::path::PathBuf;

// 符号链接在现有的扫描中不会被识别成模组

/// 创建 目录符号链接
/// - 需要cmd的管理员权限,
/// - 或者 系统>开发者选项>开发人员模式 打开
/// # 参数
/// - `link_dir_path`：命令执行后, 会创建的目录链接;
/// - `original_dir_path`：模组实际存放的物理路径;
pub fn create_link(original_dir_path: &PathBuf, link_dir_path: &PathBuf) -> std::io::Result<()> {
    fs::symlink_dir(original_dir_path, link_dir_path)?;
    println!("符号链接创建成功！");
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

        create_link(&original_dir_path, &link_dir_path);

        let link_dir_path =
            PathBuf::from("C:/Program Files (x86)/Steam/steamapps/common/Stardew Valley/Mods/gbh");
        assert!(link_dir_path.is_dir());
    }
}
