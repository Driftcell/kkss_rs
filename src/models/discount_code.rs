use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "code_type", rename_all = "snake_case")]
pub enum CodeType {
    ShareholderReward,
    SuperShareholderReward,
    SweetsCreditsReward,
}

impl std::fmt::Display for CodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeType::ShareholderReward => write!(f, "shareholder_reward"),
            CodeType::SuperShareholderReward => write!(f, "super_shareholder_reward"),
            CodeType::SweetsCreditsReward => write!(f, "sweets_credits_reward"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct DiscountCode {
    pub id: Option<i64>,
    pub user_id: Option<i64>,
    pub code: String,
    pub discount_amount: Option<i64>, // 优惠金额(美分)
    pub code_type: CodeType,
    pub is_used: Option<bool>,
    pub used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub external_id: Option<i64>, // 七云优惠码ID
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DiscountCodeResponse {
    pub id: i64,
    pub code: String,
    pub discount_amount: i64,
    pub code_type: CodeType,
    pub is_used: bool,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DiscountCodeQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub status: Option<String>,    // available/used/expired
    pub code_type: Option<String>, // shareholder_reward/super_shareholder_reward/sweets_credits_reward
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RedeemDiscountCodeRequest {
    pub discount_amount: i64, // 要兑换的优惠码金额(美分)
    pub expire_months: u32,   // 有效期(月)，1-3
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RedeemDiscountCodeResponse {
    pub discount_code: DiscountCodeResponse,
    pub stamps_used: i64,
    pub remaining_stamps: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RedeemBalanceDiscountCodeRequest {
    pub discount_amount: i64, // 要兑换的优惠码金额(美分)，与 balance 1:1 扣减
    pub expire_months: u32,   // 有效期(月)，1-3
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RedeemBalanceDiscountCodeResponse {
    pub discount_code: DiscountCodeResponse,
    pub balance_used: i64,
    pub remaining_balance: i64,
}

impl From<DiscountCode> for DiscountCodeResponse {
    fn from(code: DiscountCode) -> Self {
        Self {
            id: code.id.unwrap_or(0),
            code: code.code,
            discount_amount: code.discount_amount.unwrap_or(0),
            code_type: code.code_type,
            is_used: code.is_used.unwrap_or(false),
            expires_at: code.expires_at.unwrap_or_else(Utc::now),
            created_at: code.created_at.unwrap_or_else(Utc::now),
        }
    }
}
