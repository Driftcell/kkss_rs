use crate::entities::{
    discount_code_entity as discount_codes, order_entity as orders, user_entity as users,
};
use crate::error::{AppError, AppResult};
use crate::models::*;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, QuerySelect, Set,
};

#[derive(Clone)]
pub struct UserService {
    pool: DatabaseConnection,
}

impl UserService {
    pub fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }

    /// 获取用户个人资料和统计信息
    pub async fn get_user_profile(
        &self,
        user_id: i64,
    ) -> AppResult<(UserResponse, UserStatistics)> {
        let u = users::Entity::find_by_id(user_id).one(&self.pool).await?;
        let user = u.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // 获取推荐人数
        #[derive(Debug, sea_orm::FromQueryResult)]
        struct CountRow {
            count: i64,
        }
        let total_referrals = users::Entity::find()
            .filter(users::Column::ReferrerId.eq(user_id))
            .select_only()
            .column_as(Expr::val(1).count(), "count")
            .into_model::<CountRow>()
            .one(&self.pool)
            .await?
            .map(|r| r.count)
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
        let mut model = users::Entity::find_by_id(user_id)
            .one(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?
            .into_active_model();
        if let Some(username) = &request.username {
            model.username = Set(username.clone());
        }
        if let Some(b) = &birthday {
            model.birthday = Set(*b);
        }
        let _updated = model.update(&self.pool).await?;

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
        #[derive(Debug, sea_orm::FromQueryResult)]
        struct CountRow2 {
            count: i64,
        }
        let total = users::Entity::find()
            .filter(users::Column::ReferrerId.eq(user_id))
            .select_only()
            .column_as(Expr::val(1).count(), "count")
            .into_model::<CountRow2>()
            .one(&self.pool)
            .await?
            .map(|r| r.count)
            .unwrap_or(0);

        // 获取推荐用户列表
        let models = users::Entity::find()
            .filter(users::Column::ReferrerId.eq(user_id))
            .order_by_desc(users::Column::CreatedAt)
            .limit(limit as u64)
            .offset(offset as u64)
            .all(&self.pool)
            .await?;
        let items: Vec<UserResponse> = models.into_iter().map(UserResponse::from).collect();

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
        #[derive(Debug, sea_orm::FromQueryResult)]
        struct OrderStatsRow {
            total_orders: i64,
            total_spent: i64,
            total_earned_stamps: i64,
        }
        let order_stats_row: Option<OrderStatsRow> = orders::Entity::find()
            .filter(orders::Column::UserId.eq(user_id))
            .select_only()
            .column_as(Expr::val(1).count(), "total_orders")
            .column_as(Expr::col(orders::Column::Price).sum(), "total_spent")
            .column_as(
                Expr::col(orders::Column::StampsEarned).sum(),
                "total_earned_stamps",
            )
            .into_model::<OrderStatsRow>()
            .one(&self.pool)
            .await?;

        // 获取可用优惠码数量
        #[derive(Debug, sea_orm::FromQueryResult)]
        struct CountRow3 {
            count: i64,
        }
        let available_codes = discount_codes::Entity::find()
            .filter(discount_codes::Column::UserId.eq(user_id))
            .filter(discount_codes::Column::IsUsed.eq(false))
            .filter(discount_codes::Column::ExpiresAt.gt(chrono::Utc::now()))
            .select_only()
            .column_as(Expr::val(1).count(), "count")
            .into_model::<CountRow3>()
            .one(&self.pool)
            .await?
            .map(|r| r.count)
            .unwrap_or(0);

        Ok(UserStatistics {
            total_orders: order_stats_row
                .as_ref()
                .map(|r| r.total_orders)
                .unwrap_or(0),
            total_spent: order_stats_row.as_ref().map(|r| r.total_spent).unwrap_or(0),
            total_earned_stamps: order_stats_row
                .as_ref()
                .map(|r| r.total_earned_stamps)
                .unwrap_or(0),
            available_discount_codes: available_codes,
        })
    }
}
