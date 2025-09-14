use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::entities::{
    lucky_draw_chance_entity as chances_entity, lucky_draw_prize_entity as prize_entity,
    lucky_draw_record_entity as record_entity,
};

use super::PaginatedResponse;

/// 抽奖记录查询参数
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct LuckyDrawRecordQuery {
    /// 页码 (默认 1)
    pub page: Option<u32>,
    /// 每页数量 (默认 20)
    pub per_page: Option<u32>,
}

/// 用户抽奖次数信息响应
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LuckyDrawChancesResponse {
    /// 累计发放次数
    pub total_awarded: i64,
    /// 已使用次数
    pub total_used: i64,
    /// 剩余次数
    pub remaining: i64,
}

impl From<chances_entity::Model> for LuckyDrawChancesResponse {
    fn from(m: chances_entity::Model) -> Self {
        LuckyDrawChancesResponse {
            total_awarded: m.total_awarded,
            total_used: m.total_used,
            remaining: m.total_awarded - m.total_used,
        }
    }
}

/// 奖品基础信息（用于展示列表）
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LuckyDrawPrizeResponse {
    /// 奖品ID
    pub id: i64,
    /// 英文名称 (前端显示 / 业务要求英文)
    pub name_en: String,
    /// 面值 (美分) - 无金额类为 0
    pub value_cents: i64,
    /// 概率 (basis points: 100% = 10000)
    pub probability_bp: i32,
    /// 总库存 (NULL / None = 无限)
    pub stock_limit: Option<i64>,
    /// 剩余库存 (NULL / None = 无限)
    pub stock_remaining: Option<i64>,
    /// 是否启用
    pub is_active: bool,
}

impl From<prize_entity::Model> for LuckyDrawPrizeResponse {
    fn from(m: prize_entity::Model) -> Self {
        LuckyDrawPrizeResponse {
            id: m.id,
            name_en: m.name_en,
            value_cents: m.value_cents,
            probability_bp: m.probability_bp,
            stock_limit: m.stock_limit,
            stock_remaining: m.stock_remaining,
            is_active: m.is_active,
        }
    }
}

/// 抽奖后返回给用户的奖品（隐藏不必要字段）
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LuckyDrawWonPrize {
    /// 奖品ID
    pub id: i64,
    /// 英文名称
    pub name_en: String,
    /// 奖品面值(美分) - 无金额为0
    pub value_cents: i64,
}

impl From<prize_entity::Model> for LuckyDrawWonPrize {
    fn from(m: prize_entity::Model) -> Self {
        LuckyDrawWonPrize {
            id: m.id,
            name_en: m.name_en,
            value_cents: m.value_cents,
        }
    }
}

/// 抽奖记录响应
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LuckyDrawRecordResponse {
    /// 记录ID
    pub id: i64,
    /// 奖品ID
    pub prize_id: i64,
    /// 奖品英文名称 (历史快照)
    pub prize_name_en: String,
    /// 奖品面值(美分)
    pub value_cents: i64,
    /// 抽奖时间
    pub created_at: DateTime<Utc>,
}

impl From<record_entity::Model> for LuckyDrawRecordResponse {
    fn from(m: record_entity::Model) -> Self {
        LuckyDrawRecordResponse {
            id: m.id,
            prize_id: m.prize_id,
            prize_name_en: m.prize_name_en,
            value_cents: m.value_cents,
            created_at: m.created_at.unwrap_or_else(Utc::now),
        }
    }
}

/// 抽奖（Spin）响应
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LuckyDrawSpinResponse {
    /// 获得的奖品
    pub prize: LuckyDrawWonPrize,
    /// 剩余抽奖次数
    pub remaining_chances: i64,
}

/// 奖品列表响应（分页）
pub type LuckyDrawPrizeListResponse = Vec<LuckyDrawPrizeResponse>;

/// 抽奖记录分页响应
pub type LuckyDrawRecordPageResponse = PaginatedResponse<LuckyDrawRecordResponse>;
