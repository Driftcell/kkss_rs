use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "orders")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_id: i64,
    pub member_code: Option<String>,
    pub price: i64,
    pub product_name: String,
    pub product_no: Option<String>,
    pub order_status: i32,
    pub pay_type: Option<i32>,
    pub stamps_earned: Option<i64>,
    pub external_created_at: DateTime<Utc>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
