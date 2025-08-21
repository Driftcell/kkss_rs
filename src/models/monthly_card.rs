use crate::entities::{MonthlyCardPlanType, MonthlyCardStatus, monthly_card_entity as mc};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateMonthlyCardIntentRequest {
    pub plan_type: MonthlyCardPlanType,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateMonthlyCardIntentResponse {
    pub payment_intent_id: String,
    pub client_secret: String,
    /// Stripe Checkout 会话 URL（跳转到 Stripe 官方收银台）
    pub checkout_url: String,
    pub amount: i64,
    pub plan_type: MonthlyCardPlanType,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ConfirmMonthlyCardRequest {
    pub payment_intent_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MonthlyCardRecordResponse {
    pub id: i64,
    pub plan_type: MonthlyCardPlanType,
    pub status: MonthlyCardStatus,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub last_coupon_granted_on: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}

impl From<mc::Model> for MonthlyCardRecordResponse {
    fn from(m: mc::Model) -> Self {
        Self {
            id: m.id,
            plan_type: m.plan_type,
            status: m.status,
            starts_at: m.starts_at,
            ends_at: m.ends_at,
            last_coupon_granted_on: m.last_coupon_granted_on,
            created_at: m.created_at.unwrap_or_else(Utc::now),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ConfirmMonthlyCardResponse {
    pub monthly_card: MonthlyCardRecordResponse,
}
