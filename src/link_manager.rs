use std::os::windows;
use std::path::PathBuf;

/// `link_parent_path`: 所有配置的父文件夹
pub struct LinkManager {
    pub link_parent_path: PathBuf,
}

// 符号链接在现有的扫描中不会被识别成模组
impl LinkManager {
    pub fn default() -> Self {
        LinkManager {
            link_parent_path: PathBuf::from(
                "C:/Program Files (x86)/Steam/steamapps/common/Stardew Valley/Profiles",
            ),
        }
    }
    /// 创建 目录符号链接
    /// - 需要cmd的管理员权限,
    /// - 或者 系统>开发者选项>开发人员模式 打开
    /// # 参数
    /// - `original_dir_path`：模组实际存放的物理路径;
    /// - `link_dir_path`：命令执行后, 会创建的目录链接, 要参考self.link_partent_path
    fn create_link(
        &self,
        original_dir_path: &PathBuf,
        link_dir_path: &PathBuf,
    ) -> std::io::Result<()> {
        windows::fs::symlink_dir(original_dir_path, link_dir_path)?;
        eprintln!(
            "{:?} -> {:?} 符号链接创建成功.",
            link_dir_path, original_dir_path
        );
        Ok(())
    }

    /// 创建一个配置并加入多个目录符号链接, 也可以用作向一个配置中添加多个目录符号链接
    /// - 需要cmd的管理员权限,
    /// - 或者 系统>开发者选项>开发人员模式 打开
    /// # 参数
    /// - `mod_path_vec`：模组实际存放的物理路径的数组;
    /// - `profile_name`: 配置名称
    pub fn create_links(
        &self,
        mod_path_vec: &Vec<PathBuf>,
        profile_name: &str,
    ) -> std::io::Result<()> {
        //如果profile不存在对应目录, 则创建
        let profile_path = self.link_parent_path.join(profile_name);
        if !profile_path.exists() {
            std::fs::create_dir_all(&profile_path)?;
        }

        //接下来为参数数组中的每个模组创建目录链接
        // bug8c0096a 这里使用模组文件夹名作为链接文件夹名
        for odp in mod_path_vec {
            let mod_folder_name = odp.file_name().unwrap().to_str().unwrap();
            let _ = self.create_link(odp, &profile_path.join(mod_folder_name));
        }
        Ok(())
    }

    /// 通过删除profile对应的link的folder, 完成删除profile在文件系统的同步
    /// # 参数
    /// - `profile_name`: profile名, 与self.link_parent_path拼接成完整路径
    pub fn remove_profile(&self, profile_name: &str) -> Result<(), String> {
        match std::fs::remove_dir_all(self.link_parent_path.join(profile_name)) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
    /// 从配置中移除mod对应的目录链接
    /// # 参数
    /// - `profile_name`: profile名, 与self.link_parent_path拼接成完整路径
    /// - `mod_path`: 模组路径
    /// bug8c0096a 这里使用模组名(与模组文件夹名不同)拼接出 profile下链接文件夹路径
    pub fn remove_mod_from_profile(
        &self,
        profile_name: &str,
        mod_path: PathBuf,
    ) -> Result<(), String> {
        let mod_folder_name = mod_path.file_name().unwrap().to_str().unwrap();
        let mod_link_path = self
            .link_parent_path
            .join(profile_name)
            .join(mod_folder_name);
        match std::fs::remove_dir_all(mod_link_path) {
            Ok(_) => {
                eprintln!(
                    "link_manager: link{:?} -> {:?} removed.",
                    mod_folder_name, mod_path
                );
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_link() {
        let l_m = LinkManager::default();
        let original_dir_path = PathBuf::from(
            "C:/Program Files (x86)/Steam/steamapps/common/Stardew Valley/Mods/GoBackHome",
        );
        let link_dir_path = l_m
            .link_parent_path
            .join("test1_profile")
            .join("GoBackHome");

        let _ = l_m.create_link(&original_dir_path, &link_dir_path);

        assert!(link_dir_path.is_dir());
    }

    #[test]
    fn test_create_links() {
        let l_m = LinkManager::default();

        let profile_name = "test_profile";
        let mods_path = vec![
            PathBuf::from(
                "C:/Program Files (x86)/Steam/steamapps/common/Stardew Valley/Mods/GoBackHome",
            ),
            PathBuf::from(
                "C:/Program Files (x86)/Steam/steamapps/common/Stardew Valley/Mods/ConsoleCommands",
            ),
        ];
        let _ = l_m.create_links(&mods_path, profile_name);

        for mp in mods_path {
            let mod_folder_name = mp.file_name().unwrap().to_str().unwrap();
            let link_path = l_m
                .link_parent_path
                .join(format!("{}/{}", profile_name, mod_folder_name));
            assert!(link_path.is_dir());
        }
    }

    #[test]
    fn test_remove_profile() {
        let l_m = LinkManager::default();

        let profile_name = "test_profile";
        let _ = l_m.remove_profile(profile_name);
    }
}
