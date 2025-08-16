use crate::entities::order_entity as orders;
use crate::error::AppResult;
use crate::models::*;
use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

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

        // 构建查询条件
        let mut where_conditions = vec!["user_id = ?".to_string()];
        let mut query_params = vec![user_id.to_string()];

        if let Some(status) = query.status {
            where_conditions.push("order_status = ?".to_string());
            query_params.push(status.to_string());
        }

        if let Some(start_date) = &query.start_date {
            where_conditions.push("date(external_created_at) >= ?".to_string());
            query_params.push(start_date.clone());
        }

        if let Some(end_date) = &query.end_date {
            where_conditions.push("date(external_created_at) <= ?".to_string());
            query_params.push(end_date.clone());
        }

        let _where_clause = where_conditions.join(" AND ");

        // 获取总数
        #[derive(Debug, sea_orm::FromQueryResult)]
        struct CountRow {
            count: i64,
        }
        let total = orders::Entity::find()
            .filter(orders::Column::UserId.eq(user_id))
            .select_only()
            .column_as(Expr::val(1).count(), "count")
            .into_model::<CountRow>()
            .one(&self.pool)
            .await?
            .map(|r| r.count)
            .unwrap_or(0);

        // 获取订单列表
        let models = orders::Entity::find()
            .filter(orders::Column::UserId.eq(user_id))
            .order_by_desc(orders::Column::ExternalCreatedAt)
            .limit(limit as u64)
            .offset(offset as u64)
            .all(&self.pool)
            .await?;
        let items: Vec<OrderResponse> = models.into_iter().map(OrderResponse::from).collect();

        Ok(PaginatedResponse::new(
            items,
            params.get_offset() / params.get_limit() + 1,
            params.get_limit(),
            total,
        ))
    }
}
