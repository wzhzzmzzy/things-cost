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
        /// 要更新的字段
        field: String,
        /// 物品 ID
        id: i64,
        /// 新值
        value: String,
    },
    /// 删除物品
    Delete {
        /// 物品 ID
        id: i64,
    },
    /// 弃用物品
    Discard {
        /// 物品 ID
        id: i64,
        /// 弃用日期 (YYYY-MM-DD)
        discard_date: String,
        /// 卖出价格，可选
        selling_price: Option<f64>,
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
            Commands::Update { field, id, value } => self.handle_update(field, id, value),
            Commands::Delete { id } => self.handle_delete(id),
            Commands::Discard {
                id,
                discard_date,
                selling_price,
            } => self.handle_discard(id, discard_date, selling_price),
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

                // 计算价格显示
                let price_display = if let Some(selling_price) = summary.item.selling_price {
                    if selling_price > 0.0 {
                        let effective_price = summary.item.price - selling_price;
                        format!(
                            "{:.2} {} (购入: {:.2}, 卖出: {:.2})",
                            effective_price,
                            summary.item.currency,
                            summary.item.price,
                            selling_price
                        )
                    } else {
                        // 卖出价格为0或负数时，显示原始价格
                        format!("{:.2} {}", summary.item.price, summary.item.currency)
                    }
                } else {
                    format!("{:.2} {}", summary.item.price, summary.item.currency)
                };

                ItemTable {
                    id,
                    name: summary.item.name,
                    start_date: summary.item.start_date.to_string(),
                    discard_date,
                    price: price_display,
                    total_days: summary.total_days.to_string(),
                    daily_cost: format!("{:.4} {}", summary.daily_cost, summary.item.currency),
                }
            })
            .collect();

        let table = Table::new(table_data).to_string();
        println!("{}", table);
        Ok(())
    }

    fn handle_update(&self, field: String, id: i64, value: String) -> Result<()> {
        let items = self.service.get_all_items()?;
        let mut item_to_update = items
            .into_iter()
            .find(|item| item.id == Some(id))
            .ok_or_else(|| rusqlite::Error::InvalidParameterName("物品不存在".to_string()))?;

        match field.to_lowercase().as_str() {
            "name" => {
                item_to_update.name = value;
            }
            "start_date" => {
                item_to_update.start_date = NaiveDate::parse_from_str(&value, "%Y-%m-%d")
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
            }
            "discard_date" => {
                let parsed_date = NaiveDate::parse_from_str(&value, "%Y-%m-%d")
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

                // 验证弃用日期不能早于起始日期
                if parsed_date < item_to_update.start_date {
                    return Err(rusqlite::Error::InvalidParameterName(
                        "弃用日期不能早于起始日期".to_string(),
                    ));
                }

                item_to_update.discard_date = Some(parsed_date);
            }
            "price" => {
                let price = value
                    .parse::<f64>()
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
                if price <= 0.0 {
                    return Err(rusqlite::Error::InvalidParameterName(
                        "价格必须大于0".to_string(),
                    ));
                }
                item_to_update.price = price;
            }
            "currency" => {
                item_to_update.currency = value;
            }
            "selling_price" => {
                if value.trim().is_empty() || value.to_lowercase() == "null" {
                    // 清空卖出价格
                    item_to_update.selling_price = None;
                } else {
                    let selling_price = value
                        .parse::<f64>()
                        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

                    // 验证卖出价格不能大于购买价格
                    if selling_price > item_to_update.price {
                        return Err(rusqlite::Error::InvalidParameterName(
                            "卖出价格不能大于购买价格".to_string(),
                        ));
                    }

                    // 验证卖出价格不能为负数
                    if selling_price < 0.0 {
                        return Err(rusqlite::Error::InvalidParameterName(
                            "卖出价格不能为负数".to_string(),
                        ));
                    }

                    item_to_update.selling_price = Some(selling_price);
                }
            }
            _ => {
                return Err(rusqlite::Error::InvalidParameterName(format!(
                    "不支持的字段: {}",
                    field
                )));
            }
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

    fn handle_discard(
        &self,
        id: i64,
        discard_date: String,
        selling_price: Option<f64>,
    ) -> Result<()> {
        let items = self.service.get_all_items()?;
        let mut item_to_update = items
            .into_iter()
            .find(|item| item.id == Some(id))
            .ok_or_else(|| rusqlite::Error::InvalidParameterName("物品不存在".to_string()))?;

        let parsed_discard_date = NaiveDate::parse_from_str(&discard_date, "%Y-%m-%d")
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

        // 验证弃用日期不能早于起始日期
        if parsed_discard_date < item_to_update.start_date {
            return Err(rusqlite::Error::InvalidParameterName(
                "弃用日期不能早于起始日期".to_string(),
            ));
        }

        item_to_update.discard_date = Some(parsed_discard_date);
        item_to_update.selling_price = selling_price;
        self.service.update_item(item_to_update)?;

        println!("物品弃用成功！");
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

        // 更新名称
        let update_name_command = Commands::Update {
            field: "name".to_string(),
            id,
            value: "Updated Item".to_string(),
        };
        handler.handle_command(update_name_command)?;

        // 更新价格
        let update_price_command = Commands::Update {
            field: "price".to_string(),
            id,
            value: "2000.0".to_string(),
        };
        handler.handle_command(update_price_command)?;

        // 更新卖出价格
        let update_selling_price_command = Commands::Update {
            field: "selling_price".to_string(),
            id,
            value: "500.0".to_string(),
        };
        handler.handle_command(update_selling_price_command)?;

        // 验证物品已更新
        let items = handler.service.get_all_items()?;
        assert_eq!(items[0].name, "Updated Item");
        assert_eq!(items[0].price, 2000.0);
        assert_eq!(items[0].selling_price, Some(500.0));

        // 清空卖出价格
        let clear_selling_price_command = Commands::Update {
            field: "selling_price".to_string(),
            id,
            value: "".to_string(),
        };
        handler.handle_command(clear_selling_price_command)?;

        // 验证卖出价格已清空
        let items = handler.service.get_all_items()?;
        assert_eq!(items[0].selling_price, None);

        Ok(())
    }

    #[test]
    fn test_handle_update_invalid_field() {
        let db = Database::new_in_memory().unwrap();
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
        handler.handle_command(add_command).unwrap();

        // 获取物品 ID
        let items = handler.service.get_all_items().unwrap();
        let id = items[0].id.unwrap();

        let update_command = Commands::Update {
            field: "invalid_field".to_string(),
            id,
            value: "some value".to_string(),
        };

        // 应该失败，因为字段不存在
        let result = handler.handle_command(update_command);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_update_invalid_price() {
        let db = Database::new_in_memory().unwrap();
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
        handler.handle_command(add_command).unwrap();

        // 获取物品 ID
        let items = handler.service.get_all_items().unwrap();
        let id = items[0].id.unwrap();

        let update_command = Commands::Update {
            field: "price".to_string(),
            id,
            value: "-100".to_string(), // 无效价格
        };

        // 应该失败，因为价格必须大于0
        let result = handler.handle_command(update_command);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_update_discard_date_earlier_than_start() {
        let db = Database::new_in_memory().unwrap();
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
        handler.handle_command(add_command).unwrap();

        // 获取物品 ID
        let items = handler.service.get_all_items().unwrap();
        let id = items[0].id.unwrap();

        let update_command = Commands::Update {
            field: "discard_date".to_string(),
            id,
            value: "2019-12-31".to_string(), // 早于起始日期
        };

        // 应该失败，因为弃用日期不能早于起始日期
        let result = handler.handle_command(update_command);
        assert!(result.is_err());
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

    #[test]
    fn test_handle_discard() -> Result<()> {
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

        let discard_command = Commands::Discard {
            id,
            discard_date: "2022-01-01".to_string(),
            selling_price: None,
        };

        // 应该成功执行
        handler.handle_command(discard_command)?;

        // 验证物品已弃用
        let items = handler.service.get_all_items()?;
        assert_eq!(
            items[0].discard_date,
            Some(NaiveDate::from_ymd_opt(2022, 1, 1).unwrap())
        );

        Ok(())
    }

    #[test]
    fn test_handle_discard_invalid_date() {
        let db = Database::new_in_memory().unwrap();
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
        handler.handle_command(add_command).unwrap();

        // 获取物品 ID
        let items = handler.service.get_all_items().unwrap();
        let id = items[0].id.unwrap();

        let discard_command = Commands::Discard {
            id,
            discard_date: "invalid-date".to_string(), // 无效日期
            selling_price: None,
        };

        // 应该失败，因为日期格式无效
        let result = handler.handle_command(discard_command);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_discard_earlier_than_start_date() {
        let db = Database::new_in_memory().unwrap();
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
        handler.handle_command(add_command).unwrap();

        // 获取物品 ID
        let items = handler.service.get_all_items().unwrap();
        let id = items[0].id.unwrap();

        let discard_command = Commands::Discard {
            id,
            discard_date: "2019-12-31".to_string(), // 早于起始日期
            selling_price: None,
        };

        // 应该失败，因为弃用日期早于起始日期
        let result = handler.handle_command(discard_command);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_discard_nonexistent_item() {
        let db = Database::new_in_memory().unwrap();
        let service = ItemService::new_with_db(db);
        let handler = CliHandler::new_with_service(service);

        let discard_command = Commands::Discard {
            id: 999, // 不存在的物品 ID
            discard_date: "2022-01-01".to_string(),
            selling_price: None,
        };

        // 应该失败，因为物品不存在
        let result = handler.handle_command(discard_command);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_discard_with_selling_price() -> Result<()> {
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

        let discard_command = Commands::Discard {
            id,
            discard_date: "2022-01-01".to_string(),
            selling_price: Some(500.0),
        };

        // 应该成功执行
        handler.handle_command(discard_command)?;

        // 验证物品已弃用且卖出价格已设置
        let items = handler.service.get_all_items()?;
        assert_eq!(
            items[0].discard_date,
            Some(NaiveDate::from_ymd_opt(2022, 1, 1).unwrap())
        );
        assert_eq!(items[0].selling_price, Some(500.0));

        Ok(())
    }

    #[test]
    fn test_handle_update_selling_price_higher_than_price() {
        let db = Database::new_in_memory().unwrap();
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
        handler.handle_command(add_command).unwrap();

        // 获取物品 ID
        let items = handler.service.get_all_items().unwrap();
        let id = items[0].id.unwrap();

        let update_command = Commands::Update {
            field: "selling_price".to_string(),
            id,
            value: "1500.0".to_string(), // 大于购买价格
        };

        // 应该失败，因为卖出价格大于购买价格
        let result = handler.handle_command(update_command);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_update_selling_price_negative() {
        let db = Database::new_in_memory().unwrap();
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
        handler.handle_command(add_command).unwrap();

        // 获取物品 ID
        let items = handler.service.get_all_items().unwrap();
        let id = items[0].id.unwrap();

        let update_command = Commands::Update {
            field: "selling_price".to_string(),
            id,
            value: "-100.0".to_string(), // 负数
        };

        // 应该失败，因为卖出价格不能为负数
        let result = handler.handle_command(update_command);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_update_selling_price_invalid_format() {
        let db = Database::new_in_memory().unwrap();
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
        handler.handle_command(add_command).unwrap();

        // 获取物品 ID
        let items = handler.service.get_all_items().unwrap();
        let id = items[0].id.unwrap();

        let update_command = Commands::Update {
            field: "selling_price".to_string(),
            id,
            value: "not_a_number".to_string(), // 无效格式
        };

        // 应该失败，因为格式无效
        let result = handler.handle_command(update_command);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_update_selling_price_null() -> Result<()> {
        let db = Database::new_in_memory()?;
        let service = ItemService::new_with_db(db);
        let handler = CliHandler::new_with_service(service);

        // 先添加一个物品并设置卖出价格
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

        // 先设置卖出价格
        let set_selling_price_command = Commands::Update {
            field: "selling_price".to_string(),
            id,
            value: "500.0".to_string(),
        };
        handler.handle_command(set_selling_price_command)?;

        // 验证卖出价格已设置
        let items = handler.service.get_all_items()?;
        assert_eq!(items[0].selling_price, Some(500.0));

        // 使用 "null" 清空卖出价格
        let clear_selling_price_command = Commands::Update {
            field: "selling_price".to_string(),
            id,
            value: "null".to_string(),
        };
        handler.handle_command(clear_selling_price_command)?;

        // 验证卖出价格已清空
        let items = handler.service.get_all_items()?;
        assert_eq!(items[0].selling_price, None);

        Ok(())
    }
}
