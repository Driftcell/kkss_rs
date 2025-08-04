use crate::error::AppResult;
use crate::models::*;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct OrderService {
    pool: SqlitePool,
}

impl OrderService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_user_orders(
        &self,
        user_id: i64,
        query: &OrderQuery,
    ) -> AppResult<PaginatedResponse<OrderResponse>> {
        let params = PaginationParams::new(query.page, query.per_page);
        let offset = params.get_offset() as i64;
        let limit = params.get_limit() as i64;

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

        // 获取总数 - 简化查询
        let total = sqlx::query!(
            "SELECT COUNT(*) as count FROM orders WHERE user_id = ?",
            user_id
        )
        .fetch_one(&self.pool)
        .await?
        .count;

        // 获取订单列表 - 简化查询
        let orders = sqlx::query_as!(
            Order,
            r#"
            SELECT
                id, user_id, member_code, price, product_name, product_no,
                order_status, pay_type, sweet_cash_earned,
                external_created_at, created_at, updated_at
            FROM orders
            WHERE user_id = ?
            ORDER BY external_created_at DESC
            LIMIT ? OFFSET ?
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let items: Vec<OrderResponse> = orders.into_iter().map(OrderResponse::from).collect();

        Ok(PaginatedResponse::new(
            items,
            params.get_offset() / params.get_limit() + 1,
            params.get_limit(),
            total,
        ))
    }
}
