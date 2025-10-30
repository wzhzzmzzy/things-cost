use crate::models::Item;
use crate::services::{DailyCostSummary, ItemService};
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
        /// 弃用日期 (YYYY-MM-DD)，可选
        discard_date: Option<String>,
        /// 价格
        price: f64,
        /// 币种，默认为 CNY
        #[arg(default_value = "CNY")]
        currency: String,
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

    pub fn handle_command(&self, command: Commands) -> Result<()> {
        match command {
            Commands::Add {
                name,
                start_date,
                discard_date,
                price,
                currency,
            } => self.handle_add(name, start_date, discard_date, price, currency),
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
        discard_date: Option<String>,
        price: f64,
        currency: String,
    ) -> Result<()> {
        let start_date = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

        let discard_date = if let Some(date_str) = discard_date {
            Some(NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?)
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
            item_to_update.discard_date = Some(NaiveDate::parse_from_str(&discard_date, "%Y-%m-%d")
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?);
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