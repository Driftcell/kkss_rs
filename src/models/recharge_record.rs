use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, ToSchema)]
#[sqlx(type_name = "recharge_status", rename_all = "snake_case")]
pub enum RechargeStatus {
    Pending,
    Succeeded,
    Failed,
    Canceled,
}

impl std::fmt::Display for RechargeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RechargeStatus::Pending => write!(f, "pending"),
            RechargeStatus::Succeeded => write!(f, "succeeded"),
            RechargeStatus::Failed => write!(f, "failed"),
            RechargeStatus::Canceled => write!(f, "canceled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct RechargeRecord {
    pub id: Option<i64>,
    pub user_id: Option<i64>,
    pub stripe_payment_intent_id: String,
    pub amount: Option<i64>,       // 充值金额(美分)
    pub bonus_amount: Option<i64>, // 奖励金额(美分)
    pub total_amount: Option<i64>, // 实际到账金额(美分)
    pub status: RechargeStatus,
    pub stripe_status: Option<String>,
    // Postgres 中使用 TIMESTAMPTZ -> DateTime<Utc>
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreatePaymentIntentRequest {
    pub amount: i64, // 充值金额(美分)，支持: 10000, 20000, 30000, 50000
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

impl From<RechargeRecord> for RechargeRecordResponse {
    fn from(record: RechargeRecord) -> Self {
        Self {
            id: record.id.unwrap_or(0),
            amount: record.amount.unwrap_or(0),
            bonus_amount: record.bonus_amount.unwrap_or(0),
            total_amount: record.total_amount.unwrap_or(0),
            status: record.status,
            created_at: record
                .created_at
                .unwrap_or_else(|| Utc::now()),
        }
    }
}

/// 根据充值金额计算奖励金额
pub fn calculate_bonus_amount(amount: i64) -> i64 {
    match amount {
        10000 => 1500,  // $100 -> $115 (15%奖励)
        20000 => 3500,  // $200 -> $235 (17.5%奖励)
        30000 => 7500,  // $300 -> $375 (25%奖励)
        50000 => 15000, // $500 -> $650 (30%奖励)
        _ => 0,
    }
}
