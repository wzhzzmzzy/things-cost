use crate::models::Item;
use crate::services::ItemService;
use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use rusqlite::Result;
use tabled::{Table, Tabled};

#[derive(Parser)]
#[command(name = "things-cost")]
#[command(about = "物品日均花费统计工具", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 添加新物品
    Add {
        /// 物品名称
        name: String,
        /// 起始日期 (YYYY-MM-DD)
        start_date: String,
        /// 价格
        price: f64,
        /// 币种，默认为 CNY
        #[arg(default_value = "CNY")]
        currency: String,
        /// 弃用日期 (YYYY-MM-DD)，可选
        discard_date: Option<String>,
    },
    /// 列出所有物品及其日均成本
    List,
    /// 更新物品信息
    Update {
        /// 物品 ID
        id: i64,
        /// 物品名称
        name: Option<String>,
        /// 起始日期 (YYYY-MM-DD)
        start_date: Option<String>,
        /// 弃用日期 (YYYY-MM-DD)
        discard_date: Option<String>,
        /// 价格
        price: Option<f64>,
        /// 币种
        currency: Option<String>,
    },
    /// 删除物品
    Delete {
        /// 物品 ID
        id: i64,
    },
}

#[derive(Tabled)]
pub struct ItemTable {
    #[tabled(rename = "ID")]
    pub id: String,
    #[tabled(rename = "名称")]
    pub name: String,
    #[tabled(rename = "起始日期")]
    pub start_date: String,
    #[tabled(rename = "弃用日期")]
    pub discard_date: String,
    #[tabled(rename = "价格")]
    pub price: String,
    #[tabled(rename = "使用天数")]
    pub total_days: String,
    #[tabled(rename = "日均成本")]
    pub daily_cost: String,
}

pub struct CliHandler {
    service: ItemService,
}

impl CliHandler {
    pub fn new() -> Result<Self> {
        let service = ItemService::new()?;
        Ok(Self { service })
    }

    /// 用于测试的构造函数
    #[cfg(test)]
    pub fn new_with_service(service: ItemService) -> Self {
        Self { service }
    }

    pub fn handle_command(&self, command: Commands) -> Result<()> {
        match command {
            Commands::Add {
                name,
                start_date,
                price,
                currency,
                discard_date,
            } => self.handle_add(name, start_date, price, currency, discard_date),
            Commands::List => self.handle_list(),
            Commands::Update {
                id,
                name,
                start_date,
                discard_date,
                price,
                currency,
            } => self.handle_update(id, name, start_date, discard_date, price, currency),
            Commands::Delete { id } => self.handle_delete(id),
        }
    }

    fn handle_add(
        &self,
        name: String,
        start_date: String,
        price: f64,
        currency: String,
        discard_date: Option<String>,
    ) -> Result<()> {
        let start_date = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

        let discard_date = if let Some(date_str) = discard_date {
            Some(
                NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
            )
        } else {
            None
        };

        let item = Item::new(name, start_date, discard_date, price, currency);
        self.service.add_item(item)?;

        println!("物品添加成功！");
        Ok(())
    }

