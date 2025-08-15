use crate::models::CodeType;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "discount_codes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_id: i64,
    pub code: String,
    pub discount_amount: i64,
    pub code_type: CodeType,
    pub is_used: Option<bool>,
    pub used_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub external_id: Option<i64>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
