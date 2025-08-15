use chrono::{DateTime, Utc};
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult, ToSchema)]
pub struct Order {
    pub id: Option<i64>, // 七云订单ID
    pub user_id: Option<i64>,
    pub member_code: Option<String>,
    pub price: Option<i64>, // 订单金额(美分)
    pub product_name: String,
    pub product_no: Option<String>,
    pub order_status: Option<i64>,                  // 修改为 i64
    pub pay_type: Option<i64>,                      // 修改为 i64
    pub stamps_earned: Option<i64>,                 // 获得的 Stamps
    pub external_created_at: Option<DateTime<Utc>>, // 七云订单创建时间
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrderResponse {
    pub id: i64,
    pub product_name: String,
    pub price: i64,
    pub stamps_earned: i64,
    pub order_status: i32,
    pub external_created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrderQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub status: Option<i32>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

impl From<Order> for OrderResponse {
    fn from(order: Order) -> Self {
        Self {
            id: order.id.unwrap_or(0),
            product_name: order.product_name,
            price: order.price.unwrap_or(0),
            stamps_earned: order.stamps_earned.unwrap_or(0),
            order_status: order.order_status.unwrap_or(0) as i32, // 转换为 i32
            external_created_at: order.external_created_at.unwrap_or_else(Utc::now),
        }
    }
}
