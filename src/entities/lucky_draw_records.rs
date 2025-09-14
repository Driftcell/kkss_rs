use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 抽奖记录实体
/// 说明:
/// - 每次用户抽奖产生一条记录
/// - prize_name_en 冗余存储方便历史查询 (即使奖品配置后续修改或下线仍可回溯)
/// - value_cents 奖品价值 (美分)，虚拟/谢谢参与类为 0
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "lucky_draw_records")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 用户ID
    pub user_id: i64,
    /// 奖品ID (指向 lucky_draw_prizes.id)
    pub prize_id: i64,
    /// 英文奖品名称 (历史快照)
    pub prize_name_en: String,
    /// 奖品价值(美分)，无金额类为 0
    pub value_cents: i64,
    /// 创建时间
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // 如果后续需要可建立与奖品表的关系:
    // #[sea_orm(
    //     belongs_to = "crate::entities::lucky_draw_prize_entity::Entity",
    //     from = "Column::PrizeId",
    //     to = "crate::entities::lucky_draw_prize_entity::Column::Id",
    //     on_update = "NoAction",
    //     on_delete = "NoAction"
    // )]
    // Prize,
}

impl ActiveModelBehavior for ActiveModel {}
