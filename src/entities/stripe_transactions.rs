use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{DeriveActiveEnum, EnumIter};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, DeriveActiveEnum, EnumIter,
)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "stripe_transaction_category"
)]
#[serde(rename_all = "snake_case")]
pub enum StripeTransactionCategory {
    #[sea_orm(string_value = "recharge")]
    Recharge,
    #[sea_orm(string_value = "membership")]
    Membership,
    #[sea_orm(string_value = "monthly_card")]
    MonthlyCard,
}

impl std::fmt::Display for StripeTransactionCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StripeTransactionCategory::Recharge => write!(f, "recharge"),
            StripeTransactionCategory::Membership => write!(f, "membership"),
            StripeTransactionCategory::MonthlyCard => write!(f, "monthly_card"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "stripe_transactions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_id: i64,
    pub category: StripeTransactionCategory,
    pub payment_intent_id: Option<String>,
    pub charge_id: Option<String>,
    pub refund_id: Option<String>,
    pub subscription_id: Option<String>,
    pub invoice_id: Option<String>,
    pub amount: Option<i64>,
    pub currency: Option<String>,
    pub status: Option<String>,
    pub description: Option<String>,
    pub raw_event: Option<Json>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
