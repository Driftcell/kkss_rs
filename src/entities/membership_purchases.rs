use crate::entities::MemberType;
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
    enum_name = "membership_purchase_status"
)]
#[serde(rename_all = "snake_case")]
pub enum MembershipPurchaseStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "succeeded")]
    Succeeded,
    #[sea_orm(string_value = "failed")]
    Failed,
    #[sea_orm(string_value = "canceled")]
    Canceled,
}

impl std::fmt::Display for MembershipPurchaseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MembershipPurchaseStatus::Pending => write!(f, "pending"),
            MembershipPurchaseStatus::Succeeded => write!(f, "succeeded"),
            MembershipPurchaseStatus::Failed => write!(f, "failed"),
            MembershipPurchaseStatus::Canceled => write!(f, "canceled"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "membership_purchases")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_id: i64,
    pub stripe_payment_intent_id: String,
    pub target_member_type: MemberType,
    pub amount: i64,
    pub status: MembershipPurchaseStatus,
    pub stripe_status: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
