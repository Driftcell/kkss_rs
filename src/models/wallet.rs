use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WalletTransactionKind {
    /// Stripe 充值（仅包含已成功的充值）
    Recharge,
    /// 系统发放的生日礼物
    BirthdayReward,
    /// 将余额兑换成优惠码
    Redeem,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WalletTransactionResponse {
    /// 源记录ID（充值记录ID或 sweet_cash_transactions ID）
    pub id: i64,
    pub kind: WalletTransactionKind,
    /// 金额（美分，正数）
    pub amount: i64,
    /// 余额变动后的余额，仅兑换/生日奖励记录会携带，充值记录可能为 None
    pub balance_after: Option<i64>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}
