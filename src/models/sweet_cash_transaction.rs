use chrono::{DateTime, Utc};
use sea_orm::{DeriveActiveEnum, EnumIter, FromQueryResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, DeriveActiveEnum, EnumIter)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "transaction_type")]
pub enum TransactionType {
    #[sea_orm(string_value = "earn")]
    Earn, // 赚取
    #[sea_orm(string_value = "redeem")]
    Redeem, // 兑换
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionType::Earn => write!(f, "earn"),
            TransactionType::Redeem => write!(f, "redeem"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult)]
pub struct SweetCashTransaction {
    pub id: i64,
    pub user_id: i64,
    pub transaction_type: TransactionType,
    pub amount: i64,        // 交易金额(美分)
    pub balance_after: i64, // 交易后余额
    pub related_order_id: Option<i64>,
    pub related_discount_code_id: Option<i64>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SweetCashTransactionResponse {
    pub id: i64,
    pub transaction_type: TransactionType,
    pub amount: i64,
    pub balance_after: i64,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<SweetCashTransaction> for SweetCashTransactionResponse {
    fn from(transaction: SweetCashTransaction) -> Self {
        Self {
            id: transaction.id,
            transaction_type: transaction.transaction_type,
            amount: transaction.amount,
            balance_after: transaction.balance_after,
            description: transaction.description,
            created_at: transaction.created_at,
        }
    }
}
