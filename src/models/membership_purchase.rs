use crate::entities::{MemberType, MembershipPurchaseStatus, membership_purchase_entity as mp};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateMembershipIntentRequest {
    pub target_member_type: MemberType,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateMembershipIntentResponse {
    pub payment_intent_id: String,
    pub client_secret: String,
    /// Stripe Checkout 会话 URL（跳转到 Stripe 官方收银台）
    pub checkout_url: String,
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

impl From<mp::Model> for MembershipPurchaseRecordResponse {
    fn from(m: mp::Model) -> Self {
        Self {
            id: m.id,
            amount: m.amount,
            target_member_type: m.target_member_type,
            status: m.status,
            created_at: m.created_at.unwrap_or_else(Utc::now),
        }
    }
}
