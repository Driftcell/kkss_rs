use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{DeriveActiveEnum, EnumIter};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, DeriveActiveEnum, EnumIter, PartialEq, Eq, ToSchema)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "stripe_transaction_type")]
pub enum StripeTransactionType {
    #[sea_orm(string_value = "recharge")]
    Recharge,
    #[sea_orm(string_value = "membership")]
    Membership,
    #[sea_orm(string_value = "month_card")]
    MonthCard,
}

impl std::fmt::Display for StripeTransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StripeTransactionType::Recharge => write!(f, "recharge"),
            StripeTransactionType::Membership => write!(f, "membership"),
            StripeTransactionType::MonthCard => write!(f, "month_card"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, DeriveActiveEnum, EnumIter, PartialEq, Eq, ToSchema)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "stripe_transaction_status")]
pub enum StripeTransactionStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "succeeded")]
    Succeeded,
    #[sea_orm(string_value = "failed")]
    Failed,
    #[sea_orm(string_value = "canceled")]
    Canceled,
    #[sea_orm(string_value = "refunded")]
    Refunded,
}

impl std::fmt::Display for StripeTransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StripeTransactionStatus::Pending => write!(f, "pending"),
            StripeTransactionStatus::Succeeded => write!(f, "succeeded"),
            StripeTransactionStatus::Failed => write!(f, "failed"),
            StripeTransactionStatus::Canceled => write!(f, "canceled"),
            StripeTransactionStatus::Refunded => write!(f, "refunded"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "stripe_transactions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_id: i64,
    pub stripe_payment_intent_id: String,
    pub transaction_type: StripeTransactionType,
    pub amount: i64,
    pub status: StripeTransactionStatus,
    pub metadata: Option<Value>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}