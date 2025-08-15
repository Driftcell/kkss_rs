use crate::error::{AppError, AppResult};
use crate::models::*;
use sqlx::PgPool;

#[derive(Clone)]
pub struct UserService {
    pool: PgPool,
}

impl UserService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 获取用户个人资料和统计信息
    pub async fn get_user_profile(
        &self,
        user_id: i64,
    ) -> AppResult<(UserResponse, UserStatistics)> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT
                id, member_code, phone, username, password_hash, birthday,
                member_type,
                membership_expires_at,
                balance, stamps, referrer_id, referral_code,
                created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        let user = user.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // 获取推荐人数
        let total_referrals: i64 =
            sqlx::query_scalar("SELECT COUNT(*)::BIGINT FROM users WHERE referrer_id = $1")
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?
                .unwrap_or(0);

        // 获取用户统计信息
        let statistics = self.get_user_statistics(user_id).await?;

        let mut user_response = UserResponse::from(user);
        user_response.total_referrals = total_referrals;

        Ok((user_response, statistics))
    }

    /// 更新用户Profile
    pub async fn update_user_profile(
        &self,
        user_id: i64,
        request: UpdateUserRequest,
    ) -> AppResult<UserResponse> {
        // 验证输入
        if let Some(username) = &request.username {
            if username.len() < 2 || username.len() > 20 {
                return Err(AppError::ValidationError(
                    "Username length must be between 2 and 20 characters".to_string(),
                ));
            }
        }

        let birthday = if let Some(birthday_str) = &request.birthday {
            Some(
                chrono::NaiveDate::parse_from_str(birthday_str, "%Y-%m-%d").map_err(|_| {
                    AppError::ValidationError("Invalid birthday format".to_string())
                })?,
            )
        } else {
            None
        };

        // 检查是否有需要更新的字段
        if request.username.is_none() && request.birthday.is_none() {
            return Err(AppError::ValidationError("No fields to update".to_string()));
        }

        // 根据提供的字段执行相应的更新
        match (&request.username, &birthday) {
            (Some(username), Some(birthday)) => {
                // 同时更新用户名和生日
                sqlx::query!(
                    "UPDATE users SET username = $1, birthday = $2, updated_at = NOW() WHERE id = $3",
                    username,
                    birthday,
                    user_id
                )
                .execute(&self.pool)
                .await?;
            }
            (Some(username), None) => {
                // 只更新用户名
                sqlx::query!(
                    "UPDATE users SET username = $1, updated_at = NOW() WHERE id = $2",
                    username,
                    user_id
                )
                .execute(&self.pool)
                .await?;
            }
            (None, Some(birthday)) => {
                // 只更新生日
                sqlx::query!(
                    "UPDATE users SET birthday = $1, updated_at = NOW() WHERE id = $2",
                    birthday,
                    user_id
                )
                .execute(&self.pool)
                .await?;
            }
            (None, None) => {
                // 这种情况已经在上面检查过了，但为了完整性保留
                return Err(AppError::ValidationError("No fields to update".to_string()));
            }
        }

        // 返回更新后的用户信息
        let (user_response, _) = self.get_user_profile(user_id).await?;
        Ok(user_response)
    }

    /// 获取用户推荐列表
    pub async fn get_user_referrals(
        &self,
        user_id: i64,
        params: &PaginationParams,
    ) -> AppResult<PaginatedResponse<UserResponse>> {
        let offset = params.get_offset();
        let limit = params.get_limit();

        // 获取总数
        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*)::BIGINT FROM users WHERE referrer_id = $1")
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?
                .unwrap_or(0);

        // 获取推荐用户列表
        let referrals = sqlx::query_as::<_, User>(
            r#"
            SELECT
                id, member_code, phone, username, password_hash, birthday,
                member_type,
                membership_expires_at,
                balance, stamps, referrer_id, referral_code,
                created_at, updated_at
            FROM users
            WHERE referrer_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let items: Vec<UserResponse> = referrals.into_iter().map(UserResponse::from).collect();

        Ok(PaginatedResponse::new(
            items,
            params.page.unwrap_or(1),
            params.page_size.unwrap_or(20),
            total,
        ))
    }

    /// 获取用户统计信息
    async fn get_user_statistics(&self, user_id: i64) -> AppResult<UserStatistics> {
        // 获取订单统计
        let order_stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*)::BIGINT as total_orders,
                COALESCE(SUM(price), 0)::BIGINT as total_spent,
                COALESCE(SUM(stamps_earned), 0)::BIGINT as total_earned_stamps
            FROM orders 
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        // 获取可用优惠码数量
        let available_codes = sqlx::query!(
            r#"
            SELECT COUNT(*)::BIGINT as count 
            FROM discount_codes 
            WHERE user_id = $1 AND is_used = FALSE AND expires_at > NOW()
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(UserStatistics {
            total_orders: order_stats.total_orders.unwrap_or(0),
            total_spent: order_stats.total_spent.unwrap_or(0),
            total_earned_stamps: order_stats.total_earned_stamps.unwrap_or(0),
            available_discount_codes: available_codes.count.unwrap_or(0),
        })
    }
}
