use crate::entities::{
    discount_code_entity as discount_codes, sweet_cash_transaction_entity as sct,
    user_entity as users,
};
use crate::error::{AppError, AppResult};
use crate::external::*;
use crate::models::*;
use crate::utils::generate_six_digit_code;
use chrono::{Duration, Utc};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, QuerySelect, Set, TransactionTrait,
};

// removed raw-sql IdRow

#[derive(Clone)]
pub struct DiscountCodeService {
    pool: DatabaseConnection,
    sevencloud_api: std::sync::Arc<tokio::sync::Mutex<SevenCloudAPI>>,
}

impl DiscountCodeService {
    pub fn new(
        pool: DatabaseConnection,
        sevencloud_api: std::sync::Arc<tokio::sync::Mutex<SevenCloudAPI>>,
    ) -> Self {
        Self {
            pool,
            sevencloud_api,
        }
    }

    /// 获取用户的优惠码
    pub async fn get_user_discount_codes(
        &self,
        user_id: i64,
        query: &DiscountCodeQuery,
    ) -> AppResult<PaginatedResponse<DiscountCodeResponse>> {
        let params = PaginationParams::new(query.page, query.per_page);
        let offset = params.get_offset();
        let limit = params.get_limit();

        // 获取总数
        #[derive(Debug, sea_orm::FromQueryResult)]
        struct CountRow {
            count: i64,
        }
        let total = discount_codes::Entity::find()
            .filter(discount_codes::Column::UserId.eq(user_id))
            .select_only()
            .column_as(Expr::val(1).count(), "count")
            .into_model::<CountRow>()
            .one(&self.pool)
            .await?
            .map(|r| r.count)
            .unwrap_or(0);

        // 获取优惠码列表
        let models = discount_codes::Entity::find()
            .filter(discount_codes::Column::UserId.eq(user_id))
            .order_by_desc(discount_codes::Column::CreatedAt)
            .limit(limit as u64)
            .offset(offset as u64)
            .all(&self.pool)
            .await?;
        let discount_codes: Vec<DiscountCode> = models.into_iter().map(map_discount_code).collect();

        let items: Vec<DiscountCodeResponse> = discount_codes
            .into_iter()
            .map(DiscountCodeResponse::from)
            .collect();

        Ok(PaginatedResponse::new(
            items,
            params.page.unwrap_or(1),
            params.page_size.unwrap_or(20),
            total,
        ))
    }

    /// 兑换优惠码
    pub async fn redeem_discount_code(
        &self,
        user_id: i64,
        request: RedeemDiscountCodeRequest,
    ) -> AppResult<RedeemDiscountCodeResponse> {
        // 验证兑换金额
        let allowed = [(5, 10)];
        let mut stamps_required: Option<i64> = None;
        for (value_dollars, stamps) in allowed {
            if request.discount_amount == (value_dollars * 100) as i64 {
                stamps_required = Some(stamps as i64);
                break;
            }
        }

        let stamps_needed = stamps_required
            .ok_or_else(|| AppError::ValidationError("Unsupported discount amount".to_string()))?;

        // 验证有效期
        if request.expire_months < 1 || request.expire_months > 3 {
            return Err(AppError::ValidationError(
                "The expiration period must be between 1 and 3 months".to_string(),
            ));
        }

        // 开始事务
        let txn = self.pool.begin().await?;

        // 检查用户 stamps 余额
        let current_stamps = users::Entity::find_by_id(user_id)
            .one(&txn)
            .await?
            .and_then(|u| u.stamps)
            .unwrap_or(0);

        // current_stamps computed above

        if current_stamps < stamps_needed {
            return Err(AppError::ValidationError("Insufficient stamps".to_string()));
        }

        // 扣除 stamps
        if let Some(u) = users::Entity::find_by_id(user_id).one(&txn).await? {
            let new_stamps = u.stamps.unwrap_or(0) - stamps_needed;
            let mut am = u.into_active_model();
            am.stamps = Set(Some(new_stamps));
            am.update(&txn).await?;
        }

        // 生成优惠码
        let code = generate_six_digit_code(); // 生成6位数字码
        let expires_at = Utc::now() + Duration::days(30 * request.expire_months as i64);
        let discount_dollars = request.discount_amount as f64 / 100.0;

        // 调用七云API生成优惠码
        {
            let mut api = self.sevencloud_api.lock().await;
            api.generate_discount_code(&code, discount_dollars, request.expire_months)
                .await?;
        }

        // 保存优惠码到本地数据库
        let code_type_enum = CodeType::SweetsCreditsReward;
        let created = discount_codes::ActiveModel {
            user_id: Set(user_id),
            code: Set(code.clone()),
            discount_amount: Set(request.discount_amount),
            code_type: Set(code_type_enum),
            is_used: Set(Some(false)),
            expires_at: Set(expires_at),
            ..Default::default()
        }
        .insert(&txn)
        .await?;
        let discount_code_id = created.id;

        txn.commit().await?;

        // 返回结果
        let discount_code = DiscountCodeResponse {
            id: discount_code_id,
            code,
            discount_amount: request.discount_amount,
            code_type: CodeType::SweetsCreditsReward,
            is_used: false,
            expires_at,
            created_at: Utc::now(),
        };

        Ok(RedeemDiscountCodeResponse {
            discount_code,
            stamps_used: stamps_needed,
            remaining_stamps: current_stamps - stamps_needed,
        })
    }

