use sqlx::SqlitePool;
use crate::models::*;
use crate::utils::*;
use crate::error::{AppError, AppResult};

#[derive(Clone)]
pub struct UserService {
    pool: SqlitePool,
}

impl UserService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_user_profile(&self, user_id: i64) -> AppResult<(UserResponse, UserStatistics)> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT
                id, member_code, phone, username, password_hash, birthday,
                member_type as "member_type: MemberType",
                balance, sweet_cash, referrer_id, referral_code,
                created_at, updated_at
            FROM users
            WHERE id = ?
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        let user = user.ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

        // 获取推荐人数
        let total_referrals = sqlx::query!(
            "SELECT COUNT(*) as count FROM users WHERE referrer_id = ?",
            user_id
        )
        .fetch_one(&self.pool)
        .await?
        .count;

        // 获取用户统计信息
        let statistics = self.get_user_statistics(user_id).await?;

        let mut user_response = UserResponse::from(user);
        user_response.total_referrals = total_referrals;

        Ok((user_response, statistics))
    }

    pub async fn update_user_profile(&self, user_id: i64, request: UpdateUserRequest) -> AppResult<UserResponse> {
        let mut update_fields = Vec::new();
        let mut values: Vec<Box<dyn ToSql + Send + Sync>> = Vec::new();

        if let Some(username) = &request.username {
            if username.len() < 2 || username.len() > 20 {
                return Err(AppError::ValidationError("用户名长度必须在2-20字符之间".to_string()));
            }
            update_fields.push("username = ?");
            values.push(Box::new(username.clone()));
        }

        if let Some(birthday_str) = &request.birthday {
            let birthday = chrono::NaiveDate::parse_from_str(birthday_str, "%Y-%m-%d")
                .map_err(|_| AppError::ValidationError("生日格式无效".to_string()))?;
            update_fields.push("birthday = ?");
            values.push(Box::new(birthday));
        }

        if update_fields.is_empty() {
            return Err(AppError::ValidationError("没有需要更新的字段".to_string()));
        }

        update_fields.push("updated_at = CURRENT_TIMESTAMP");
        values.push(Box::new(user_id));

        let _query = format!(
            "UPDATE users SET {} WHERE id = ?",
            update_fields.join(", ")
        );

        // 由于sqlx的限制，这里简化处理
        if let Some(username) = &request.username {
            sqlx::query!(
                "UPDATE users SET username = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                username,
                user_id
            )
            .execute(&self.pool)
            .await?;
        }

        if let Some(birthday_str) = &request.birthday {
            let birthday = chrono::NaiveDate::parse_from_str(birthday_str, "%Y-%m-%d")
                .map_err(|_| AppError::ValidationError("生日格式无效".to_string()))?;
            sqlx::query!(
                "UPDATE users SET birthday = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                birthday,
                user_id
            )
            .execute(&self.pool)
            .await?;
        }

        // 返回更新后的用户信息
        let (user_response, _) = self.get_user_profile(user_id).await?;
        Ok(user_response)
    }

    pub async fn get_user_referrals(&self, user_id: i64, params: &PaginationParams) -> AppResult<PaginatedResponse<UserResponse>> {
        let offset = params.get_offset() as i64;
        let limit = params.get_limit() as i64;

        // 获取总数
        let total = sqlx::query!(
            "SELECT COUNT(*) as count FROM users WHERE referrer_id = ?",
            user_id
        )
        .fetch_one(&self.pool)
        .await?
        .count;

        // 获取推荐用户列表
        let referrals = sqlx::query_as!(
            User,
            r#"
            SELECT
                id, member_code, phone, username, password_hash, birthday,
                member_type as "member_type: MemberType",
                balance, sweet_cash, referrer_id, referral_code,
                created_at, updated_at
            FROM users
            WHERE referrer_id = ?
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let items: Vec<UserResponse> = referrals.into_iter().map(UserResponse::from).collect();

        Ok(PaginatedResponse::new(items, params.page.unwrap_or(1), params.page_size.unwrap_or(20), total))
    }

    async fn get_user_statistics(&self, user_id: i64) -> AppResult<UserStatistics> {
        // 获取订单统计
        let order_stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_orders,
                COALESCE(SUM(price), 0) as total_spent,
                COALESCE(SUM(sweet_cash_earned), 0) as total_earned_sweet_cash
            FROM orders 
            WHERE user_id = ?
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        // 获取可用优惠码数量
        let available_codes = sqlx::query!(
            r#"
            SELECT COUNT(*) as count 
            FROM discount_codes 
            WHERE user_id = ? AND is_used = FALSE AND expires_at > datetime('now')
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(UserStatistics {
            total_orders: order_stats.total_orders,
            total_spent: order_stats.total_spent,
            total_earned_sweet_cash: order_stats.total_earned_sweet_cash,
            available_discount_codes: available_codes.count,
        })
    }
}

// 简化的ToSql trait，实际应该使用sqlx的ToSql
trait ToSql {
    // 简化实现
}

impl ToSql for String {}
impl ToSql for chrono::NaiveDate {}
impl ToSql for i64 {}
