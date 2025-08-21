use crate::entities::StripeTransactionCategory;
use crate::entities::{
    RechargeStatus, TransactionType, recharge_record_entity as rr,
    sweet_cash_transaction_entity as sct, user_entity as users,
};
use crate::error::{AppError, AppResult};
use crate::external::stripe::StripeService;
use crate::models::{
    ConfirmRechargeRequest, ConfirmRechargeResponse, CreatePaymentIntentResponse,
    PaginatedResponse, PaginationParams, RechargeQuery, RechargeRecordResponse,
};
use crate::services::StripeTransactionService;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
};
use stripe::PaymentIntentStatus;

#[derive(Clone)]
pub struct RechargeService {
    pool: DatabaseConnection,
    stripe_service: StripeService,
    stx_service: StripeTransactionService,
}

impl RechargeService {
    pub fn new(pool: DatabaseConnection, stripe_service: StripeService) -> Self {
        let stx_service = StripeTransactionService::new(pool.clone());
        Self {
            pool,
            stripe_service,
            stx_service,
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
            .create_payment_intent_with_category(
                request.amount,
                user_id,
                "recharge",
                Some("usd".to_string()),
                Some(format!(
                    "User {} recharges ${:.2}",
                    user_id,
                    request.amount as f64 / 100.0
                )),
                None,
            )
            .await?;

        // 保存充值记录 (直接绑定枚举到 ENUM 列)
        let status = RechargeStatus::Pending;
        let payment_intent_id_str = payment_intent.id.to_string();
        let _ = rr::ActiveModel {
            user_id: Set(user_id),
            stripe_payment_intent_id: Set(payment_intent_id_str.clone()),
            amount: Set(request.amount),
            bonus_amount: Set(bonus_amount),
            total_amount: Set(total_amount),
            status: Set(status),
            ..Default::default()
        }
        .insert(&self.pool)
        .await?;

        // 记录 unified stripe transaction
        let _ = self
            .stx_service
            .record_payment_intent(
                user_id,
                StripeTransactionCategory::Recharge,
                &payment_intent.id.to_string(),
                Some(request.amount),
                Some("usd".to_string()),
                Some(format!("{:?}", payment_intent.status)),
                payment_intent.description.clone(),
            )
            .await;

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
        let txn = self.pool.begin().await?;

        // 获取充值记录
        let mut recharge_record = rr::Entity::find()
            .filter(rr::Column::StripePaymentIntentId.eq(request.payment_intent_id.clone()))
            .filter(rr::Column::UserId.eq(user_id))
            .one(&txn)
            .await?
            .ok_or_else(|| AppError::NotFound("Recharge record not found".into()))?;

        // 检查是否已经处理过
        if recharge_record.status == RechargeStatus::Succeeded {
            let current_balance = users::Entity::find_by_id(user_id)
                .one(&txn)
                .await?
                .and_then(|u| u.balance)
                .unwrap_or(0);

            return Ok(ConfirmRechargeResponse {
                recharge_record: RechargeRecordResponse::from(recharge_record),
                new_balance: current_balance,
            });
        }

        // 更新充值记录状态 (使用枚举)
        let success_status = RechargeStatus::Succeeded;
        let stripe_status_str = format!("{:?}", payment_intent.status);
        if let Some(m) = rr::Entity::find_by_id(recharge_record.id).one(&txn).await? {
            let mut am = m.into_active_model();
            am.status = Set(success_status);
            am.stripe_status = Set(Some(stripe_status_str));
            am.update(&txn).await?;
        }

        // 更新用户余额
        if let Some(u) = users::Entity::find_by_id(user_id).one(&txn).await? {
            let cur = u.balance.unwrap_or(0);
            let delta = recharge_record.total_amount;
            let mut am = u.into_active_model();
            am.balance = Set(Some(cur + delta));
            am.update(&txn).await?;
        }

        // 获取新余额
        let current_balance = users::Entity::find_by_id(user_id)
            .one(&txn)
            .await?
            .and_then(|u| u.balance)
            .unwrap_or(0);

        // 记录 sweet_cash_transactions (Earn)
        sct::ActiveModel {
            user_id: Set(user_id),
            transaction_type: Set(TransactionType::Earn),
            amount: Set(recharge_record.total_amount),
            balance_after: Set(current_balance),
            related_order_id: Set(None),
            related_discount_code_id: Set(None),
            description: Set(Some(format!(
                "Recharge confirmed via Stripe {}",
                request.payment_intent_id
            ))),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        txn.commit().await?;

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
        let offset = params.get_offset();
        let limit = params.get_limit();

        // 获取总数
        let total = rr::Entity::find()
            .filter(rr::Column::UserId.eq(user_id))
            .count(&self.pool)
            .await? as i64;

        // 获取充值记录列表
        let models = rr::Entity::find()
            .filter(rr::Column::UserId.eq(user_id))
            .order_by_desc(rr::Column::CreatedAt)
            .limit(limit as u64)
            .offset(offset as u64)
            .all(&self.pool)
            .await?;
        let items: Vec<RechargeRecordResponse> = models
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
        let txn = self.pool.begin().await?;

        // 获取充值记录
        let recharge_record = rr::Entity::find()
            .filter(rr::Column::StripePaymentIntentId.eq(payment_intent_id.to_string()))
            .filter(rr::Column::UserId.eq(user_id))
            .one(&txn)
            .await?;
        let recharge_record = if let Some(m) = recharge_record {
            m
        } else {
            log::warn!(
                "Recharge record not found for payment_intent_id: {payment_intent_id} and user_id: {user_id}"
            );
            return Ok(());
        };

        // 检查是否已经处理过
        if recharge_record.status == RechargeStatus::Succeeded {
            log::info!("Payment already processed for payment_intent_id: {payment_intent_id}");
            return Ok(());
        }

        // 更新充值记录状态
        let success_status = RechargeStatus::Succeeded;
        if let Some(m) = rr::Entity::find_by_id(recharge_record.id).one(&txn).await? {
            let mut am = m.into_active_model();
            am.status = Set(success_status);
            am.stripe_status = Set(Some("succeeded".to_string()));
            am.update(&txn).await?;
        }

        // 更新用户余额
        let mut new_balance_after: Option<i64> = None;
        if let Some(u) = users::Entity::find_by_id(user_id).one(&txn).await? {
            let cur = u.balance.unwrap_or(0);
            let delta = recharge_record.total_amount;
            let mut am = u.into_active_model();
            let updated = cur + delta;
            am.balance = Set(Some(updated));
            am.update(&txn).await?;
            new_balance_after = Some(updated);
        }

        // 记录 sweet_cash_transactions (Earn)
        if let Some(balance_after) = new_balance_after {
            sct::ActiveModel {
                user_id: Set(user_id),
                transaction_type: Set(TransactionType::Earn),
                amount: Set(recharge_record.total_amount),
                balance_after: Set(balance_after),
                related_order_id: Set(None),
                related_discount_code_id: Set(None),
                description: Set(Some(format!(
                    "Recharge succeeded via Stripe {payment_intent_id}"
                ))),
                ..Default::default()
            }
            .insert(&txn)
            .await?;
        }

        txn.commit().await?;

        log::info!(
            "Successfully processed payment webhook for user {} with amount {}",
            user_id,
            recharge_record.total_amount
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
        if let Some(m) = rr::Entity::find()
            .filter(rr::Column::StripePaymentIntentId.eq(payment_intent_id.to_string()))
            .filter(rr::Column::UserId.eq(user_id))
            .one(&self.pool)
            .await?
        {
            let mut am = m.into_active_model();
            am.status = Set(failed_status);
            am.stripe_status = Set(Some("failed".to_string()));
            am.update(&self.pool).await?;
            log::info!(
                "Marked payment as failed for payment_intent_id: {payment_intent_id} and user_id: {user_id}"
            );
        } else {
            log::warn!(
                "No recharge record found to mark as failed for payment_intent_id: {payment_intent_id} and user_id: {user_id}"
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
        if let Some(m) = rr::Entity::find()
            .filter(rr::Column::StripePaymentIntentId.eq(payment_intent_id.to_string()))
            .filter(rr::Column::UserId.eq(user_id))
            .one(&self.pool)
            .await?
        {
            let mut am = m.into_active_model();
            am.status = Set(canceled_status);
            am.stripe_status = Set(Some("canceled".to_string()));
            am.update(&self.pool).await?;
            log::info!(
                "Marked payment as canceled for payment_intent_id: {payment_intent_id} and user_id: {user_id}"
            );
        } else {
            log::warn!(
                "No recharge record found to mark as canceled for payment_intent_id: {payment_intent_id} and user_id: {user_id}"
            );
        }

        Ok(())
    }
}

/// 根据充值金额计算奖励金额
fn calculate_bonus_amount(amount: i64) -> i64 {
    match amount {
        500 => 50,     // $5 -> $0.5
        1000 => 200,   // $10 -> $2
        2000 => 400,   // $20 -> $4
        10000 => 2500, // $100 -> $25
        _ => 0,
    }
}
