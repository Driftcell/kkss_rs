use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

use crate::models::MemberType;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, ToSchema)]
#[sqlx(type_name = "membership_purchase_status", rename_all = "snake_case")]
pub enum MembershipPurchaseStatus {
    Pending,
    Succeeded,
    Failed,
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct MembershipPurchaseRecord {
    pub id: Option<i64>,
    pub user_id: Option<i64>,
    pub stripe_payment_intent_id: String,
    pub target_member_type: MemberType,
    pub amount: i64,
    pub status: MembershipPurchaseStatus,
    pub stripe_status: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateMembershipIntentRequest {
    pub target_member_type: MemberType,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateMembershipIntentResponse {
    pub payment_intent_id: String,
    pub client_secret: String,
    pub amount: i64,
    pub target_member_type: MemberType,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ConfirmMembershipRequest {
    pub payment_intent_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ConfirmMembershipResponse {
    pub membership_record: MembershipPurchaseRecordResponse,
    pub new_member_type: MemberType,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MembershipPurchaseRecordResponse {
    pub id: i64,
    pub amount: i64,
    pub target_member_type: MemberType,
    pub status: MembershipPurchaseStatus,
    pub created_at: DateTime<Utc>,
}

impl From<MembershipPurchaseRecord> for MembershipPurchaseRecordResponse {
    fn from(r: MembershipPurchaseRecord) -> Self {
        Self {
            id: r.id.unwrap_or(0),
            amount: r.amount,
            target_member_type: r.target_member_type,
            status: r.status,
            created_at: r.created_at.unwrap_or_else(|| Utc::now()),
        }
    }
}
