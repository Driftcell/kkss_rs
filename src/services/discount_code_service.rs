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

    pub async fn get_user_discount_codes(
        &self,
        user_id: i64,
        query: &DiscountCodeQuery,
    ) -> AppResult<PaginatedResponse<DiscountCodeResponse>> {
        let params = PaginationParams::new(query.page, query.per_page);
        let offset = params.get_offset() as i64;
        let limit = params.get_limit() as i64;

        // 获取总数
        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM discount_codes WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(&self.pool)
                .await?;

        // 获取优惠码列表
        let discount_codes = sqlx::query_as::<_, DiscountCode>(
            r#"
            SELECT
                id, user_id, code, discount_amount,
                code_type,
                is_used, used_at, expires_at, external_id,
                created_at, updated_at
            FROM discount_codes
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
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

    pub async fn redeem_discount_code(
        &self,
        user_id: i64,
        request: RedeemDiscountCodeRequest,
    ) -> AppResult<RedeemDiscountCodeResponse> {
        // 验证兑换金额
        let allowed = [(5, 5), (10, 8), (20, 12), (25, 15)];
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
        let user = sqlx::query!("SELECT stamps FROM users WHERE id = $1", user_id)
            .fetch_optional(&mut *tx)
            .await?;

        let user = user.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        let current_stamps = user.stamps.unwrap_or(0);

        if current_stamps < stamps_needed {
            return Err(AppError::ValidationError("Insufficient stamps".to_string()));
        }

        // 扣除 stamps
        sqlx::query("UPDATE users SET stamps = stamps - $1 WHERE id = $2")
            .bind(stamps_needed)
            .bind(user_id)
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
        let discount_code_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO discount_codes (
                user_id, code, discount_amount, code_type, expires_at
            ) VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(&code)
        .bind(request.discount_amount)
        .bind(code_type_enum)
        .bind(expires_at)
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
        let user = sqlx::query!("SELECT balance FROM users WHERE id = $1", user_id)
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
        sqlx::query("UPDATE users SET balance = balance - $1 WHERE id = $2")
            .bind(request.discount_amount)
            .bind(user_id)
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
        let discount_code_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO discount_codes (
                user_id, code, discount_amount, code_type, expires_at
            ) VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(&code)
        .bind(request.discount_amount)
        .bind(code_type_enum)
        .bind(expires_at)
        .fetch_one(&mut *tx)
        .await?;

        // 记录 sweet_cash_transactions (Redeem)
        sqlx::query(
            r#"INSERT INTO sweet_cash_transactions (
                user_id, transaction_type, amount, balance_after, related_discount_code_id, description
            ) VALUES ($1, 'redeem', $2, $3, $4, $5)"#,
        )
        .bind(user_id)
        .bind(request.discount_amount)
        .bind(current_balance - request.discount_amount)
        .bind(discount_code_id)
        .bind(format!("Redeem balance for discount code {}", code))
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
}
