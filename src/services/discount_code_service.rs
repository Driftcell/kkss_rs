use crate::error::{AppError, AppResult};
use crate::external::*;
use crate::models::*;
use crate::utils::generate_six_digit_code;
use chrono::{Duration, Utc};
use sqlx::PgPool;

#[derive(Clone)]
pub struct DiscountCodeService {
    pool: PgPool,
    sevencloud_api: std::sync::Arc<tokio::sync::Mutex<SevenCloudAPI>>,
}

impl DiscountCodeService {
    pub fn new(
        pool: PgPool,
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
        let offset = params.get_offset() as i64;
        let limit = params.get_limit() as i64;

        // 获取总数
        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!: i64" FROM discount_codes WHERE user_id = $1"#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        // 获取优惠码列表
        let discount_codes = sqlx::query_as!(
            DiscountCode,
            r#"
            SELECT
                id, user_id, code, discount_amount,
                code_type as "code_type: _",
                is_used, used_at, expires_at, external_id,
                created_at, updated_at
            FROM discount_codes
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

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
        let mut tx = self.pool.begin().await?;

        // 检查用户 stamps 余额
        let user = sqlx::query!(r#"SELECT stamps FROM users WHERE id = $1"#, user_id)
            .fetch_optional(&mut *tx)
            .await?;

        let user = user.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        let current_stamps = user.stamps.unwrap_or(0);

        if current_stamps < stamps_needed {
            return Err(AppError::ValidationError("Insufficient stamps".to_string()));
        }

        // 扣除 stamps
        sqlx::query!(
            r#"UPDATE users SET stamps = stamps - $1 WHERE id = $2"#,
            stamps_needed,
            user_id
        )
        .execute(&mut *tx)
        .await?;

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
        let code_type_enum = CodeType::Redeemed;
        let discount_code_id: i64 = sqlx::query_scalar!(
            r#"
            INSERT INTO discount_codes (
                user_id, code, discount_amount, code_type, expires_at
            ) VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
            user_id,
            code,
            request.discount_amount,
            code_type_enum as _,
            expires_at
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        // 返回结果
        let discount_code = DiscountCodeResponse {
            id: discount_code_id,
            code,
            discount_amount: request.discount_amount,
            code_type: CodeType::Redeemed,
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

        let mut tx = self.pool.begin().await?;

        // 查询余额
        let user = sqlx::query!(r#"SELECT balance FROM users WHERE id = $1"#, user_id)
            .fetch_optional(&mut *tx)
            .await?;
        let user = user.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        let current_balance = user.balance.unwrap_or(0);
        if current_balance < request.discount_amount {
            return Err(AppError::ValidationError(
                "Insufficient balance".to_string(),
            ));
        }

        // 扣减余额
        sqlx::query!(
            r#"UPDATE users SET balance = balance - $1 WHERE id = $2"#,
            request.discount_amount,
            user_id
        )
        .execute(&mut *tx)
        .await?;

        // 生成优惠码
        let code = generate_six_digit_code();
        let expires_at = Utc::now() + Duration::days(30 * request.expire_months as i64);
        let discount_dollars = request.discount_amount as f64 / 100.0;
        {
            let mut api = self.sevencloud_api.lock().await;
            api.generate_discount_code(&code, discount_dollars, request.expire_months)
                .await?;
        }

        let code_type_enum = CodeType::Redeemed; // 与 stamps 兑换一致，标记为 redeemed
        let discount_code_id: i64 = sqlx::query_scalar!(
            r#"
            INSERT INTO discount_codes (
                user_id, code, discount_amount, code_type, expires_at
            ) VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
            user_id,
            code,
            request.discount_amount,
            code_type_enum as _,
            expires_at
        )
        .fetch_one(&mut *tx)
        .await?;

        // 记录 sweet_cash_transactions (Redeem)
        sqlx::query!(
            r#"INSERT INTO sweet_cash_transactions (
                user_id, transaction_type, amount, balance_after, related_discount_code_id, description
            ) VALUES ($1, 'redeem', $2, $3, $4, $5)"#,
            user_id,
            request.discount_amount,
            current_balance - request.discount_amount,
            discount_code_id,
            format!("Redeem balance for discount code {}", code)
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        let discount_code = DiscountCodeResponse {
            id: discount_code_id,
            code,
            discount_amount: request.discount_amount,
            code_type: CodeType::Redeemed,
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
                let exists = sqlx::query_scalar!(
                    "SELECT 1 as \"exists!: i64\" FROM discount_codes WHERE code = $1",
                    candidate
                )
                .fetch_optional(&self.pool)
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
        let id: i64 = sqlx::query_scalar!(
            r#"INSERT INTO discount_codes (user_id, code, discount_amount, code_type, expires_at)
                VALUES ($1, $2, $3, $4, $5) RETURNING id"#,
            user_id,
            code,
            amount,
            code_type as _,
            expires_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }
}