    fn handle_list(&self) -> Result<()> {
        let summaries = self.service.get_daily_cost_summary()?;

        if summaries.is_empty() {
            println!("暂无物品记录");
            return Ok(());
        }

        let table_data: Vec<ItemTable> = summaries
            .into_iter()
            .map(|summary| {
                let id = summary
                    .item
                    .id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "-".to_string());

                let discard_date = summary
                    .item
                    .discard_date
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "-".to_string());

                ItemTable {
                    id,
                    name: summary.item.name,
                    start_date: summary.item.start_date.to_string(),
                    discard_date,
                    price: format!("{:.2} {}", summary.item.price, summary.item.currency),
                    total_days: summary.total_days.to_string(),
                    daily_cost: format!("{:.4} {}", summary.daily_cost, summary.item.currency),
                }
            })
            .collect();

        let table = Table::new(table_data).to_string();
        println!("{}", table);
        Ok(())
    }

    fn handle_update(
        &self,
        id: i64,
        name: Option<String>,
        start_date: Option<String>,
        discard_date: Option<String>,
        price: Option<f64>,
        currency: Option<String>,
    ) -> Result<()> {
        let items = self.service.get_all_items()?;
        let mut item_to_update = items
            .into_iter()
            .find(|item| item.id == Some(id))
            .ok_or_else(|| rusqlite::Error::InvalidParameterName("物品不存在".to_string()))?;

        if let Some(name) = name {
            item_to_update.name = name;
        }

        if let Some(start_date) = start_date {
            item_to_update.start_date = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        }

        if let Some(discard_date) = discard_date {
            item_to_update.discard_date = Some(
                NaiveDate::parse_from_str(&discard_date, "%Y-%m-%d")
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
            );
        }

        if let Some(price) = price {
            item_to_update.price = price;
        }

        if let Some(currency) = currency {
            item_to_update.currency = currency;
        }

        self.service.update_item(item_to_update)?;
        println!("物品更新成功！");
        Ok(())
    }

    fn handle_delete(&self, id: i64) -> Result<()> {
        self.service.delete_item(id)?;
        println!("物品删除成功！");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;
    use crate::services::ItemService;

    #[test]
    fn test_cli_handler_creation() -> Result<()> {
        let db = Database::new_in_memory()?;
        let service = ItemService::new_with_db(db);
        let handler = CliHandler::new_with_service(service);

        // 验证 handler 创建成功
        assert!(std::mem::size_of_val(&handler) > 0);
        Ok(())
    }

    #[test]
    fn test_handle_add() -> Result<()> {
        let db = Database::new_in_memory()?;
        let service = ItemService::new_with_db(db);
        let handler = CliHandler::new_with_service(service);

        let command = Commands::Add {
            name: "Test Item".to_string(),
            start_date: "2020-01-01".to_string(),
            price: 1000.0,
            currency: "CNY".to_string(),
            discard_date: None,
        };

        // 应该成功执行
        handler.handle_command(command)?;

        // 验证物品已添加
        let items = handler.service.get_all_items()?;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Test Item");

        Ok(())
    }

    #[test]
    fn test_handle_add_with_discard_date() -> Result<()> {
        let db = Database::new_in_memory()?;
        let service = ItemService::new_with_db(db);
        let handler = CliHandler::new_with_service(service);

        let command = Commands::Add {
            name: "Test Item".to_string(),
            start_date: "2020-01-01".to_string(),
            price: 1000.0,
            currency: "CNY".to_string(),
            discard_date: Some("2022-01-01".to_string()),
        };

        // 应该成功执行
        handler.handle_command(command)?;

        // 验证物品已添加
        let items = handler.service.get_all_items()?;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Test Item");
        assert_eq!(
            items[0].discard_date,
            Some(NaiveDate::from_ymd_opt(2022, 1, 1).unwrap())
        );

        Ok(())
    }

    #[test]
    fn test_handle_list_empty() -> Result<()> {
        let db = Database::new_in_memory()?;
        let service = ItemService::new_with_db(db);
        let handler = CliHandler::new_with_service(service);

        let command = Commands::List;

        // 应该成功执行（空列表）
        handler.handle_command(command)?;

        Ok(())
    }

    #[test]
    fn test_handle_list_with_items() -> Result<()> {
        let db = Database::new_in_memory()?;
        let service = ItemService::new_with_db(db);
        let handler = CliHandler::new_with_service(service);

        // 先添加一个物品
        let add_command = Commands::Add {
            name: "Test Item".to_string(),
            start_date: "2020-01-01".to_string(),
            price: 1000.0,
            currency: "CNY".to_string(),
            discard_date: None,
        };
        handler.handle_command(add_command)?;

        let list_command = Commands::List;

        // 应该成功执行
        handler.handle_command(list_command)?;

        Ok(())
    }

    #[test]
    fn test_handle_delete() -> Result<()> {
        let db = Database::new_in_memory()?;
        let service = ItemService::new_with_db(db);
        let handler = CliHandler::new_with_service(service);

        // 先添加一个物品
        let add_command = Commands::Add {
            name: "Test Item".to_string(),
            start_date: "2020-01-01".to_string(),
            price: 1000.0,
            currency: "CNY".to_string(),
            discard_date: None,
        };
        handler.handle_command(add_command)?;

        // 获取物品 ID
        let items = handler.service.get_all_items()?;
        let id = items[0].id.unwrap();

        let delete_command = Commands::Delete { id };

        // 应该成功执行
        handler.handle_command(delete_command)?;

        // 验证物品已删除
        let items = handler.service.get_all_items()?;
        assert_eq!(items.len(), 0);

        Ok(())
    }

    #[test]
    fn test_handle_update() -> Result<()> {
        let db = Database::new_in_memory()?;
        let service = ItemService::new_with_db(db);
        let handler = CliHandler::new_with_service(service);

        // 先添加一个物品
        let add_command = Commands::Add {
            name: "Test Item".to_string(),
            start_date: "2020-01-01".to_string(),
            price: 1000.0,
            currency: "CNY".to_string(),
            discard_date: None,
        };
        handler.handle_command(add_command)?;

        // 获取物品 ID
        let items = handler.service.get_all_items()?;
        let id = items[0].id.unwrap();

        let update_command = Commands::Update {
            id,
            name: Some("Updated Item".to_string()),
            start_date: None,
            discard_date: None,
            price: Some(2000.0),
            currency: None,
        };

        // 应该成功执行
        handler.handle_command(update_command)?;

        // 验证物品已更新
        let items = handler.service.get_all_items()?;
        assert_eq!(items[0].name, "Updated Item");
        assert_eq!(items[0].price, 2000.0);

        Ok(())
    }

    #[test]
    fn test_handle_add_invalid_date() {
        let db = Database::new_in_memory().unwrap();
        let service = ItemService::new_with_db(db);
        let handler = CliHandler::new_with_service(service);

        let command = Commands::Add {
            name: "Test Item".to_string(),
            start_date: "invalid-date".to_string(), // 无效日期
            price: 1000.0,
            currency: "CNY".to_string(),
            discard_date: None,
        };

        // 应该失败，因为日期格式无效
        let result = handler.handle_command(command);
        assert!(result.is_err());
    }
}
