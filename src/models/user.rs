use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::{DeriveActiveEnum, EnumIter, FromQueryResult};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, DeriveActiveEnum, EnumIter,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "member_type")]
pub enum MemberType {
    #[sea_orm(string_value = "fan")]
    Fan,
    #[sea_orm(string_value = "sweet_shareholder")]
    SweetShareholder,
    #[sea_orm(string_value = "super_shareholder")]
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

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult, ToSchema)]
pub struct User {
    pub id: i64,
    pub member_code: String,
    pub phone: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub birthday: NaiveDate,
    pub member_type: MemberType,
    pub membership_expires_at: Option<DateTime<Utc>>, // 会员到期时间
    pub balance: Option<i64>,                         // 余额(美分)
    pub stamps: Option<i64>,                          // Stamps 数量
    pub referrer_id: Option<i64>,
    pub referral_code: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
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
    pub balance: i64,
    pub stamps: i64,
    pub referral_code: Option<String>,
    pub total_referrals: i64,
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
            membership_expires_at: user.membership_expires_at,
            balance: user.balance.unwrap_or(0),
            stamps: user.stamps.unwrap_or(0),
            referral_code: user.referral_code,
            total_referrals: 0, // 需要单独查询
            created_at: user.created_at.unwrap_or_else(Utc::now),
        }
    }
}
