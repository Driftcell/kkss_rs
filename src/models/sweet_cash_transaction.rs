use crate::entities::TransactionType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SweetCashTransactionResponse {
    pub id: i64,
    pub transaction_type: TransactionType,
    pub amount: i64,
    pub balance_after: i64,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}