    /// 兑换余额优惠码
    pub async fn redeem_balance_discount_code(
        &self,
        user_id: i64,
        request: RedeemBalanceDiscountCodeRequest,
    ) -> AppResult<RedeemBalanceDiscountCodeResponse> {
        // 校验金额: 为正且是100的倍数 (>= $1)
        if request.discount_amount <= 0 || request.discount_amount % 100 != 0 {
            return Err(AppError::ValidationError(
                "discount_amount must be positive and in cents (multiple of 100)".to_string(),
            ));
        }
        // 有效期 1-3 月
        if request.expire_months < 1 || request.expire_months > 3 {
            return Err(AppError::ValidationError(
                "The expiration period must be between 1 and 3 months".to_string(),
            ));
        }

        let txn = self.pool.begin().await?;

        // 查询余额
        let current_balance = users::Entity::find_by_id(user_id)
            .one(&txn)
            .await?
            .and_then(|u| u.balance)
            .unwrap_or(0);
        // current_balance computed above
        if current_balance < request.discount_amount {
            return Err(AppError::ValidationError(
                "Insufficient balance".to_string(),
            ));
        }

        // 扣减余额
        if let Some(u) = users::Entity::find_by_id(user_id).one(&txn).await? {
            let new_balance = u.balance.unwrap_or(0) - request.discount_amount;
            let mut am = u.into_active_model();
            am.balance = Set(Some(new_balance));
            am.update(&txn).await?;
        }

        // 生成优惠码
        let code = generate_six_digit_code();
        let expires_at = Utc::now() + Duration::days(30 * request.expire_months as i64);
        let discount_dollars = request.discount_amount as f64 / 100.0;
        {
            let mut api = self.sevencloud_api.lock().await;
            api.generate_discount_code(&code, discount_dollars, request.expire_months)
                .await?;
        }

        let code_type_enum = CodeType::SweetsCreditsReward; // 兑换获得，标记为 sweets_credits_reward
        let created = discount_codes::ActiveModel {
            user_id: Set(user_id),
            code: Set(code.clone()),
            discount_amount: Set(request.discount_amount),
            code_type: Set(code_type_enum),
            is_used: Set(Some(false)),
            expires_at: Set(expires_at),
            ..Default::default()
        }
        .insert(&txn)
        .await?;
        let discount_code_id = created.id;

        // 记录 sweet_cash_transactions (Redeem)
        sct::ActiveModel {
            user_id: Set(user_id),
            transaction_type: Set("redeem".to_string()),
            amount: Set(request.discount_amount),
            balance_after: Set(current_balance - request.discount_amount),
            related_order_id: Set(None),
            related_discount_code_id: Set(Some(discount_code_id)),
            description: Set(Some(format!("Redeem balance for discount code {}", code))),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        txn.commit().await?;

        let discount_code = DiscountCodeResponse {
            id: discount_code_id,
            code,
            discount_amount: request.discount_amount,
            code_type: CodeType::SweetsCreditsReward,
            is_used: false,
            expires_at,
            created_at: Utc::now(),
        };

        Ok(RedeemBalanceDiscountCodeResponse {
            discount_code,
            balance_used: request.discount_amount,
            remaining_balance: current_balance - request.discount_amount,
        })
    }

    /// 通用创建用户优惠码（注册奖励、推荐奖励等）
    ///
    /// # 参数
    ///
    /// * `user_id`: 用户id
    /// * `amount`: 美分
    /// * `code_type`: 优惠码类型
    /// * `expire_months`: 优惠码有效时间（1-3月）
    pub async fn create_user_discount_code(
        &self,
        user_id: i64,
        amount: i64,
        code_type: CodeType,
        expire_months: u32,
    ) -> AppResult<i64> {
        if amount <= 0 {
            return Err(AppError::ValidationError(
                "Discount amount must be positive".into(),
            ));
        }
        if expire_months == 0 || expire_months > 3 {
            return Err(AppError::ValidationError(
                "Expiration period must be between 1-3 months".into(),
            ));
        }

        let expires_at = Utc::now() + Duration::days(30 * expire_months as i64);

        // 生成唯一 6 位数字码
        let code = {
            let mut tries = 0;
            loop {
                tries += 1;
                let candidate = generate_six_digit_code();
                let exists = discount_codes::Entity::find()
                    .filter(discount_codes::Column::Code.eq(candidate.clone()))
                    .one(&self.pool)
                    .await?;
                if exists.is_none() {
                    break candidate;
                }
                if tries >= 10 {
                    return Err(AppError::InternalError(
                        "Failed to generate unique discount code".into(),
                    ));
                }
            }
        };

        let discount_dollars = amount as f64 / 100.0;
        {
            let mut api = self.sevencloud_api.lock().await;
            api.generate_discount_code(&code, discount_dollars, expire_months)
                .await?;
        }

        // 插入数据库
        let created = discount_codes::ActiveModel {
            user_id: Set(user_id),
            code: Set(code),
            discount_amount: Set(amount),
            code_type: Set(code_type),
            is_used: Set(Some(false)),
            expires_at: Set(expires_at),
            ..Default::default()
        }
        .insert(&self.pool)
        .await?;
        let id = created.id;

        Ok(id)
    }
}

fn map_discount_code(m: discount_codes::Model) -> DiscountCode {
    DiscountCode {
        id: Some(m.id),
        user_id: Some(m.user_id),
        code: m.code,
        discount_amount: Some(m.discount_amount),
        code_type: m.code_type,
        is_used: m.is_used,
        used_at: m.used_at,
        expires_at: Some(m.expires_at),
        external_id: m.external_id,
        created_at: m.created_at,
        updated_at: m.updated_at,
    }
}
