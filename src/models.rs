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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_creation() {
        let start_date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let item = Item::new(
            "Test Item".to_string(),
            start_date,
            None,
            1000.0,
            "CNY".to_string(),
        );

        assert_eq!(item.name, "Test Item");
        assert_eq!(item.start_date, start_date);
        assert_eq!(item.discard_date, None);
        assert_eq!(item.price, 1000.0);
        assert_eq!(item.currency, "CNY");
        assert_eq!(item.id, None);
    }

    #[test]
    fn test_daily_cost_with_discard_date() {
        let start_date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let discard_date = NaiveDate::from_ymd_opt(2022, 1, 1).unwrap();
        let today = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();

        let item = Item::new(
            "Test Item".to_string(),
            start_date,
            Some(discard_date),
            3000.0,
            "CNY".to_string(),
        );

        // 应该使用弃用日期而不是今天来计算
        let expected_days = 731.0; // 2020-2022 是闰年+平年
        let expected_cost = 3000.0 / expected_days;

        assert!((item.daily_cost(today) - expected_cost).abs() < 0.0001);
    }

    #[test]
    fn test_daily_cost_without_discard_date() {
        let start_date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let today = NaiveDate::from_ymd_opt(2022, 1, 1).unwrap();

        let item = Item::new(
            "Test Item".to_string(),
            start_date,
            None,
            3000.0,
            "CNY".to_string(),
        );

        // 应该使用今天来计算
        let expected_days = 731.0; // 2020-2022 是闰年+平年
        let expected_cost = 3000.0 / expected_days;

        assert!((item.daily_cost(today) - expected_cost).abs() < 0.0001);
    }

    #[test]
    fn test_daily_cost_zero_days() {
        let start_date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let today = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();

        let item = Item::new(
            "Test Item".to_string(),
            start_date,
            None,
            1000.0,
            "CNY".to_string(),
        );

        // 当天购买，应该返回原价
        assert_eq!(item.daily_cost(today), 1000.0);
    }

    #[test]
    fn test_total_days() {
        let start_date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let discard_date = NaiveDate::from_ymd_opt(2020, 1, 10).unwrap();
        let today = NaiveDate::from_ymd_opt(2020, 1, 15).unwrap();

        let item_with_discard = Item::new(
            "Test Item".to_string(),
            start_date,
            Some(discard_date),
            1000.0,
            "CNY".to_string(),
        );

        let item_without_discard = Item::new(
            "Test Item".to_string(),
            start_date,
            None,
            1000.0,
            "CNY".to_string(),
        );

        // 有弃用日期的应该使用弃用日期
        assert_eq!(item_with_discard.total_days(today), 9);
        // 没有弃用日期的应该使用今天
        assert_eq!(item_without_discard.total_days(today), 14);
    }
}