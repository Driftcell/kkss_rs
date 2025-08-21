use crate::entities::{
    month_card_entity as mc, stripe_transaction_entity as st,
    StripeTransactionStatus, StripeTransactionType,
};
use crate::error::{AppError, AppResult};
use crate::external::stripe::StripeService;
use crate::models::{
    CreateMonthCardIntentRequest, CreateMonthCardIntentResponse, ConfirmMonthCardRequest,
    ConfirmMonthCardResponse, MonthCardResponse, StripeTransactionResponse,
    UnifiedConfirmRequest, UnifiedConfirmResponse,
};
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::{DatabaseConnection, Set, TransactionTrait, IntoActiveModel};
use serde_json::{json, Value};
use stripe::PaymentIntentStatus;

#[derive(Clone)]
pub struct StripeTransactionService {
    pool: DatabaseConnection,
    stripe_service: StripeService,
}

impl StripeTransactionService {
    pub fn new(pool: DatabaseConnection, stripe_service: StripeService) -> Self {
        Self {
            pool,
            stripe_service,
        }
    }

    /// Create a Stripe transaction record
    pub async fn create_stripe_transaction(
        &self,
        user_id: i64,
        payment_intent_id: String,
        transaction_type: StripeTransactionType,
        amount: i64,
        metadata: Option<Value>,
    ) -> AppResult<StripeTransactionResponse> {
        let transaction = st::ActiveModel {
            user_id: Set(user_id),
            stripe_payment_intent_id: Set(payment_intent_id),
            transaction_type: Set(transaction_type),
            amount: Set(amount),
            status: Set(StripeTransactionStatus::Pending),
            metadata: Set(metadata),
            ..Default::default()
        }
        .insert(&self.pool)
        .await?;

        Ok(StripeTransactionResponse::from(transaction))
    }

