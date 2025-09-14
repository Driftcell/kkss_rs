use crate::entities::order_entity;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrderResponse {
    pub id: i64,
    pub product_name: String,
    pub price: i64,
    pub stamps_earned: i64,
    pub order_status: i32,
    pub external_created_at: DateTime<Utc>,
    /// 通过 sweet_cash_transactions (transaction_type = 'earn' 且 related_order_id = 本订单 id) 汇总得到的甜品现金收益 (美分)
    /// 在基础查询中默认填充为 0，调用方可在 service 层额外补充
    #[schema(example = 1200)]
    pub sweet_cash_earned: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrderQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub status: Option<i32>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

impl From<order_entity::Model> for OrderResponse {
    fn from(m: order_entity::Model) -> Self {
        Self {
            id: m.id,
            product_name: m.product_name,
            price: m.price,
            stamps_earned: m.stamps_earned.unwrap_or(0),
            order_status: m.order_status,
            external_created_at: m.external_created_at,
            sweet_cash_earned: 0,
        }
    }
}
