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
    pub fn insert_mods(&self, mods: &Vec<ModInfo>) {}

    /// 从数据库中的mods中删除单个模组, 同时会从所有配置中移除该模组
    /// # 参数
    /// - `mod_unique_id`: 需要删除的模组的uinque_id
    pub fn remove_mod(&self, mod_unique_id: &ModInfo) {}

    /// 查询所有模组
    /// - 返回值: ModInfo的数组
    pub fn get_mods(&self) -> Vec<ModInfo> {
        Vec::new()
    }

    /// 创建一个空配置
    /// # 参数
    /// - `name`: 配置名
    /// - `description`: 配置描述
    pub fn create_profile(&self, name: &str, description: &str) {}

    /// 移除一个配置
    /// # 参数
    /// - `name`: 配置名
    pub fn remove_profile(&self, name: &str) {}

    /// 查询所有配置
    /// - 返回值: 配置的数组
    pub fn get_profiles(&self) -> Vec<Profile> {
        Vec::new()
    }

    /// 查询一个配置中使用的模组
    /// # 参数
    /// - `profile_name`: 配置名
    /// # 返回
    /// - 模组信息数组
    pub fn get_mods_from_profile(&self, profile_name: &str) -> Vec<ModInfo> {
        Vec::new()
    }

    /// 向一个配置中插入一个模组
    /// # 参数
    /// - `profile_name`: 配置名
    /// - `mods`: 模组信息
    pub fn insert_mod_to_profile(&self, profile_name: &str, mods: &Vec<ModInfo>) {}

    /// 从一个配置中移除一个模组
    /// # 参数
    /// - `profile_name`: 配置名
    /// - `mod_info`: 模组信息
    pub fn remove_mod_from_profile(&self, profile_name: &str, mod_info: ModInfo) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_db() -> Result<()> {
        let db = ModManagerDb::new(PathBuf::from("./test.db")).unwrap();
        // 检查表是否创建成功
        let mut stmt = db
            .get_connection()
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")?;
        let tables: Vec<String> = stmt
            .query_map([], |row| Ok(row.get(0)?))?
            .collect::<Result<Vec<_>>>()?;
        eprintln!("{:?}", tables);
        //这里的1代表sqlite_sequence表, 这是一个SQLite系统表
        assert_eq!(tables.len(), 1 + 3);
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
