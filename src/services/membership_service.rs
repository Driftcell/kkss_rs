use crate::error::{AppError, AppResult};
use crate::external::StripeService;
use crate::models::*;
use crate::services::DiscountCodeService;
use sqlx::PgPool;
use stripe::PaymentIntentStatus;

#[derive(Clone)]
pub struct MembershipService {
    pool: PgPool,
    stripe_service: StripeService,
    discount_code_service: DiscountCodeService,
}

impl MembershipService {
    pub fn new(
        pool: PgPool,
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
            MemberType::SweetShareholder => Some(800), // $8 -> 兑换任意配料冰淇淋优惠码
            MemberType::SuperShareholder => Some(3000), // $30 假设价格，需要根据业务调整
            MemberType::Fan => None,                   // 不允许购买回Fan
        }
    }

    pub async fn create_membership_intent(
        &self,
        user_id: i64,
        req: CreateMembershipIntentRequest,
    ) -> AppResult<CreateMembershipIntentResponse> {
        // 查询当前用户会员类型
        let current: MemberType = sqlx::query_scalar!(
            "SELECT member_type as \"member_type: MemberType\" FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await?
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

        let amount = Self::membership_price_cents(&req.target_member_type)
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
        sqlx::query(
            r#"INSERT INTO membership_purchases (user_id, stripe_payment_intent_id, target_member_type, amount, status)
                VALUES ($1,$2,$3,$4,$5) ON CONFLICT(stripe_payment_intent_id) DO NOTHING"#,
        )
        .bind(user_id)
        .bind(&payment_intent_id)
        .bind(req.target_member_type.to_string())
        .bind(amount)
        .bind(status.to_string())
        .execute(&self.pool).await?;

        Ok(CreateMembershipIntentResponse {
            payment_intent_id,
            client_secret: payment_intent.client_secret.unwrap_or_default(),
            amount,
            target_member_type: req.target_member_type,
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

        let mut tx = self.pool.begin().await?;
        // 读取记录
        let rec = sqlx::query_as::<_, MembershipPurchaseRecord>(
            r#"SELECT id, user_id, stripe_payment_intent_id, target_member_type as "target_member_type: MemberType", amount, status as "status: MembershipPurchaseStatus", stripe_status, created_at, updated_at
               FROM membership_purchases WHERE stripe_payment_intent_id = $1 AND user_id = $2"#,
        )
        .bind(&req.payment_intent_id)
        .bind(user_id)
        .fetch_optional(&mut *tx).await?;
        let mut rec =
            rec.ok_or_else(|| AppError::NotFound("Membership purchase record not found".into()))?;

        if rec.status == MembershipPurchaseStatus::Succeeded {
            // 已经处理，直接返回用户当前会员类型
            let mt: MemberType = sqlx::query_scalar!(
                "SELECT member_type as \"member_type: MemberType\" FROM users WHERE id = $1",
                user_id
            )
            .fetch_one(&mut *tx)
            .await?;
            let resp = MembershipPurchaseRecordResponse::from(rec);
            return Ok(ConfirmMembershipResponse {
                membership_record: resp,
                new_member_type: mt,
            });
        }

        // 升级用户会员类型
        sqlx::query("UPDATE users SET member_type = $1 WHERE id = $2")
            .bind(rec.target_member_type.to_string())
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        // 更新记录状态
        let success = MembershipPurchaseStatus::Succeeded;
        sqlx::query("UPDATE membership_purchases SET status = $1, stripe_status = $2, updated_at = NOW() WHERE id = $3")
            .bind(success.to_string())
            .bind(format!("{:?}", payment_intent.status))
            .bind(rec.id)
            .execute(&mut *tx).await?;

        // 发放福利（使用 DiscountCodeService 以保持统一逻辑 & 外部七云同步）
        match rec.target_member_type {
            MemberType::SweetShareholder => {
                // 1 个 $8 优惠码，有效期 1 个月
                // 800 cents, code_type: PurchaseReward
                self.discount_code_service
                    .create_user_discount_code(user_id, 800, CodeType::PurchaseReward, 1)
                    .await?;
            }
            MemberType::SuperShareholder => {
                // 10 个 $3 优惠码
                for _ in 0..10 {
                    self.discount_code_service
                        .create_user_discount_code(user_id, 300, CodeType::PurchaseReward, 1)
                        .await?;
                }
            }
            MemberType::Fan => { /* 不会进入此分支 */ }
        }

        tx.commit().await?;
        rec.status = MembershipPurchaseStatus::Succeeded;
        let new_type = rec.target_member_type.clone();
        let resp = MembershipPurchaseRecordResponse::from(rec);
        Ok(ConfirmMembershipResponse {
            membership_record: resp,
            new_member_type: new_type,
        })
    }
}
