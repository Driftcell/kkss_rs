use crate::error::{AppError, AppResult};
use crate::external::*;
use crate::models::*;
use crate::utils::generate_six_digit_code;
use chrono::{Duration, Utc};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct DiscountCodeService {
    pool: SqlitePool,
    sevencloud_api: std::sync::Arc<tokio::sync::Mutex<SevenCloudAPI>>,
}

impl DiscountCodeService {
    pub fn new(
        pool: SqlitePool,
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
        let total = sqlx::query!(
            "SELECT COUNT(*) as count FROM discount_codes WHERE user_id = ?",
            user_id
        )
        .fetch_one(&self.pool)
        .await?
        .count;

        // 获取优惠码列表
        let discount_codes = sqlx::query_as!(
            DiscountCode,
            r#"
            SELECT
                id, user_id, code, discount_amount,
                code_type as "code_type: CodeType",
                is_used, used_at, expires_at, external_id,
                created_at, updated_at
            FROM discount_codes
            WHERE user_id = ?
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
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

    pub async fn redeem_discount_code(
        &self,
        user_id: i64,
        request: RedeemDiscountCodeRequest,
    ) -> AppResult<RedeemDiscountCodeResponse> {
        // 验证兑换金额
        let sweet_cash_needed = request.discount_amount;

        // 验证有效期
        if request.expire_months < 1 || request.expire_months > 3 {
            return Err(AppError::ValidationError(
                "有效期必须在1-3个月之间".to_string(),
            ));
        }

        // 开始事务
        let mut tx = self.pool.begin().await?;

        // 检查用户甜品现金余额
        let user = sqlx::query!("SELECT sweet_cash FROM users WHERE id = ?", user_id)
            .fetch_optional(&mut *tx)
            .await?;

        let user = user.ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

        if user.sweet_cash.unwrap_or(0) < sweet_cash_needed {
            return Err(AppError::ValidationError("甜品现金余额不足".to_string()));
        }

        // 扣除甜品现金
        sqlx::query!(
            "UPDATE users SET sweet_cash = sweet_cash - ? WHERE id = ?",
            sweet_cash_needed,
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
            let api = self.sevencloud_api.lock().await;
            api.generate_discount_code(&code, discount_dollars, request.expire_months)
                .await?;
        }

        // 保存优惠码到本地数据库
        let code_type_str = CodeType::Redeemed.to_string();
        let discount_code_id = sqlx::query!(
            r#"
            INSERT INTO discount_codes (
                user_id, code, discount_amount, code_type, expires_at
            ) VALUES (?, ?, ?, ?, ?)
            "#,
            user_id,
            code,
            request.discount_amount,
            code_type_str,
            expires_at
        )
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();

        // 记录甜品现金交易
        let remaining_sweet_cash = user.sweet_cash.unwrap_or(0) - sweet_cash_needed;
        let transaction_type_str = TransactionType::Redeem.to_string();
        let negative_amount = -sweet_cash_needed;
        let description = format!("兑换{}美分优惠码", request.discount_amount);

        sqlx::query!(
            r#"
            INSERT INTO sweet_cash_transactions (
                user_id, transaction_type, amount, balance_after,
                related_discount_code_id, description
            ) VALUES (?, ?, ?, ?, ?, ?)
            "#,
            user_id,
            transaction_type_str,
            negative_amount,
            remaining_sweet_cash,
            discount_code_id,
            description
        )
        .execute(&mut *tx)
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
            sweet_cash_used: sweet_cash_needed,
            remaining_sweet_cash,
        })
    }
}
