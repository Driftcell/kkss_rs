use crate::entities::{RechargeStatus, recharge_record_entity};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreatePaymentIntentRequest {
    pub amount: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreatePaymentIntentResponse {
    pub payment_intent_id: String,
    pub client_secret: String,
    pub amount: i64,
    pub bonus_amount: i64,
    pub total_amount: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ConfirmRechargeRequest {
    pub payment_intent_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ConfirmRechargeResponse {
    pub recharge_record: RechargeRecordResponse,
    pub new_balance: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RechargeRecordResponse {
    pub id: i64,
    pub amount: i64,
    pub bonus_amount: i64,
    pub total_amount: i64,
    pub status: RechargeStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RechargeQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

// Convert from entity Model to API response directly for convenience
impl From<recharge_record_entity::Model> for RechargeRecordResponse {
    fn from(m: recharge_record_entity::Model) -> Self {
        Self {
            id: m.id,
            amount: m.amount,
            bonus_amount: m.bonus_amount,
            total_amount: m.total_amount,
            status: m.status,
            created_at: m.created_at.unwrap_or_else(Utc::now),
        }
    }
}
