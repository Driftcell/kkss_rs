use crate::entities::StripeTransactionCategory;
use crate::entities::{MonthlyCardStatus, monthly_card_entity as mc};
use crate::error::{AppError, AppResult};
use crate::external::StripeService;
use crate::models::*;
use crate::services::{DiscountCodeService, StripeTransactionService};
use chrono::{Duration, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, Set, TransactionTrait,
};

#[derive(Clone)]
pub struct MonthlyCardService {
    pool: DatabaseConnection,
    stripe_service: StripeService,
    discount_code_service: DiscountCodeService,
    stx_service: StripeTransactionService,
}

impl MonthlyCardService {
    pub fn new(
        pool: DatabaseConnection,
        stripe_service: StripeService,
        discount_code_service: DiscountCodeService,
    ) -> Self {
        let stx_service = StripeTransactionService::new(pool.clone());
        Self {
            pool,
            stripe_service,
            discount_code_service,
            stx_service,
        }
    }

    fn monthly_card_price_cents() -> i64 {
        2000
    }

    pub async fn create_monthly_card_intent(
        &self,
        user_id: i64,
        req: CreateMonthlyCardIntentRequest,
    ) -> AppResult<CreateMonthlyCardIntentResponse> {
        let amount = Self::monthly_card_price_cents();
        // Create PaymentIntent for one_time plan. Subscription flow TBD.
        let mut extra = std::collections::HashMap::new();
        extra.insert("plan_type".to_string(), req.plan_type.to_string());
        let pi = self
            .stripe_service
            .create_payment_intent_with_category(
                amount,
                user_id,
                "monthly_card",
                Some("usd".to_string()),
                Some(format!(
                    "User {user_id} buys monthly card ({})",
                    req.plan_type
                )),
                Some(extra),
            )
            .await?;

        let status = MonthlyCardStatus::Pending;
        let _ = mc::ActiveModel {
            user_id: Set(user_id),
            plan_type: Set(req.plan_type.clone()),
            status: Set(status),
            ..Default::default()
        }
        .insert(&self.pool)
        .await?;

        // record stripe tx
        let _ = self
            .stx_service
            .record_payment_intent(
                user_id,
                StripeTransactionCategory::MonthlyCard,
                &pi.id.to_string(),
                Some(amount),
                Some("usd".to_string()),
                Some(format!("{:?}", pi.status)),
                pi.description.clone(),
            )
            .await;

        Ok(CreateMonthlyCardIntentResponse {
            payment_intent_id: pi.id.to_string(),
            client_secret: pi.client_secret.clone().unwrap_or_default(),
            amount,
            plan_type: req.plan_type,
        })
    }

    pub async fn confirm_monthly_card(
        &self,
        user_id: i64,
        req: ConfirmMonthlyCardRequest,
    ) -> AppResult<ConfirmMonthlyCardResponse> {
        let pi = self
            .stripe_service
            .retrieve_payment_intent(&req.payment_intent_id)
            .await?;
        if pi.status != stripe::PaymentIntentStatus::Succeeded {
            return Err(AppError::ValidationError("Payment not successful".into()));
        }
        let txn = self.pool.begin().await?;
        // pick the latest pending record for user
        let rec = mc::Entity::find()
            .filter(mc::Column::UserId.eq(user_id))
            .order_by_desc(mc::Column::CreatedAt)
            .one(&txn)
            .await?
            .ok_or_else(|| AppError::NotFound("Monthly card record not found".into()))?;
        if rec.status == MonthlyCardStatus::Active {
            let resp = MonthlyCardRecordResponse::from(rec);
            return Ok(ConfirmMonthlyCardResponse { monthly_card: resp });
        }
        let mut am = rec.into_active_model();
        am.status = Set(MonthlyCardStatus::Active);
        am.starts_at = Set(Some(Utc::now()));
        am.ends_at = Set(Some(Utc::now() + Duration::days(30)));
        am.update(&txn).await?;
        txn.commit().await?;
        let rec = mc::Entity::find()
            .filter(mc::Column::UserId.eq(user_id))
            .order_by_desc(mc::Column::CreatedAt)
            .one(&self.pool)
            .await?
            .unwrap();
        Ok(ConfirmMonthlyCardResponse {
            monthly_card: MonthlyCardRecordResponse::from(rec),
        })
    }

    /// 每日为活跃月卡用户发放 $5.5 优惠码，保证一天 1 次。
    pub async fn grant_daily_coupons(&self) -> AppResult<i64> {
        let today = Utc::now().date_naive();
        let active_cards = mc::Entity::find()
            .filter(mc::Column::Status.eq(MonthlyCardStatus::Active))
            .filter(mc::Column::EndsAt.gte(Utc::now()))
            .all(&self.pool)
            .await?;
        let mut granted = 0i64;
        for card in active_cards {
            if card.last_coupon_granted_on == Some(today) {
                continue;
            }
            // 发放 550 cents 优惠码，有效期 1 个月
            self.discount_code_service
                .create_user_discount_code(
                    card.user_id,
                    550,
                    crate::entities::CodeType::SweetsCreditsReward,
                    1,
                )
                .await?;
            let mut am = card.into_active_model();
            am.last_coupon_granted_on = Set(Some(today));
            am.update(&self.pool).await?;
            granted += 1;
        }
        Ok(granted)
    }

    /// 订阅续费成功，延长有效期 30 天
    pub async fn renew_by_subscription(&self, subscription_id: &str) -> AppResult<()> {
        if let Some(card) = mc::Entity::find()
            .filter(mc::Column::StripeSubscriptionId.eq(subscription_id.to_string()))
            .one(&self.pool)
            .await?
        {
            let mut am = card.clone().into_active_model();
            let base = card.ends_at.unwrap_or(Utc::now());
            am.ends_at = Set(Some(base + Duration::days(30)));
            am.status = Set(MonthlyCardStatus::Active);
            am.update(&self.pool).await?;
        }
        Ok(())
    }
}
