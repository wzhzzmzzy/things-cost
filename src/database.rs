use crate::models::Item;
use chrono::NaiveDate;
use rusqlite::{Connection, Result, params};
use std::path::PathBuf;

const CURRENT_DB_VERSION: i32 = 2;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;
        let conn = Connection::open(db_path)?;

        // 初始化数据库版本管理
        Self::init_database(&conn)?;

        Ok(Self { conn })
    }

    fn init_database(conn: &Connection) -> Result<()> {
        // 创建版本表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS db_version (
                version INTEGER PRIMARY KEY
            )",
            [],
        )?;

        // 获取当前版本
        let current_version = Self::get_current_version(conn)?;

        // 执行迁移
        Self::migrate_database(conn, current_version)?;

        Ok(())
    }

    fn get_current_version(conn: &Connection) -> Result<i32> {
        // 检查版本表是否存在且包含数据
        let version_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='db_version'",
                [],
                |row| row.get(0),
            )?;

        if !version_exists {
            // 检查 items 表是否存在来判断是否是旧数据库
            let items_exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='items'",
                    [],
                    |row| row.get(0),
                )?;

            if items_exists {
                // items 表存在但没有版本表，说明是版本1的数据库
                return Ok(1);
            } else {
                // 全新数据库
                return Ok(0);
            }
        }

        let version_count: i32 = conn
            .query_row("SELECT COUNT(*) FROM db_version", [], |row| row.get(0))?;

        if version_count == 0 {
            // 版本表存在但没有版本记录，说明是旧数据库
            return Ok(1);
        }

        // 获取当前版本
        conn.query_row("SELECT version FROM db_version", [], |row| row.get(0))
    }

    fn migrate_database(conn: &Connection, current_version: i32) -> Result<()> {
        if current_version < 1 {
            // 版本 0 -> 1: 创建初始表结构
            Self::migrate_to_version_1(conn)?;
        }

        if current_version < 2 {
            // 版本 1 -> 2: 添加 selling_price 列
            Self::migrate_to_version_2(conn)?;
        }

        // 更新到最新版本
        if current_version < CURRENT_DB_VERSION {
            conn.execute(
                "INSERT OR REPLACE INTO db_version (version) VALUES (?1)",
                params![CURRENT_DB_VERSION],
            )?;
        }

        Ok(())
    }

    fn migrate_to_version_1(conn: &Connection) -> Result<()> {
        // 创建初始表结构
        conn.execute(
            "CREATE TABLE IF NOT EXISTS items (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                start_date TEXT NOT NULL,
                discard_date TEXT,
                price REAL NOT NULL,
                currency TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    fn migrate_to_version_2(conn: &Connection) -> Result<()> {
        // 检查 selling_price 列是否存在
        let column_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('items') WHERE name='selling_price'",
                [],
                |row| row.get(0),
            )?;

        if !column_exists {
            // 添加 selling_price 列
            conn.execute("ALTER TABLE items ADD COLUMN selling_price REAL", [])?;
        }

        Ok(())
    }

    /// 用于测试的构造函数，使用内存数据库
    #[cfg(test)]
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;

        // 直接创建完整的表结构用于测试
        conn.execute(
            "CREATE TABLE IF NOT EXISTS items (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                start_date TEXT NOT NULL,
                discard_date TEXT,
                price REAL NOT NULL,
                currency TEXT NOT NULL,
                selling_price REAL
            )",
            [],
        )?;

        // 创建版本表并设置版本
        conn.execute(
            "CREATE TABLE IF NOT EXISTS db_version (
                version INTEGER PRIMARY KEY
            )",
            [],
        )?;

        conn.execute(
            "INSERT OR REPLACE INTO db_version (version) VALUES (?1)",
            params![CURRENT_DB_VERSION],
        )?;

        Ok(Self { conn })
    }

    fn get_db_path() -> Result<PathBuf> {
        let data_dir = dirs::data_dir().ok_or_else(|| {
            rusqlite::Error::InvalidPath("Cannot determine data directory".to_string().into())
        })?;

        let app_dir = data_dir.join("things-cost");
        std::fs::create_dir_all(&app_dir)
            .map_err(|e| rusqlite::Error::InvalidPath(e.to_string().into()))?;

        Ok(app_dir.join("data.db"))
    }

    pub fn add_item(&self, item: &Item) -> Result<()> {
        self.conn.execute(
            "INSERT INTO items (name, start_date, discard_date, price, currency, selling_price)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &item.name,
                &item.start_date.to_string(),
                &item.discard_date.map(|d| d.to_string()),
                item.price,
                &item.currency,
                &item.selling_price
            ],
        )?;
        Ok(())
    }

    pub fn get_all_items(&self) -> Result<Vec<Item>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, start_date, discard_date, price, currency, selling_price FROM items ORDER BY start_date DESC"
        )?;

        let item_iter = stmt.query_map([], |row| {
            let start_date: String = row.get(2)?;
            let discard_date: Option<String> = row.get(3)?;

            Ok(Item {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                start_date: NaiveDate::parse_from_str(&start_date, "%Y-%m-%d").map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?,
                discard_date: discard_date
                    .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok()),
                price: row.get(4)?,
                currency: row.get(5)?,
                selling_price: row.get(6)?,
            })
        })?;

        let mut items = Vec::new();
        for item in item_iter {
            items.push(item?);
        }
        Ok(items)
    }

    pub fn update_item(&self, item: &Item) -> Result<()> {
        if let Some(id) = item.id {
            self.conn.execute(
                "UPDATE items SET name = ?1, start_date = ?2, discard_date = ?3, price = ?4, currency = ?5, selling_price = ?6
                 WHERE id = ?7",
                params![
                    &item.name,
                    &item.start_date.to_string(),
                    &item.discard_date.map(|d| d.to_string()),
                    item.price,
                    &item.currency,
                    &item.selling_price,
                    id
                ],
            )?;
        }
        Ok(())
    }

    pub fn delete_item(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM items WHERE id = ?1", params![id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_add_and_get_items() -> Result<()> {
        let db = Database::new_in_memory()?;

        let item = Item::new(
            "Test Item".to_string(),
            NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            None,
            1000.0,
            "CNY".to_string(),
        );

        // 添加物品
        db.add_item(&item)?;

        // 获取所有物品
        let items = db.get_all_items()?;

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Test Item");
        assert_eq!(items[0].price, 1000.0);
        assert_eq!(items[0].currency, "CNY");
        assert!(items[0].id.is_some());

        Ok(())
    }

    #[test]
    fn test_add_item_with_discard_date() -> Result<()> {
        let db = Database::new_in_memory()?;

        let item = Item::new(
            "Test Item".to_string(),
            NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            Some(NaiveDate::from_ymd_opt(2022, 1, 1).unwrap()),
            1000.0,
            "CNY".to_string(),
        );

        db.add_item(&item)?;
        let items = db.get_all_items()?;

        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].discard_date,
            Some(NaiveDate::from_ymd_opt(2022, 1, 1).unwrap())
        );

        Ok(())
    }

    #[test]
    fn test_update_item() -> Result<()> {
        let db = Database::new_in_memory()?;

        let mut item = Item::new(
            "Test Item".to_string(),
            NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            None,
            1000.0,
            "CNY".to_string(),
        );

        // 添加物品
        db.add_item(&item)?;
        let items = db.get_all_items()?;
        let id = items[0].id.unwrap();

        // 更新物品
        item.id = Some(id);
        item.name = "Updated Item".to_string();
        item.price = 2000.0;

        db.update_item(&item)?;

        // 验证更新
        let items = db.get_all_items()?;
        assert_eq!(items[0].name, "Updated Item");
        assert_eq!(items[0].price, 2000.0);

        Ok(())
    }

    #[test]
    fn test_delete_item() -> Result<()> {
        let db = Database::new_in_memory()?;

        let item = Item::new(
            "Test Item".to_string(),
            NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            None,
            1000.0,
            "CNY".to_string(),
        );

        // 添加物品
        db.add_item(&item)?;
        let items = db.get_all_items()?;
        let id = items[0].id.unwrap();

        // 删除物品
        db.delete_item(id)?;

        // 验证删除
        let items = db.get_all_items()?;
        assert_eq!(items.len(), 0);

        Ok(())
    }

    #[test]
    fn test_add_item_with_selling_price() -> Result<()> {
        let db = Database::new_in_memory()?;

        let mut item = Item::new(
            "Test Item".to_string(),
            NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            None,
            1000.0,
            "CNY".to_string(),
        );
        item.selling_price = Some(500.0);

        // 添加物品
        db.add_item(&item)?;

        // 获取所有物品
        let items = db.get_all_items()?;

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Test Item");
        assert_eq!(items[0].price, 1000.0);
        assert_eq!(items[0].currency, "CNY");
        assert_eq!(items[0].selling_price, Some(500.0));
        assert!(items[0].id.is_some());

        Ok(())
    }

    #[test]
    fn test_get_all_items_empty() -> Result<()> {
        let db = Database::new_in_memory()?;

        let items = db.get_all_items()?;
        assert_eq!(items.len(), 0);

        Ok(())
    }
}
