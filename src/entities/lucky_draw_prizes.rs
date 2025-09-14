use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 抽奖奖品配置实体
/// 概念说明:
/// - probability_bp: 概率 (basis points) 1% = 100bp, 100% = 10000bp
/// - stock_limit: 奖品总库存 (NULL 表示无限)
/// - stock_remaining: 剩余库存 (NULL 表示无限, 不参与扣减)
/// - value_cents: 奖品对应价值(美分)，如优惠券金额；虚拟/谢谢参与类为0
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "lucky_draw_prizes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 英文奖品名称 (唯一)
    pub name_en: String,
    /// 奖品面值(美分) - 无金额类为0
    pub value_cents: i64,
    /// 概率 (basis points)
    pub probability_bp: i32,
    /// 库存上限 (NULL=无限)
    pub stock_limit: Option<i64>,
    /// 剩余库存 (NULL=无限)
    pub stock_remaining: Option<i64>,
    /// 是否启用
    pub is_active: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Model {
    /// 是否还有库存 (无限库存或剩余 > 0)
    pub fn is_available(&self) -> bool {
        match self.stock_remaining {
            None => true,
            Some(remain) => remain > 0,
        }
    }

    /// 是否是限量奖品
    pub fn is_limited(&self) -> bool {
        self.stock_limit.is_some()
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
