use crate::entities::{stripe_transaction_entity as st, StripeTransactionStatus, StripeTransactionType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StripeTransactionResponse {
    pub id: i64,
    pub user_id: i64,
    pub stripe_payment_intent_id: String,
    pub transaction_type: StripeTransactionType,
    pub amount: i64,
    pub status: StripeTransactionStatus,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
}

impl From<st::Model> for StripeTransactionResponse {
    fn from(m: st::Model) -> Self {
        Self {
            id: m.id,
            user_id: m.user_id,
            stripe_payment_intent_id: m.stripe_payment_intent_id,
            transaction_type: m.transaction_type,
            amount: m.amount,
            status: m.status,
            metadata: m.metadata,
            created_at: m.created_at.unwrap_or_else(Utc::now),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateStripeTransactionRequest {
    pub transaction_type: StripeTransactionType,
    pub amount: i64,
    pub metadata: Option<Value>,
}