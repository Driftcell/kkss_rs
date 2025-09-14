use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 抽奖次数统计表实体
/// 说明:
/// - total_awarded: 累计发放的抽奖次数
/// - total_used: 已使用的抽奖次数
/// - 剩余次数 = total_awarded - total_used
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "lucky_draw_chances")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_id: i64,
    pub total_awarded: i64,
    pub total_used: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Model {
    /// 计算剩余抽奖次数
    pub fn remaining(&self) -> i64 {
        self.total_awarded - self.total_used
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
