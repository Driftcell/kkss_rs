use crate::entities::{
    CodeType, MemberType, MembershipPurchaseStatus, membership_purchase_entity as mp,
    user_entity as users, stripe_transaction_entity as st, StripeTransactionStatus, StripeTransactionType,
};
use crate::error::{AppError, AppResult};
use crate::external::StripeService;
use crate::models::*;
use crate::services::DiscountCodeService;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    Set, TransactionTrait,
};
use serde_json::json;
use stripe::PaymentIntentStatus;

#[derive(Clone)]
pub struct MembershipService {
    pool: DatabaseConnection,
    stripe_service: StripeService,
    discount_code_service: DiscountCodeService,
}

impl MembershipService {
    pub fn new(
        pool: DatabaseConnection,
        stripe_service: StripeService,
        discount_code_service: DiscountCodeService,
    ) -> Self {
        Self {
            pool,
            stripe_service,
            discount_code_service,
        }
    }

    fn membership_price_cents(target: &MemberType) -> Option<i64> {
        match target {
            MemberType::SweetShareholder => Some(800),  // $8
            MemberType::SuperShareholder => Some(3000), // $30
            MemberType::Fan => None,                    // 不允许购买回Fan
        }
    }

    pub async fn create_membership_intent(
        &self,
        user_id: i64,
        req: CreateMembershipIntentRequest,
    ) -> AppResult<CreateMembershipIntentResponse> {
        // 查询当前用户会员类型
        let current: MemberType = users::Entity::find_by_id(user_id)
            .one(&self.pool)
            .await?
            .map(|u| u.member_type)
            .ok_or_else(|| AppError::NotFound("User not found".into()))?;

        // 不允许降级或重复购买同级
        if current == req.target_member_type {
            return Err(AppError::ValidationError("Already this membership".into()));
        }
        // 只能从 fan 升级到 sweet 或 super；从 sweet 升到 super
        if current == MemberType::SuperShareholder {
            return Err(AppError::ValidationError(
                "Already highest membership".into(),
            ));
        }
        if current == MemberType::SweetShareholder
            && req.target_member_type == MemberType::SweetShareholder
        {
            return Err(AppError::ValidationError(
                "Already sweet shareholder".into(),
            ));
        }
        if current == MemberType::Fan && req.target_member_type == MemberType::Fan {
            return Err(AppError::ValidationError(
                "Invalid target membership".into(),
            ));
        }
        if current == MemberType::SweetShareholder && req.target_member_type == MemberType::Fan {
            return Err(AppError::ValidationError("Cannot downgrade".into()));
        }

        let target_type = req.target_member_type.clone();
        let amount = Self::membership_price_cents(&target_type)
            .ok_or_else(|| AppError::ValidationError("Unsupported target member type".into()))?;

        let payment_intent = self
            .stripe_service
            .create_payment_intent(
                amount,
                user_id,
                Some("usd".to_string()),
                Some(format!(
                    "User {} upgrade to {}",
                    user_id, req.target_member_type
                )),
            )
            .await?;

        let status = MembershipPurchaseStatus::Pending;
        let payment_intent_id = payment_intent.id.to_string();
        // upsert-like: try insert, ignore unique conflict
        let _ = mp::ActiveModel {
            user_id: Set(user_id),
            stripe_payment_intent_id: Set(payment_intent_id.clone()),
            target_member_type: Set(req.target_member_type.clone()),
            amount: Set(amount),
            status: Set(status),
            ..Default::default()
        }
        .insert(&self.pool)
        .await
        .ok();

        // 创建新的 Stripe 交易记录
        let metadata = json!({
            "user_id": user_id,
            "transaction_type": "membership",
            "target_member_type": req.target_member_type,
            "current_member_type": current
        });
        
        let _ = st::ActiveModel {
            user_id: Set(user_id),
            stripe_payment_intent_id: Set(payment_intent_id.clone()),
            transaction_type: Set(StripeTransactionType::Membership),
            amount: Set(amount),
            status: Set(StripeTransactionStatus::Pending),
            metadata: Set(Some(metadata)),
            ..Default::default()
        }
        .insert(&self.pool)
        .await?;

        Ok(CreateMembershipIntentResponse {
            payment_intent_id,
            client_secret: payment_intent.client_secret.unwrap_or_default(),
            amount,
            target_member_type: target_type,
        })
    }

