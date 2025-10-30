use crate::database::Database;
use crate::models::Item;
use chrono::Utc;
use rusqlite::Result;

pub struct ItemService {
    db: Database,
}

impl ItemService {
    pub fn new() -> Result<Self> {
        let db = Database::new()?;
        Ok(Self { db })
    }

    pub fn add_item(&self, item: Item) -> Result<()> {
        self.db.add_item(&item)
    }

    pub fn get_all_items(&self) -> Result<Vec<Item>> {
        self.db.get_all_items()
    }

    pub fn update_item(&self, item: Item) -> Result<()> {
        self.db.update_item(&item)
    }

    pub fn delete_item(&self, id: i64) -> Result<()> {
        self.db.delete_item(id)
    }

    /// 获取所有物品的日均成本统计
    pub fn get_daily_cost_summary(&self) -> Result<Vec<DailyCostSummary>> {
        let items = self.db.get_all_items()?;
        let today = Utc::now().date_naive();

        let mut summaries = Vec::new();
        for item in items {
            let daily_cost = item.daily_cost(today);
            let total_days = item.total_days(today);

            summaries.push(DailyCostSummary {
                item,
                daily_cost,
                total_days,
            });
        }

        Ok(summaries)
    }
}

#[derive(Debug)]
pub struct DailyCostSummary {
    pub item: Item,
    pub daily_cost: f64,
    pub total_days: i64,
}