    /// Update transaction status
    pub async fn update_transaction_status(
        &self,
        payment_intent_id: &str,
        status: StripeTransactionStatus,
    ) -> AppResult<()> {
        let transaction = st::Entity::find()
            .filter(st::Column::StripePaymentIntentId.eq(payment_intent_id))
            .one(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Transaction not found".to_string()))?;

        let mut active_transaction = transaction.into_active_model();
        active_transaction.status = Set(status);
        active_transaction.updated_at = Set(Some(Utc::now()));
        active_transaction.update(&self.pool).await?;

        Ok(())
    }

    /// Get transaction by payment intent ID
    pub async fn get_transaction_by_payment_intent(
        &self,
        payment_intent_id: &str,
    ) -> AppResult<StripeTransactionResponse> {
        let transaction = st::Entity::find()
            .filter(st::Column::StripePaymentIntentId.eq(payment_intent_id))
            .one(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Transaction not found".to_string()))?;

        Ok(StripeTransactionResponse::from(transaction))
    }

    /// Create month card payment intent
    pub async fn create_month_card_intent(
        &self,
        user_id: i64,
        request: CreateMonthCardIntentRequest,
    ) -> AppResult<CreateMonthCardIntentResponse> {
        // Define month card pricing
        let amount = 550; // $5.50 in cents
        
        let metadata = json!({
            "user_id": user_id,
            "transaction_type": "month_card",
            "is_subscription": request.is_subscription
        });

        // Create Stripe payment intent
        let payment_intent = self
            .stripe_service
            .create_payment_intent(
                amount,
                user_id,
                Some("usd".to_string()),
                Some(format!("User {} month card purchase", user_id)),
            )
            .await?;

        // Create transaction record
        self.create_stripe_transaction(
            user_id,
            payment_intent.id.to_string(),
            StripeTransactionType::MonthCard,
            amount,
            Some(metadata),
        )
        .await?;

        Ok(CreateMonthCardIntentResponse {
            payment_intent_id: payment_intent.id.to_string(),
            client_secret: payment_intent.client_secret.unwrap_or_default(),
            amount,
            is_subscription: request.is_subscription,
        })
    }

    /// Confirm month card purchase
    pub async fn confirm_month_card(
        &self,
        user_id: i64,
        request: ConfirmMonthCardRequest,
    ) -> AppResult<ConfirmMonthCardResponse> {
        // Verify payment intent
        let payment_intent = self
            .stripe_service
            .retrieve_payment_intent(&request.payment_intent_id)
            .await?;

        if payment_intent.status != PaymentIntentStatus::Succeeded {
            return Err(AppError::ValidationError("Payment not successful".into()));
        }

        let txn = self.pool.begin().await?;

        // Get transaction record
        let transaction = st::Entity::find()
            .filter(st::Column::StripePaymentIntentId.eq(&request.payment_intent_id))
            .filter(st::Column::UserId.eq(user_id))
            .one(&txn)
            .await?
            .ok_or_else(|| AppError::NotFound("Transaction not found".into()))?;

        if transaction.status == StripeTransactionStatus::Succeeded {
            // Already processed
            let month_card = mc::Entity::find()
                .filter(mc::Column::UserId.eq(user_id))
                .filter(mc::Column::IsActive.eq(true))
                .one(&txn)
                .await?
                .ok_or_else(|| AppError::NotFound("Month card not found".into()))?;

            txn.commit().await?;
            return Ok(ConfirmMonthCardResponse {
                month_card: MonthCardResponse::from(month_card),
                transaction: StripeTransactionResponse::from(transaction),
            });
        }

        // Extract metadata
        let metadata = transaction.metadata.as_ref()
            .ok_or_else(|| AppError::ValidationError("Missing transaction metadata".into()))?;
        
        let _is_subscription = metadata.get("is_subscription")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Create month card
        let start_date = Utc::now();
        let end_date = start_date + chrono::Duration::days(30);

        let month_card = mc::ActiveModel {
            user_id: Set(user_id),
            subscription_id: Set(None), // TODO: Handle subscriptions
            product_id: Set("month_card".to_string()),
            price_id: Set("price_month_card".to_string()),
            is_active: Set(true),
            start_date: Set(start_date),
            end_date: Set(end_date),
            cancel_at_period_end: Set(false),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        // Update transaction status
        let mut active_transaction = transaction.into_active_model();
        active_transaction.status = Set(StripeTransactionStatus::Succeeded);
        active_transaction.updated_at = Set(Some(Utc::now()));
        let updated_transaction = active_transaction.update(&txn).await?;

        txn.commit().await?;

        Ok(ConfirmMonthCardResponse {
            month_card: MonthCardResponse::from(month_card),
            transaction: StripeTransactionResponse::from(updated_transaction),
        })
    }

    /// Unified confirm interface for all payment types
    pub async fn unified_confirm(
        &self,
        user_id: i64,
        request: UnifiedConfirmRequest,
    ) -> AppResult<UnifiedConfirmResponse> {
        // Get the transaction to determine its type
        let transaction = self.get_transaction_by_payment_intent(&request.payment_intent_id).await?;

        match transaction.transaction_type {
            StripeTransactionType::Recharge => {
                // Handle recharge confirmation
                // We'll need to integrate with existing recharge service
                let details = json!({
                    "message": "Use specific recharge confirm endpoint"
                });
                Ok(UnifiedConfirmResponse {
                    transaction_type: StripeTransactionType::Recharge,
                    transaction,
                    details,
                })
            }
            StripeTransactionType::Membership => {
                // Handle membership confirmation
                // We'll need to integrate with existing membership service
                let details = json!({
                    "message": "Use specific membership confirm endpoint"
                });
                Ok(UnifiedConfirmResponse {
                    transaction_type: StripeTransactionType::Membership,
                    transaction,
                    details,
                })
            }
            StripeTransactionType::MonthCard => {
                let confirm_request = ConfirmMonthCardRequest {
                    payment_intent_id: request.payment_intent_id,
                };
                let result = self.confirm_month_card(user_id, confirm_request).await?;
                let details = json!({
                    "month_card": result.month_card
                });
                Ok(UnifiedConfirmResponse {
                    transaction_type: StripeTransactionType::MonthCard,
                    transaction: result.transaction,
                    details,
                })
            }
        }
    }

    /// Check if user has active month card
    pub async fn has_active_month_card(&self, user_id: i64) -> AppResult<bool> {
        let now = Utc::now();
        let count = mc::Entity::find()
            .filter(mc::Column::UserId.eq(user_id))
            .filter(mc::Column::IsActive.eq(true))
            .filter(mc::Column::StartDate.lte(now))
            .filter(mc::Column::EndDate.gt(now))
            .count(&self.pool)
            .await?;

        Ok(count > 0)
    }

    /// Get active month card users for daily coupon generation
    pub async fn get_active_month_card_users(&self) -> AppResult<Vec<i64>> {
        let now = Utc::now();
        let users = mc::Entity::find()
            .filter(mc::Column::IsActive.eq(true))
            .filter(mc::Column::StartDate.lte(now))
            .filter(mc::Column::EndDate.gt(now))
            .all(&self.pool)
            .await?;

        Ok(users.into_iter().map(|mc| mc.user_id).collect())
    }
}