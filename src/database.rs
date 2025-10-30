use crate::models::Item;
use chrono::NaiveDate;
use dirs;
use rusqlite::{Connection, Result, params};
use std::path::PathBuf;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;
        let conn = Connection::open(db_path)?;

        // 创建表
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

        Ok(Self { conn })
    }

    fn get_db_path() -> Result<PathBuf> {
        let data_dir = dirs::data_dir().ok_or_else(|| {
            rusqlite::Error::InvalidPath("Cannot determine data directory".to_string().into())
        })?;

        let app_dir = data_dir.join("things-cost");
        std::fs::create_dir_all(&app_dir).map_err(|e| rusqlite::Error::InvalidPath(e.to_string().into()))?;

        Ok(app_dir.join("data.db"))
    }

    pub fn add_item(&self, item: &Item) -> Result<()> {
        self.conn.execute(
            "INSERT INTO items (name, start_date, discard_date, price, currency)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                &item.name,
                &item.start_date.to_string(),
                &item.discard_date.map(|d| d.to_string()),
                item.price,
                &item.currency
            ],
        )?;
        Ok(())
    }

    pub fn get_all_items(&self) -> Result<Vec<Item>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, start_date, discard_date, price, currency FROM items ORDER BY start_date DESC"
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
                "UPDATE items SET name = ?1, start_date = ?2, discard_date = ?3, price = ?4, currency = ?5
                 WHERE id = ?6",
                params![
                    &item.name,
                    &item.start_date.to_string(),
                    &item.discard_date.map(|d| d.to_string()),
                    item.price,
                    &item.currency,
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

