use crate::entities::MemberType;
use crate::entities::user_entity;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    #[schema(example = "+1234567890")]
    pub phone: String,
    #[schema(example = "123456")]
    pub verification_code: String,
    #[schema(example = "张三")]
    pub username: String,
    #[schema(example = "password123")]
    pub password: String,
    #[schema(example = "1990-01-01")]
    pub birthday: String, // YYYY-MM-DD
    #[schema(example = "REF123")]
    pub referrer_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
    #[schema(example = "+1234567890")]
    pub phone: String,
    #[schema(example = "password123")]
    pub password: String,
    pub remember_me: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    #[schema(example = "张三")]
    pub username: Option<String>,
    #[schema(example = "1990-01-01")]
    pub birthday: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserResponse {
    pub id: i64,
    pub member_code: String,
    pub username: String,
    pub phone: String,
    pub birthday: String,
    pub member_type: MemberType,
    pub membership_expires_at: Option<DateTime<Utc>>,
    pub monthly_card_expires_at: Option<DateTime<Utc>>,
    pub balance: i64,
    pub stamps: i64,
    pub referral_code: Option<String>,
    pub total_referrals: i64,
    pub is_monthly_card: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserStatistics {
    pub total_orders: i64,
    pub total_spent: i64,
    pub total_earned_stamps: i64,
    pub available_discount_codes: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SendCodeRequest {
    #[schema(example = "+1234567890")]
    pub phone: String,
    /// Turnstile token from client-side widget
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "CF_TURNSTILE_TOKEN")]
    pub cf_turnstile_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SendCodeResponse {
    pub expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ResetPasswordRequest {
    #[schema(example = "+12345678901")]
    pub phone: String,
    #[schema(example = "123456")]
    pub verification_code: String,
    #[schema(example = "NewPassword123")]
    pub new_password: String,
}

// Convert from entity Model to API response
impl From<user_entity::Model> for UserResponse {
    fn from(m: user_entity::Model) -> Self {
        Self {
            id: m.id,
            member_code: m.member_code,
            username: m.username,
            phone: m.phone,
            birthday: m.birthday.format("%Y-%m-%d").to_string(),
            member_type: m.member_type,
            membership_expires_at: m.membership_expires_at,
            monthly_card_expires_at: None,
            balance: m.balance.unwrap_or(0),
            stamps: m.stamps.unwrap_or(0),
            referral_code: m.referral_code,
            total_referrals: 0,
            is_monthly_card: false,
            created_at: m.created_at.unwrap_or_else(Utc::now),
        }
    }
}
