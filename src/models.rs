use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: Option<i64>,
    pub name: String,
    pub start_date: NaiveDate,
    pub discard_date: Option<NaiveDate>,
    pub price: f64,
    pub currency: String,
}

impl Item {
    pub fn new(
        name: String,
        start_date: NaiveDate,
        discard_date: Option<NaiveDate>,
        price: f64,
        currency: String,
    ) -> Self {
        Self {
            id: None,
            name,
            start_date,
            discard_date,
            price,
            currency,
        }
    }

    /// 计算日均成本
    pub fn daily_cost(&self, today: NaiveDate) -> f64 {
        let end_date = self.discard_date.unwrap_or(today);
        let days = (end_date - self.start_date).num_days() as f64;

        if days <= 0.0 {
            self.price
        } else {
            self.price / days
        }
    }

    /// 计算总使用天数
    pub fn total_days(&self, today: NaiveDate) -> i64 {
        let end_date = self.discard_date.unwrap_or(today);
        (end_date - self.start_date).num_days()
    }
}