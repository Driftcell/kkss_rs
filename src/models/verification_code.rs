use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct VerificationCode {
    pub id: i64,
    pub phone: String,
    pub code: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateVerificationCodeRequest {
    pub phone: String,
    pub code: String,
    pub expires_at: DateTime<Utc>,
}
