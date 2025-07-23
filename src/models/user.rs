use chrono::{DateTime, Utc, NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, ToSchema)]
#[sqlx(type_name = "member_type", rename_all = "snake_case")]
pub enum MemberType {
    #[serde(rename = "fan")]
    Fan,
    #[serde(rename = "sweet_shareholder")]
    SweetShareholder,
    #[serde(rename = "super_shareholder")]
    SuperShareholder,
}

impl std::fmt::Display for MemberType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemberType::Fan => write!(f, "fan"),
            MemberType::SweetShareholder => write!(f, "sweet_shareholder"),
            MemberType::SuperShareholder => write!(f, "super_shareholder"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct User {
    pub id: i64,
    pub member_code: String,
    pub phone: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub birthday: NaiveDate,
    pub member_type: MemberType,
    pub balance: Option<i64>,  // 余额(美分)
    pub sweet_cash: Option<i64>,  // 甜品现金(美分)
    pub referrer_id: Option<i64>,
    pub referral_code: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

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
    pub birthday: String,  // YYYY-MM-DD
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
    pub balance: i64,
    pub sweet_cash: i64,
    pub referral_code: Option<String>,
    pub total_referrals: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserStatistics {
    pub total_orders: i64,
    pub total_spent: i64,
    pub total_earned_sweet_cash: i64,
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
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SendCodeResponse {
    pub expires_in: i64,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            member_code: user.member_code,
            username: user.username,
            phone: user.phone,
            birthday: user.birthday.format("%Y-%m-%d").to_string(),
            member_type: user.member_type,
            balance: user.balance.unwrap_or(0),
            sweet_cash: user.sweet_cash.unwrap_or(0),
            referral_code: user.referral_code,
            total_referrals: 0, // 需要单独查询
            created_at: user.created_at.map(|dt| dt.and_utc()).unwrap_or_else(|| Utc::now()),
        }
    }
}
