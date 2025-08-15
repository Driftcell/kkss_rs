use chrono::{DateTime, Utc};
use sea_orm::{DeriveActiveEnum, EnumIter, FromQueryResult};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, DeriveActiveEnum, EnumIter,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "code_type")]
pub enum CodeType {
    #[sea_orm(string_value = "shareholder_reward")]
    ShareholderReward,
    #[sea_orm(string_value = "super_shareholder_reward")]
    SuperShareholderReward,
    #[sea_orm(string_value = "sweets_credits_reward")]
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

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult, ToSchema)]
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
