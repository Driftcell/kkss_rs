use chrono::{DateTime, Utc, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "code_type", rename_all = "snake_case")]
pub enum CodeType {
    Welcome,
    Referral,
    PurchaseReward,
    Redeemed,
}

impl std::fmt::Display for CodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeType::Welcome => write!(f, "welcome"),
            CodeType::Referral => write!(f, "referral"),
            CodeType::PurchaseReward => write!(f, "purchase_reward"),
            CodeType::Redeemed => write!(f, "redeemed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct DiscountCode {
    pub id: Option<i64>,
    pub user_id: Option<i64>,
    pub code: String,
    pub discount_amount: Option<i64>,  // 优惠金额(美分)
    pub code_type: CodeType,
    pub is_used: Option<bool>,
    pub used_at: Option<NaiveDateTime>,
    pub expires_at: Option<NaiveDateTime>,
    pub external_id: Option<i64>,  // 七云优惠码ID
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
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
    pub status: Option<String>,  // available/used/expired
    pub code_type: Option<String>,  // welcome/referral/purchase_reward/redeemed
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RedeemDiscountCodeRequest {
    pub discount_amount: i64,  // 要兑换的优惠码金额(美分)
    pub expire_months: u32,    // 有效期(月)，1-3
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RedeemDiscountCodeResponse {
    pub discount_code: DiscountCodeResponse,
    pub sweet_cash_used: i64,
    pub remaining_sweet_cash: i64,
}

impl From<DiscountCode> for DiscountCodeResponse {
    fn from(code: DiscountCode) -> Self {
        Self {
            id: code.id.unwrap_or(0),
            code: code.code,
            discount_amount: code.discount_amount.unwrap_or(0),
            code_type: code.code_type,
            is_used: code.is_used.unwrap_or(false),
            expires_at: code.expires_at.map(|dt| dt.and_utc()).unwrap_or_else(|| Utc::now()),
            created_at: code.created_at.map(|dt| dt.and_utc()).unwrap_or_else(|| Utc::now()),
        }
    }
}