    pub async fn confirm_membership(
        &self,
        user_id: i64,
        req: ConfirmMembershipRequest,
    ) -> AppResult<ConfirmMembershipResponse> {
        // 查询 intent
        let payment_intent = self
            .stripe_service
            .retrieve_payment_intent(&req.payment_intent_id)
            .await?;
        if payment_intent.status != PaymentIntentStatus::Succeeded {
            return Err(AppError::ValidationError("Payment not successful".into()));
        }

        let txn = self.pool.begin().await?;
        // 读取记录
        let rec_m = mp::Entity::find()
            .filter(mp::Column::StripePaymentIntentId.eq(req.payment_intent_id.clone()))
            .filter(mp::Column::UserId.eq(user_id))
            .one(&txn)
            .await?
            .ok_or_else(|| AppError::NotFound("Membership purchase record not found".into()))?;
        let mut rec = rec_m;

        if rec.status == MembershipPurchaseStatus::Succeeded {
            // 已经处理，直接返回用户当前会员类型
            let mt = users::Entity::find_by_id(user_id)
                .one(&txn)
                .await?
                .map(|u| u.member_type)
                .unwrap_or(MemberType::Fan);
            let resp = MembershipPurchaseRecordResponse::from(rec);
            return Ok(ConfirmMembershipResponse {
                membership_record: resp,
                new_member_type: mt,
            });
        }

        // 升级用户会员类型并设置到期时间为NOW() + 1 year
        let new_member_type = rec.target_member_type.clone();
        if let Some(u) = users::Entity::find_by_id(user_id).one(&txn).await? {
            let mut am = u.into_active_model();
            am.member_type = Set(new_member_type.clone());
            let next = chrono::Utc::now() + chrono::Duration::days(365);
            am.membership_expires_at = Set(Some(next));
            am.update(&txn).await?;
        }

        // 更新记录状态
        let success = MembershipPurchaseStatus::Succeeded;
        if let Some(m) = mp::Entity::find_by_id(rec.id).one(&txn).await? {
            let mut am = m.into_active_model();
            am.status = Set(success);
            am.stripe_status = Set(Some(format!("{:?}", payment_intent.status)));
            am.update(&txn).await?;
        }

        // 发放福利（使用 DiscountCodeService 以保持统一逻辑 & 外部七云同步）
        match new_member_type {
            MemberType::SweetShareholder => {
                // 1 个 $8 优惠码，有效期 1 个月
                // 800 cents, code_type: ShareholderReward
                self.discount_code_service
                    .create_user_discount_code(user_id, 800, CodeType::ShareholderReward, 1)
                    .await?;
            }
            MemberType::SuperShareholder => {
                // 10 个 $3 优惠码，并发创建以减少等待时间（注意：部分失败不会回滚已成功的）
                let mut handles = Vec::with_capacity(10);
                for _ in 0..10 {
                    let svc = self.discount_code_service.clone();
                    handles.push(tokio::spawn(async move {
                        svc.create_user_discount_code(
                            user_id,
                            300,
                            CodeType::SuperShareholderReward,
                            1,
                        )
                        .await
                    }));
                }
                for h in handles {
                    match h.await {
                        Ok(Ok(_id)) => {}
                        Ok(Err(e)) => {
                            log::error!(
                                "Failed to create one of super shareholder discount codes: {e:?}"
                            );
                            return Err(e);
                        }
                        Err(join_err) => {
                            return Err(AppError::InternalError(format!(
                                "Join error creating discount codes: {join_err}"
                            )));
                        }
                    }
                }
            }
            MemberType::Fan => {
                unreachable!("Fan membership should not reach here")
            }
        }

        txn.commit().await?;
        rec.status = MembershipPurchaseStatus::Succeeded;
        let new_type = new_member_type;
        let resp = MembershipPurchaseRecordResponse::from(rec);
        Ok(ConfirmMembershipResponse {
            membership_record: resp,
            new_member_type: new_type,
        })
    }

    /// 将已过期的会员降级为 Fan，返回处理的用户数量
    pub async fn expire_memberships(&self) -> AppResult<i64> {
        // approximate bulk update by scanning and updating; for simplicity
        let to_downgrade = users::Entity::find()
            .filter(users::Column::MembershipExpiresAt.lte(chrono::Utc::now()))
            .filter(users::Column::MembershipExpiresAt.is_not_null())
            .filter(users::Column::MemberType.ne(MemberType::Fan))
            .all(&self.pool)
            .await?;
        let mut count = 0i64;
        for u in to_downgrade {
            let mut am = u.into_active_model();
            am.member_type = Set(MemberType::Fan);
            am.update(&self.pool).await?;
            count += 1;
        }
        Ok(count)
    }
}
