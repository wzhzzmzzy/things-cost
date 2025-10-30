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

    /// 用于测试的构造函数
    #[cfg(test)]
    pub fn new_with_db(db: Database) -> Self {
        Self { db }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;
    use chrono::NaiveDate;

    #[test]
    fn test_add_and_get_items() -> Result<()> {
        let db = Database::new_in_memory()?;
        let service = ItemService::new_with_db(db);

        let item = Item::new(
            "Test Item".to_string(),
            NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            None,
            1000.0,
            "CNY".to_string(),
        );

        // 添加物品
        service.add_item(item)?;

        // 获取所有物品
        let items = service.get_all_items()?;

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Test Item");

        Ok(())
    }

    #[test]
    fn test_update_item() -> Result<()> {
        let db = Database::new_in_memory()?;
        let service = ItemService::new_with_db(db);

        let mut item = Item::new(
            "Test Item".to_string(),
            NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            None,
            1000.0,
            "CNY".to_string(),
        );

        // 添加物品
        service.add_item(item.clone())?;
        let items = service.get_all_items()?;
        let id = items[0].id.unwrap();

        // 更新物品
        item.id = Some(id);
        item.name = "Updated Item".to_string();
        service.update_item(item)?;

        // 验证更新
        let items = service.get_all_items()?;
        assert_eq!(items[0].name, "Updated Item");

        Ok(())
    }

    #[test]
    fn test_delete_item() -> Result<()> {
        let db = Database::new_in_memory()?;
        let service = ItemService::new_with_db(db);

        let item = Item::new(
            "Test Item".to_string(),
            NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            None,
            1000.0,
            "CNY".to_string(),
        );

        // 添加物品
        service.add_item(item)?;
        let items = service.get_all_items()?;
        let id = items[0].id.unwrap();

        // 删除物品
        service.delete_item(id)?;

        // 验证删除
        let items = service.get_all_items()?;
        assert_eq!(items.len(), 0);

        Ok(())
    }

    #[test]
    fn test_get_daily_cost_summary() -> Result<()> {
        let db = Database::new_in_memory()?;
        let service = ItemService::new_with_db(db);

        let item1 = Item::new(
            "Item 1".to_string(),
            NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            None,
            1000.0,
            "CNY".to_string(),
        );

        let item2 = Item::new(
            "Item 2".to_string(),
            NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            Some(NaiveDate::from_ymd_opt(2022, 1, 1).unwrap()),
            2000.0,
            "CNY".to_string(),
        );

        // 添加物品
        service.add_item(item1)?;
        service.add_item(item2)?;

        // 获取日均成本统计
        let summaries = service.get_daily_cost_summary()?;

        assert_eq!(summaries.len(), 2);

        // 验证统计信息
        for summary in summaries {
            assert!(summary.daily_cost > 0.0);
            assert!(summary.total_days > 0);
            assert!(!summary.item.name.is_empty());
        }

        Ok(())
    }

    #[test]
    fn test_get_daily_cost_summary_empty() -> Result<()> {
        let db = Database::new_in_memory()?;
        let service = ItemService::new_with_db(db);

        let summaries = service.get_daily_cost_summary()?;
        assert_eq!(summaries.len(), 0);

        Ok(())
    }
}