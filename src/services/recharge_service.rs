use crate::error::{AppError, AppResult};
use crate::external::stripe::StripeService;
use crate::models::{
    ConfirmRechargeRequest, ConfirmRechargeResponse, CreatePaymentIntentResponse,
    PaginatedResponse, PaginationParams, RechargeQuery, RechargeRecord, RechargeRecordResponse,
    RechargeStatus,
};
use sqlx::PgPool;
use stripe::PaymentIntentStatus;

#[derive(Clone)]
pub struct RechargeService {
    pool: PgPool,
    stripe_service: StripeService,
}

impl RechargeService {
    pub fn new(pool: PgPool, stripe_service: StripeService) -> Self {
        Self {
            pool,
            stripe_service,
        }
    }

    pub async fn create_payment_intent(
        &self,
        user_id: i64,
        request: crate::models::CreatePaymentIntentRequest,
    ) -> AppResult<CreatePaymentIntentResponse> {
        // 验证充值金额
        if ![500, 1000, 2000, 10000].contains(&request.amount) {
            return Err(AppError::ValidationError(
                "The recharge amount must be $5, $10, $20, or $100".to_string(),
            ));
        }

        // 计算奖励金额
        let bonus_amount = calculate_bonus_amount(request.amount);
        let total_amount = request.amount + bonus_amount;

        // 创建Stripe支付意图
        let payment_intent = self
            .stripe_service
            .create_payment_intent(
                request.amount,
                user_id,
                Some("usd".to_string()), // 使用美元作为默认货币
                Some(format!(
                    "User {} recharges ${:.2}",
                    user_id,
                    request.amount as f64 / 100.0
                )),
            )
            .await?;

        // 保存充值记录 (直接绑定枚举到 ENUM 列)
        let status = RechargeStatus::Pending;
        let payment_intent_id_str = payment_intent.id.to_string();
        sqlx::query!(
            r#"
            INSERT INTO recharge_records (
                user_id, stripe_payment_intent_id, amount, bonus_amount,
                total_amount, status
            ) VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            user_id,
            payment_intent_id_str,
            request.amount,
            bonus_amount,
            total_amount,
            status as _
        )
        .execute(&self.pool)
        .await?;

        Ok(CreatePaymentIntentResponse {
            payment_intent_id: payment_intent_id_str,
            client_secret: payment_intent.client_secret.unwrap_or_default(),
            amount: request.amount,
            bonus_amount,
            total_amount,
        })
    }

