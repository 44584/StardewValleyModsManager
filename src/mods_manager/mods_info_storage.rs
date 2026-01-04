use super::{ManifestInfo, ModInfo, Profile};
use rusqlite::{Connection, Result};
use std::path::PathBuf;

pub struct ModManagerDb {
    conn: Connection,
}

impl ModManagerDb {
    /// 打开或创建数据库连接
    /// # 参数
    /// - `db_path`: db文件的路径
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // 启用外键约束
        conn.execute("PRAGMA foreign_keys = ON;", [])?;

        // 创建表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS mods (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                unique_id TEXT UNIQUE NOT NULL,
                name TEXT NOT NULL,
                version TEXT NOT NULL,
                description TEXT,
                mod_path TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS profiles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                description TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS profile_mods (
                profile_id INTEGER NOT NULL,
                mod_id INTEGER NOT NULL,
                PRIMARY KEY (profile_id, mod_id),
                FOREIGN KEY (profile_id) REFERENCES profiles(id) ON DELETE CASCADE,
                FOREIGN KEY (mod_id) REFERENCES mods(id) ON DELETE CASCADE
            )",
            [],
        )?;

        Ok(ModManagerDb { conn })
    }

    pub fn get_connection(&self) -> &Connection {
        &self.conn
    }

    /// 向数据库的mods表插入多个模组
    /// # 参数
    /// - `mods`:ModInfo的数组
    pub fn insert_mods(&self, mods: &Vec<ModInfo>) {
        // 用事务会出现借用, 拼接VALUES子句之后再试
        for mod_info in mods {
            let unique_id = mod_info.manifest_info.UniqueId.clone();
            let name = mod_info.manifest_info.Name.clone();
            let version = mod_info.manifest_info.Version.clone();
            let description = mod_info.manifest_info.Description.clone();
            let mod_path = mod_info.path.to_str().unwrap_or("");

            self.conn.execute("INSERT INTO mods (unique_id, name, version, description, mod_path) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![unique_id, name, version, description, mod_path]);
        }
    }

    /// 从数据库中的mods中删除单个模组, 同时会从所有配置中移除该模组
    /// # 参数
    /// - `mod_unique_id`: 需要删除的模组的uinque_id
    pub fn remove_mod(&self, mod_unique_id: &str) {
        self.conn.execute(
            "DELETE FROM mods WHERE unique_id = ?1",
            rusqlite::params![mod_unique_id],
        );
    }

    /// 查询所有模组
    /// - 返回值: ModInfo的数组
    pub fn get_mods(&self) -> Result<Vec<ModInfo>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, unique_id, name, version, description, mod_path FROM mods")?;
        let mod_info_s = stmt.query_map([], |row| {
            Ok(ModInfo {
                manifest_info: ManifestInfo {
                    UniqueId: row.get(1)?,
                    Name: row.get(2)?,
                    Version: row.get(3)?,
                    Description: row.get(4)?,
                },
                path: {
                    let path_str: String = row.get(5)?;
                    PathBuf::from(path_str)
                },
            })
        })?;

        let mut mod_info_vec = Vec::new();
        for mi in mod_info_s {
            mod_info_vec.push(mi?);
        }
        Ok(mod_info_vec)
    }

    /// 创建一个空配置
    /// # 参数
    /// - `name`: 配置名
    /// - `description`: 配置描述
    pub fn create_profile(&self, name: &str, description: &str) {
        let _ = self.conn.execute(
            "INSERT INTO profiles (name, description) VALUES (?1, ?2)",
            rusqlite::params![name, description],
        );
    }

    /// 移除一个配置
    /// # 参数
    /// - `name`: 配置名
    pub fn remove_profile(&self, name: &str) {
        let _ = self.conn.execute(
            "DELETE FROM profiles WHERE name = ?1",
            rusqlite::params![name],
        );
    }

    /// 查询所有配置
    /// - 返回值: 配置的数组
    pub fn get_profiles(&self) -> Vec<Profile> {
        let mut profiles = Vec::new();
        let mut stmt = match self
            .conn
            .prepare("SELECT name, description, created_at FROM profiles")
        {
            Ok(s) => s,
            Err(_) => return profiles,
        };
        let rows = stmt.query_map([], |row| {
            Ok(Profile {
                name: row.get(0)?,
                description: row.get(1)?,
                create_time: row.get(2)?,
            })
        });
        if let Ok(rows) = rows {
            for r in rows {
                if let Ok(profile) = r {
                    profiles.push(profile);
                }
            }
        }
        profiles
    }

    /// 查询一个配置中使用的模组
    /// # 参数
    /// - `profile_name`: 配置名
    /// # 返回
    /// - 模组信息数组
    pub fn get_mods_from_profile(&self, profile_name: &str) -> Vec<ModInfo> {
        let mut mods = Vec::new();
        let sql = r#"
            SELECT m.unique_id, m.name, m.version, m.description, m.mod_path
            FROM mods m
            JOIN profile_mods pm ON m.id = pm.mod_id
            JOIN profiles p ON pm.profile_id = p.id
            WHERE p.name = ?1
        "#;
        let mut stmt = match self.conn.prepare(sql) {
            Ok(s) => s,
            Err(_) => return mods,
        };
        let rows = stmt.query_map([profile_name], |row| {
            Ok(ModInfo {
                manifest_info: ManifestInfo {
                    UniqueId: row.get(0)?,
                    Name: row.get(1)?,
                    Version: row.get(2)?,
                    Description: row.get(3)?,
                },
                path: PathBuf::from(row.get::<_, String>(4)?),
            })
        });
        if let Ok(rows) = rows {
            for r in rows {
                if let Ok(modinfo) = r {
                    mods.push(modinfo);
                }
            }
        }
        mods
    }

    /// 向一个配置中插入一个模组
    /// # 参数
    /// - `profile_name`: 配置名
    /// - `mods`: 模组信息
    pub fn insert_mod_to_profile(&self, profile_name: &str, mods: &Vec<ModInfo>) {
        // 获取 profile_id
        let profile_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM profiles WHERE name = ?1",
                rusqlite::params![profile_name],
                |row| row.get(0),
            )
            .ok();
        if let Some(profile_id) = profile_id {
            for m in mods {
                // 获取 mod_id
                let mod_id: Option<i64> = self
                    .conn
                    .query_row(
                        "SELECT id FROM mods WHERE unique_id = ?1",
                        rusqlite::params![m.manifest_info.UniqueId],
                        |row| row.get(0),
                    )
                    .ok();
                if let Some(mod_id) = mod_id {
                    let _ = self.conn.execute(
                        "INSERT OR IGNORE INTO profile_mods (profile_id, mod_id) VALUES (?1, ?2)",
                        rusqlite::params![profile_id, mod_id],
                    );
                }
            }
        }
    }

    /// 从一个配置中移除一个模组
    /// # 参数
    /// - `profile_name`: 配置名
    /// - `mod_info`: 模组信息
    pub fn remove_mod_from_profile(&self, profile_name: &str, mod_info: ModInfo) {
        // 获取 profile_id
        let profile_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM profiles WHERE name = ?1",
                rusqlite::params![profile_name],
                |row| row.get(0),
            )
            .ok();
        // 获取 mod_id
        let mod_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM mods WHERE unique_id = ?1",
                rusqlite::params![mod_info.manifest_info.UniqueId],
                |row| row.get(0),
            )
            .ok();
        if let (Some(profile_id), Some(mod_id)) = (profile_id, mod_id) {
            let _ = self.conn.execute(
                "DELETE FROM profile_mods WHERE profile_id = ?1 AND mod_id = ?2",
                rusqlite::params![profile_id, mod_id],
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    fn test_db_path() -> PathBuf {
        PathBuf::from("./test_mods_manager.db")
    }

    fn clean_db() {
        let _ = fs::remove_file(test_db_path());
    }

    fn sample_mod(unique_id: &str, name: &str, path: &str) -> ModInfo {
        ModInfo {
            manifest_info: ManifestInfo {
                UniqueId: unique_id.to_string(),
                Name: name.to_string(),
                Version: "1.0.0".to_string(),
                Description: "desc".to_string(),
            },
            path: PathBuf::from(path),
        }
    }

    #[test]
    fn test_full_mods_manager_flow() -> Result<()> {
        clean_db();
        let db = ModManagerDb::new(test_db_path()).unwrap();

        // 1. 创建配置
        db.create_profile("p1", "desc1");
        db.create_profile("p2", "desc2");
        let profiles = db.get_profiles();
        assert_eq!(profiles.len(), 2);
        assert!(profiles.iter().any(|p| p.name == "p1"));

        // 2. 插入模组
        let mods = vec![
            sample_mod("mod.a", "A", "./a"),
            sample_mod("mod.b", "B", "./b"),
        ];
        db.insert_mods(&mods);
        let all_mods = db.get_mods().unwrap();
        assert!(all_mods.iter().any(|m| m.manifest_info.UniqueId == "mod.a"));

        // 3. 配置关联模组
        db.insert_mod_to_profile("p1", &mods);
        let p1_mods = db.get_mods_from_profile("p1");
        assert_eq!(p1_mods.len(), 2);

        // 4. 配置移除模组
        db.remove_mod_from_profile("p1", mods[0].clone());
        let p1_mods2 = db.get_mods_from_profile("p1");
        assert_eq!(p1_mods2.len(), 1);
        assert_eq!(p1_mods2[0].manifest_info.UniqueId, "mod.b");

        // 5. 删除配置
        db.remove_profile("p2");
        let profiles2 = db.get_profiles();
        assert_eq!(profiles2.len(), 1);
        assert_eq!(profiles2[0].name, "p1");

        // 6. 删除模组
        db.remove_mod("mod.b");
        let all_mods2 = db.get_mods().unwrap();
        assert!(
            all_mods2
                .iter()
                .all(|m| m.manifest_info.UniqueId != "mod.b")
        );

        clean_db();
        Ok(())
    }
}

// 数据库表设计
// 三张表, mods(模组元数据), profiles(记录配置的元信息(不含配置所用的模组)), profile_mods(只记录mods与profiles的多对多关系)

// -- 模组信息表
// CREATE TABLE IF NOT EXISTS mods (
//     id INTEGER PRIMARY KEY AUTOINCREMENT,
//     unique_id TEXT UNIQUE NOT NULL,
//     name TEXT NOT NULL,
//     version TEXT NOT NULL,
//     description TEXT,
//     mod_path TEXT NOT NULL
// );

// -- 配置方案表
// CREATE TABLE IF NOT EXISTS profiles (
//     id INTEGER PRIMARY KEY AUTOINCREMENT,
//     name TEXT UNIQUE NOT NULL,  -- 配置方案名称，在此唯一
//     description TEXT,
//     created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
// );

// -- 配置-模组关联表
// CREATE TABLE IF NOT EXISTS profile_mods (
//     profile_id INTEGER NOT NULL,
//     mod_id INTEGER NOT NULL,
//     PRIMARY KEY (profile_id, mod_id), -- 联合主键，防止重复关联
//     --ON DELETE CASCADE外键约束意味着当删除一个配置或一个模组时，关联表中的对应记录会自动被删除
//     FOREIGN KEY (profile_id) REFERENCES profiles(id) ON DELETE CASCADE,
//     FOREIGN KEY (mod_id) REFERENCES mods(id) ON DELETE CASCADE
// );
