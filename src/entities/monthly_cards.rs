use chrono::{DateTime, NaiveDate, Utc};
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
    enum_name = "monthly_card_plan_type"
)]
#[serde(rename_all = "snake_case")]
pub enum MonthlyCardPlanType {
    #[sea_orm(string_value = "one_time")]
    OneTime,
    #[sea_orm(string_value = "subscription")]
    Subscription,
}

impl std::fmt::Display for MonthlyCardPlanType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonthlyCardPlanType::OneTime => write!(f, "one_time"),
            MonthlyCardPlanType::Subscription => write!(f, "subscription"),
        }
    }
}

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, DeriveActiveEnum, EnumIter,
)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "monthly_card_status"
)]
#[serde(rename_all = "snake_case")]
pub enum MonthlyCardStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "canceled")]
    Canceled,
    #[sea_orm(string_value = "expired")]
    Expired,
}

impl std::fmt::Display for MonthlyCardStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonthlyCardStatus::Pending => write!(f, "pending"),
            MonthlyCardStatus::Active => write!(f, "active"),
            MonthlyCardStatus::Canceled => write!(f, "canceled"),
            MonthlyCardStatus::Expired => write!(f, "expired"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "monthly_cards")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_id: i64,
    pub plan_type: MonthlyCardPlanType,
    pub status: MonthlyCardStatus,
    pub stripe_subscription_id: Option<String>,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub last_coupon_granted_on: Option<NaiveDate>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
