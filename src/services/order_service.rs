use crate::entities::TransactionType;
use crate::entities::order_entity as orders;
use crate::entities::sweet_cash_transaction_entity as sct;
use crate::error::AppResult;
use crate::models::*;
use chrono::{NaiveDate, TimeZone, Utc};
use sea_orm::Condition;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use std::collections::HashMap;

#[derive(Clone)]
pub struct OrderService {
    pool: DatabaseConnection,
}

impl OrderService {
    pub fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }

    /// 获取用户订单记录
    pub async fn get_user_orders(
        &self,
        user_id: i64,
        query: &OrderQuery,
    ) -> AppResult<PaginatedResponse<OrderResponse>> {
        let params = PaginationParams::new(query.page, query.per_page);
        let offset = params.get_offset();
        let limit = params.get_limit();
        // 构建 SeaORM 过滤条件
        let mut cond = Condition::all().add(orders::Column::UserId.eq(user_id));
        if let Some(status) = query.status {
            cond = cond.add(orders::Column::OrderStatus.eq(status));
        }
        if let Some(start_date) = &query.start_date
            && let Ok(nd) = NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
        {
            let start_dt = Utc.from_utc_datetime(&nd.and_hms_opt(0, 0, 0).unwrap());
            cond = cond.add(orders::Column::ExternalCreatedAt.gte(start_dt));
        }
        if let Some(end_date) = &query.end_date
            && let Ok(nd) = NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
        {
            let end_dt = Utc.from_utc_datetime(&nd.and_hms_opt(23, 59, 59).unwrap());
            cond = cond.add(orders::Column::ExternalCreatedAt.lte(end_dt));
        }

        let total = orders::Entity::find()
            .filter(cond.clone())
            .count(&self.pool)
            .await? as i64;

        // 获取订单列表
        let models = orders::Entity::find()
            .filter(cond)
            .order_by_desc(orders::Column::ExternalCreatedAt)
            .limit(limit as u64)
            .offset(offset as u64)
            .all(&self.pool)
            .await?;
        // 组装 sweet_cash_earned
        let order_ids: Vec<i64> = models.iter().map(|m| m.id).collect();
        let mut earned_map: HashMap<i64, i64> = HashMap::new();
        if !order_ids.is_empty() {
            let txs = sct::Entity::find()
                .filter(
                    Condition::all()
                        .add(sct::Column::UserId.eq(user_id))
                        .add(sct::Column::TransactionType.eq(TransactionType::Earn))
                        .add(sct::Column::RelatedOrderId.is_in(order_ids.clone())),
                )
                .all(&self.pool)
                .await?;
            for t in txs {
                if let Some(oid) = t.related_order_id {
                    *earned_map.entry(oid).or_insert(0) += t.amount;
                }
            }
        }
        let mut items: Vec<OrderResponse> = Vec::with_capacity(models.len());
        for m in models {
            let mut resp = OrderResponse::from(m);
            resp.sweet_cash_earned = *earned_map.get(&resp.id).unwrap_or(&0);
            items.push(resp);
        }

        Ok(PaginatedResponse::new(
            items,
            params.get_offset() / params.get_limit() + 1,
            params.get_limit(),
            total,
        ))
    }
}
