use crate::entities::{month_card_entity as mc, StripeTransactionType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MonthCardResponse {
    pub id: i64,
    pub user_id: i64,
    pub subscription_id: Option<String>,
    pub product_id: String,
    pub price_id: String,
    pub is_active: bool,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub cancel_at_period_end: bool,
    pub created_at: DateTime<Utc>,
}

impl From<mc::Model> for MonthCardResponse {
    fn from(m: mc::Model) -> Self {
        Self {
            id: m.id,
            user_id: m.user_id,
            subscription_id: m.subscription_id,
            product_id: m.product_id,
            price_id: m.price_id,
            is_active: m.is_active,
            start_date: m.start_date,
            end_date: m.end_date,
            cancel_at_period_end: m.cancel_at_period_end,
            created_at: m.created_at.unwrap_or_else(Utc::now),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateMonthCardIntentRequest {
    pub is_subscription: bool, // true for subscription, false for one-time
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateMonthCardIntentResponse {
    pub payment_intent_id: String,
    pub client_secret: String,
    pub amount: i64,
    pub is_subscription: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ConfirmMonthCardRequest {
    pub payment_intent_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ConfirmMonthCardResponse {
    pub month_card: MonthCardResponse,
    pub transaction: crate::models::stripe_transaction::StripeTransactionResponse,
}

// Unified confirm request/response for all payment types
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UnifiedConfirmRequest {
    pub payment_intent_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UnifiedConfirmResponse {
    pub transaction_type: StripeTransactionType,
    pub transaction: crate::models::stripe_transaction::StripeTransactionResponse,
    pub details: serde_json::Value, // Will contain specific details based on transaction type
}