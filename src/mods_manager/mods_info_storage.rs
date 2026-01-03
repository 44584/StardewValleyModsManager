use super::mods_scanner;
use rusqlite::{Connection, Result};

/// 创建数据库文件, 返回Connection实例
fn establish_connection() -> Result<Connection> {
    let conn = Connection::open("mod_manager.db")?;

    // 启用外键约束
    conn.execute("PRAGMA foreign_keys = ON;", [])?;

    Ok(conn)
}

/// 创建数据库
fn init_db() -> Result<()> {
    Ok(())
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