    pub async fn confirm_recharge(
        &self,
        user_id: i64,
        request: ConfirmRechargeRequest,
    ) -> AppResult<ConfirmRechargeResponse> {
        // 获取Stripe支付状态
        let payment_intent = self
            .stripe_service
            .retrieve_payment_intent(&request.payment_intent_id)
            .await?;

        if payment_intent.status != PaymentIntentStatus::Succeeded {
            return Err(AppError::ValidationError(
                "Payment not successful".to_string(),
            ));
        }

        // 开始事务
        let mut tx = self.pool.begin().await?;

        // 获取充值记录
        let recharge_record = sqlx::query_as!(
            RechargeRecord,
            r#"
            SELECT
                id, user_id, stripe_payment_intent_id, amount, bonus_amount,
                total_amount, status as "status: _",
                stripe_status, created_at, updated_at
            FROM recharge_records
            WHERE stripe_payment_intent_id = $1 AND user_id = $2
            "#,
            request.payment_intent_id,
            user_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        let mut recharge_record = recharge_record
            .ok_or_else(|| AppError::NotFound("Recharge record not found".to_string()))?;

        // 检查是否已经处理过
        if recharge_record.status == RechargeStatus::Succeeded {
            let current_balance: i64 = sqlx::query_scalar!(
                r#"SELECT balance as "balance!: i64" FROM users WHERE id = $1"#,
                user_id
            )
            .fetch_one(&mut *tx)
            .await?;

            return Ok(ConfirmRechargeResponse {
                recharge_record: RechargeRecordResponse::from(recharge_record),
                new_balance: current_balance,
            });
        }

        // 更新充值记录状态 (使用枚举)
        let success_status = RechargeStatus::Succeeded;
        let stripe_status_str = format!("{:?}", payment_intent.status);
        sqlx::query!(
            r#"UPDATE recharge_records SET status = $1, stripe_status = $2 WHERE id = $3"#,
            success_status as _,
            stripe_status_str,
            recharge_record.id.unwrap()
        )
        .execute(&mut *tx)
        .await?;

        // 更新用户余额
        sqlx::query!(
            r#"UPDATE users SET balance = balance + $1 WHERE id = $2"#,
            recharge_record.total_amount,
            user_id
        )
        .execute(&mut *tx)
        .await?;

        // 获取新余额
        let current_balance: i64 = sqlx::query_scalar!(
            r#"SELECT balance as "balance!: i64" FROM users WHERE id = $1"#,
            user_id
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        recharge_record.status = RechargeStatus::Succeeded;

        Ok(ConfirmRechargeResponse {
            recharge_record: RechargeRecordResponse::from(recharge_record),
            new_balance: current_balance,
        })
    }

    pub async fn get_recharge_history(
        &self,
        user_id: i64,
        query: &RechargeQuery,
    ) -> AppResult<PaginatedResponse<RechargeRecordResponse>> {
        let params = PaginationParams::new(query.page, query.per_page);
        let offset = params.get_offset() as i64;
        let limit = params.get_limit() as i64;

        // 获取总数
        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!: i64" FROM recharge_records WHERE user_id = $1"#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        // 获取充值记录列表
        let records = sqlx::query_as!(
            RechargeRecord,
            r#"
            SELECT
                id, user_id, stripe_payment_intent_id, amount, bonus_amount,
                total_amount, status as "status: _",
                stripe_status, created_at, updated_at
            FROM recharge_records
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

        let items: Vec<RechargeRecordResponse> = records
            .into_iter()
            .map(RechargeRecordResponse::from)
            .collect();

        Ok(PaginatedResponse::new(
            items,
            params.page.unwrap_or(1),
            params.page_size.unwrap_or(20),
            total,
        ))
    }

    /// 处理Stripe webhook支付成功事件
    ///
    /// # 参数
    ///
    /// * `payment_intent_id` - Stripe支付意图ID
    /// * `user_id` - 用户ID
    ///
    /// # 返回
    ///
    /// 成功时返回更新后的充值记录和新余额
    pub async fn handle_payment_success_webhook(
        &self,
        payment_intent_id: &str,
        user_id: i64,
    ) -> AppResult<()> {
        // 开始事务
        let mut tx = self.pool.begin().await?;

        // 获取充值记录
        let recharge_record = sqlx::query_as!(
            RechargeRecord,
            r#"
            SELECT
                id, user_id, stripe_payment_intent_id, amount, bonus_amount,
                total_amount, status as "status: _",
                stripe_status, created_at, updated_at
            FROM recharge_records
            WHERE stripe_payment_intent_id = $1 AND user_id = $2
            "#,
            payment_intent_id,
            user_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        let recharge_record = match recharge_record {
            Some(record) => record,
            None => {
                log::warn!(
                    "Recharge record not found for payment_intent_id: {} and user_id: {}",
                    payment_intent_id,
                    user_id
                );
                return Ok(());
            }
        };

        // 检查是否已经处理过
        if recharge_record.status == RechargeStatus::Succeeded {
            log::info!(
                "Payment already processed for payment_intent_id: {}",
                payment_intent_id
            );
            return Ok(());
        }

        // 更新充值记录状态
        let success_status = RechargeStatus::Succeeded;
        sqlx::query!(
            r#"UPDATE recharge_records SET status = $1, stripe_status = $2 WHERE id = $3"#,
            success_status as _,
            "succeeded",
            recharge_record.id.unwrap()
        )
        .execute(&mut *tx)
        .await?;

        // 更新用户余额
        sqlx::query!(
            r#"UPDATE users SET balance = balance + $1 WHERE id = $2"#,
            recharge_record.total_amount,
            user_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        log::info!(
            "Successfully processed payment webhook for user {} with amount {}",
            user_id,
            recharge_record.total_amount.unwrap_or(0)
        );

        Ok(())
    }

    /// 处理Stripe webhook支付失败事件
    ///
    /// # 参数
    ///
    /// * `payment_intent_id` - Stripe支付意图ID
    /// * `user_id` - 用户ID
    pub async fn handle_payment_failure_webhook(
        &self,
        payment_intent_id: &str,
        user_id: i64,
    ) -> AppResult<()> {
        // 更新充值记录状态为失败
        let failed_status = RechargeStatus::Failed;
        let result = sqlx::query!(
            r#"UPDATE recharge_records SET status = $1, stripe_status = $2 WHERE stripe_payment_intent_id = $3 AND user_id = $4"#,
            failed_status as _,
            "failed",
            payment_intent_id,
            user_id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            log::info!(
                "Marked payment as failed for payment_intent_id: {} and user_id: {}",
                payment_intent_id,
                user_id
            );
        } else {
            log::warn!(
                "No recharge record found to mark as failed for payment_intent_id: {} and user_id: {}",
                payment_intent_id,
                user_id
            );
        }

        Ok(())
    }

    /// 处理Stripe webhook支付取消事件
    ///
    /// # 参数
    ///
    /// * `payment_intent_id` - Stripe支付意图ID
    /// * `user_id` - 用户ID
    pub async fn handle_payment_canceled_webhook(
        &self,
        payment_intent_id: &str,
        user_id: i64,
    ) -> AppResult<()> {
        // 更新充值记录状态为取消
        let canceled_status = RechargeStatus::Canceled;
        let result = sqlx::query!(
            r#"UPDATE recharge_records SET status = $1, stripe_status = $2 WHERE stripe_payment_intent_id = $3 AND user_id = $4"#,
            canceled_status as _,
            "canceled",
            payment_intent_id,
            user_id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            log::info!(
                "Marked payment as canceled for payment_intent_id: {} and user_id: {}",
                payment_intent_id,
                user_id
            );
        } else {
            log::warn!(
                "No recharge record found to mark as canceled for payment_intent_id: {} and user_id: {}",
                payment_intent_id,
                user_id
            );
        }

        Ok(())
    }
}

/// 根据充值金额计算奖励金额
fn calculate_bonus_amount(amount: i64) -> i64 {
    match amount {
        10000 => 1500,  // $5 -> $1.50 (15%奖励)
        20000 => 3500,  // $10 -> $3.50 (17.5%奖励)
        30000 => 7500,  // $20 -> $7.50 (25%奖励)
        50000 => 15000, // $100 -> $15.00 (30%奖励)
        _ => 0,
    }
}