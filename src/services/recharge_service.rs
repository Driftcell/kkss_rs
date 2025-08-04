use sqlx::SqlitePool;
use crate::models::{RechargeRecord, RechargeStatus, CreatePaymentIntentResponse, ConfirmRechargeRequest, ConfirmRechargeResponse, RechargeRecordResponse, RechargeQuery, PaginatedResponse, PaginationParams, calculate_bonus_amount};
use crate::error::{AppError, AppResult};
use crate::external::stripe::{StripeService};

#[derive(Clone)]
pub struct RechargeService {
    pool: SqlitePool,
    stripe_service: StripeService,
}

impl RechargeService {
    pub fn new(pool: SqlitePool, stripe_service: StripeService) -> Self {
        Self { pool, stripe_service }
    }

    pub async fn create_payment_intent(&self, user_id: i64, request: crate::models::CreatePaymentIntentRequest) -> AppResult<CreatePaymentIntentResponse> {
        // 验证充值金额
        if ![10000, 20000, 30000, 50000].contains(&request.amount) {
            return Err(AppError::ValidationError(
                "充值金额只能是 $100, $200, $300, $500".to_string()
            ));
        }

        // 计算奖励金额
        let bonus_amount = calculate_bonus_amount(request.amount);
        let total_amount = request.amount + bonus_amount;

        // 创建Stripe支付意图
        let payment_intent = self.stripe_service
            .create_payment_intent(
                request.amount, 
                user_id, 
                Some("usd".to_string()),  // 使用美元作为默认货币
                Some(format!("用户{}充值${:.2}", user_id, request.amount as f64 / 100.0))
            )
            .await?;

        // 保存充值记录
        let status_str = RechargeStatus::Pending.to_string();
        sqlx::query!(
            r#"
            INSERT INTO recharge_records (
                user_id, stripe_payment_intent_id, amount, bonus_amount,
                total_amount, status
            ) VALUES (?, ?, ?, ?, ?, ?)
            "#,
            user_id,
            payment_intent.id,
            request.amount,
            bonus_amount,
            total_amount,
            status_str
        )
        .execute(&self.pool)
        .await?;

        Ok(CreatePaymentIntentResponse {
            payment_intent_id: payment_intent.id,
            client_secret: payment_intent.client_secret,
            amount: request.amount,
            bonus_amount,
            total_amount,
        })
    }

    pub async fn confirm_recharge(&self, user_id: i64, request: ConfirmRechargeRequest) -> AppResult<ConfirmRechargeResponse> {
        // 获取Stripe支付状态
        let payment_intent = self.stripe_service
            .retrieve_payment_intent(&request.payment_intent_id)
            .await?;

        if payment_intent.status != "succeeded" {
            return Err(AppError::ValidationError("支付未成功".to_string()));
        }

        // 开始事务
        let mut tx = self.pool.begin().await?;

        // 获取充值记录
        let recharge_record = sqlx::query_as!(
            RechargeRecord,
            r#"
            SELECT
                id, user_id, stripe_payment_intent_id, amount, bonus_amount,
                total_amount, status as "status: RechargeStatus",
                stripe_status, created_at, updated_at
            FROM recharge_records
            WHERE stripe_payment_intent_id = ? AND user_id = ?
            "#,
            request.payment_intent_id,
            user_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        let mut recharge_record = recharge_record.ok_or_else(|| {
            AppError::NotFound("充值记录不存在".to_string())
        })?;

        // 检查是否已经处理过
        if recharge_record.status == RechargeStatus::Succeeded {
            let user = sqlx::query!(
                "SELECT balance FROM users WHERE id = ?",
                user_id
            )
            .fetch_one(&mut *tx)
            .await?;

            return Ok(ConfirmRechargeResponse {
                recharge_record: RechargeRecordResponse::from(recharge_record),
                new_balance: user.balance.unwrap_or(0),
            });
        }

        // 更新充值记录状态
        let success_status = RechargeStatus::Succeeded.to_string();
        sqlx::query!(
            "UPDATE recharge_records SET status = ?, stripe_status = ? WHERE id = ?",
            success_status,
            payment_intent.status,
            recharge_record.id
        )
        .execute(&mut *tx)
        .await?;

        // 更新用户余额
        sqlx::query!(
            "UPDATE users SET balance = balance + ? WHERE id = ?",
            recharge_record.total_amount,
            user_id
        )
        .execute(&mut *tx)
        .await?;

        // 获取新余额
        let user = sqlx::query!(
            "SELECT balance FROM users WHERE id = ?",
            user_id
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        recharge_record.status = RechargeStatus::Succeeded;

        Ok(ConfirmRechargeResponse {
            recharge_record: RechargeRecordResponse::from(recharge_record),
            new_balance: user.balance.unwrap_or(0),
        })
    }

    pub async fn get_recharge_history(&self, user_id: i64, query: &RechargeQuery) -> AppResult<PaginatedResponse<RechargeRecordResponse>> {
        let params = PaginationParams::new(query.page, query.per_page);
        let offset = params.get_offset() as i64;
        let limit = params.get_limit() as i64;

        // 获取总数
        let total = sqlx::query!(
            "SELECT COUNT(*) as count FROM recharge_records WHERE user_id = ?",
            user_id
        )
        .fetch_one(&self.pool)
        .await?
        .count;

        // 获取充值记录列表
        let records = sqlx::query_as!(
            RechargeRecord,
            r#"
            SELECT
                id, user_id, stripe_payment_intent_id, amount, bonus_amount,
                total_amount, status as "status: RechargeStatus",
                stripe_status, created_at, updated_at
            FROM recharge_records
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

        let items: Vec<RechargeRecordResponse> = records.into_iter().map(RechargeRecordResponse::from).collect();

        Ok(PaginatedResponse::new(items, params.page.unwrap_or(1), params.page_size.unwrap_or(20), total))
    }
}
