use crate::entities::StripeTransactionCategory;
use crate::error::{AppError, AppResult};
use crate::external::stripe::StripeService;
use crate::models::{ConfirmMembershipRequest, ConfirmMonthlyCardRequest};
use crate::services::membership_service::MembershipService;
use crate::services::monthly_card_service::MonthlyCardService;
use crate::services::recharge_service::RechargeService;
use crate::services::stripe_transaction_service::StripeTransactionService;
use actix_web::{HttpRequest, HttpResponse, Result, web};
use log::{error, info, warn};
use stripe::{Event, EventObject, EventType, Expandable, PaymentIntent};

/// Stripe webhook处理器
///
/// 处理来自Stripe的webhook事件，主要用于处理支付状态更新
pub async fn stripe_webhook(
    req: HttpRequest,
    body: web::Bytes,
    stripe_service: web::Data<StripeService>,
    recharge_service: web::Data<RechargeService>,
    monthly_service: web::Data<MonthlyCardService>,
    membership_service: web::Data<MembershipService>,
    stx_service: web::Data<StripeTransactionService>,
) -> Result<HttpResponse> {
    let signature = match req.headers().get("stripe-signature") {
        Some(sig) => sig.to_str().unwrap_or(""),
        None => {
            warn!("Missing Stripe-Signature header");
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Missing Stripe-Signature header"
            })));
        }
    };

    let payload = std::str::from_utf8(&body).map_err(|_| {
        error!("Invalid UTF-8 in webhook payload");
        actix_web::error::ErrorBadRequest("Invalid payload encoding")
    })?;

    // 验证webhook签名
    let event = match stripe_service.verify_webhook_signature(payload, signature, 0) {
        Ok(event) => event,
        Err(e) => {
            error!("Webhook signature verification failed: {e}");
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid signature"
            })));
        }
    };

    info!(
        "Received Stripe webhook event: {} ({})",
        event.type_, event.id
    );

    // 处理不同类型的事件
    match handle_stripe_event(
        event,
        &recharge_service,
        &monthly_service,
        &membership_service,
        &stx_service,
    )
    .await
    {
        Ok(_) => {
            info!("Successfully processed webhook event");
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "received": true
            })))
        }
        Err(e) => {
            error!("Failed to process webhook event: {e}");
            // 返回200状态码避免Stripe重试，但记录错误
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "received": true,
                "error": format!("Processing failed: {}", e)
            })))
        }
    }
}

/// 处理具体的Stripe事件
async fn handle_stripe_event(
    event: Event,
    recharge_service: &RechargeService,
    monthly_service: &MonthlyCardService,
    membership_service: &MembershipService,
    stx_service: &StripeTransactionService,
) -> AppResult<()> {
    match event.type_ {
        EventType::PaymentIntentSucceeded => {
            handle_payment_intent_succeeded(
                event,
                recharge_service,
                monthly_service,
                membership_service,
                stx_service,
            )
            .await
        }
        EventType::PaymentIntentPaymentFailed => {
            handle_payment_intent_failed(event, recharge_service, stx_service).await
        }
        EventType::PaymentIntentCanceled => {
            handle_payment_intent_canceled(event, recharge_service, stx_service).await
        }
        EventType::ChargeRefunded => {
            // record refund only
            if let EventObject::Charge(charge) = event.data.object.clone() {
                let user_id = charge
                    .metadata
                    .get("user_id")
                    .and_then(|v| v.parse::<i64>().ok())
                    .unwrap_or(0);
                let category = charge
                    .metadata
                    .get("category")
                    .map(|s| s.as_str())
                    .unwrap_or("recharge");
                let cat = match category {
                    "membership" => StripeTransactionCategory::Membership,
                    "monthly_card" => StripeTransactionCategory::MonthlyCard,
                    _ => StripeTransactionCategory::Recharge,
                };
                let _ = stx_service
                    .record_refund(
                        user_id,
                        cat,
                        charge
                            .refunds
                            .as_ref()
                            .and_then(|r| r.data.first().map(|x| x.id.to_string()))
                            .as_deref()
                            .unwrap_or(""),
                        Some(charge.id.to_string()),
                        Some(charge.amount_refunded),
                        Some(charge.currency.to_string()),
                        Some(format!("{:?}", charge.status)),
                        Some("Charge refunded".to_string()),
                    )
                    .await;
            }
            Ok(())
        }
        EventType::InvoicePaymentSucceeded => {
            // Subscription renewal success
            if let EventObject::Invoice(inv) = event.data.object.clone()
                && let Some(sub) = inv.subscription.as_ref()
            {
                let sid: Option<String> = match sub {
                    Expandable::Id(id) => Some(id.to_string()),
                    Expandable::Object(obj) => Some(obj.id.to_string()),
                };
                if let Some(sub_id) = sid.as_deref() {
                    let _ = monthly_service.renew_by_subscription(sub_id).await;
                }
            }
            Ok(())
        }
        _ => {
            info!("Unhandled event type: {:?}", event.type_);
            Ok(())
        }
    }
}

/// 处理支付成功事件
async fn handle_payment_intent_succeeded(
    event: Event,
    recharge_service: &RechargeService,
    monthly_service: &MonthlyCardService,
    membership_service: &MembershipService,
    stx_service: &StripeTransactionService,
) -> AppResult<()> {
    let payment_intent = extract_payment_intent_from_event(event)?;

    info!("Payment succeeded for PaymentIntent: {}", payment_intent.id);

    // 获取用户ID从metadata
    let user_id = payment_intent
        .metadata
        .get("user_id")
        .and_then(|v| v.parse::<i64>().ok())
        .ok_or_else(|| {
            AppError::ValidationError("Missing or invalid user_id in metadata".to_string())
        })?;

    // 读取业务类别
    let category = payment_intent
        .metadata
        .get("category")
        .map(|s| s.as_str())
        .unwrap_or("recharge");

    info!(
        "Dispatching PaymentIntentSucceeded for user_id={}, category={}",
        user_id, category
    );

    // 记录统一交易表
    let _ = stx_service
        .record_payment_intent(
            user_id,
            match category {
                "membership" => StripeTransactionCategory::Membership,
                "monthly_card" => StripeTransactionCategory::MonthlyCard,
                _ => StripeTransactionCategory::Recharge,
            },
            payment_intent.id.as_ref(),
            Some(payment_intent.amount),
            Some(payment_intent.currency.to_string()),
            Some("succeeded".to_string()),
            payment_intent.description.clone(),
        )
        .await;

    match category {
        "recharge" => {
            // 充值成功
            recharge_service
                .handle_payment_success_webhook(payment_intent.id.as_ref(), user_id)
                .await?;
        }
        "monthly_card" => {
            // 月卡支付成功 -> 激活/确认
            let _ = monthly_service
                .confirm_monthly_card(
                    user_id,
                    ConfirmMonthlyCardRequest {
                        payment_intent_id: payment_intent.id.to_string(),
                    },
                )
                .await?;
        }
        "membership" => {
            // 会员升级支付成功 -> 确认并发放福利
            let _ = membership_service
                .confirm_membership(
                    user_id,
                    ConfirmMembershipRequest {
                        payment_intent_id: payment_intent.id.to_string(),
                    },
                )
                .await?;
        }
        _ => {
            // 其他分类暂不处理
        }
    }

    Ok(())
}

/// 处理支付失败事件
async fn handle_payment_intent_failed(
    event: Event,
    recharge_service: &RechargeService,
    stx_service: &StripeTransactionService,
) -> AppResult<()> {
    let payment_intent = extract_payment_intent_from_event(event)?;

    warn!("Payment failed for PaymentIntent: {}", payment_intent.id);

    // 获取用户ID从metadata
    let user_id = payment_intent
        .metadata
        .get("user_id")
        .and_then(|v| v.parse::<i64>().ok())
        .ok_or_else(|| {
            AppError::ValidationError("Missing or invalid user_id in metadata".to_string())
        })?;

    // 读取业务类别
    let category = payment_intent
        .metadata
        .get("category")
        .map(|s| s.as_str())
        .unwrap_or("recharge");

    // 统一交易表
    let _ = stx_service
        .record_payment_intent(
            user_id,
            match category {
                "membership" => StripeTransactionCategory::Membership,
                "monthly_card" => StripeTransactionCategory::MonthlyCard,
                _ => StripeTransactionCategory::Recharge,
            },
            payment_intent.id.as_ref(),
            Some(payment_intent.amount),
            Some(payment_intent.currency.to_string()),
            Some("failed".to_string()),
            payment_intent.description.clone(),
        )
        .await;

    // 仅对充值分类调用失败处理，避免误伤其他类型
    if category == "recharge" {
        recharge_service
            .handle_payment_failure_webhook(payment_intent.id.as_ref(), user_id)
            .await?;
    }

    Ok(())
}

/// 处理支付取消事件
async fn handle_payment_intent_canceled(
    event: Event,
    recharge_service: &RechargeService,
    stx_service: &StripeTransactionService,
) -> AppResult<()> {
    let payment_intent = extract_payment_intent_from_event(event)?;

    info!("Payment canceled for PaymentIntent: {}", payment_intent.id);

    // 获取用户ID从metadata
    let user_id = payment_intent
        .metadata
        .get("user_id")
        .and_then(|v| v.parse::<i64>().ok())
        .ok_or_else(|| {
            AppError::ValidationError("Missing or invalid user_id in metadata".to_string())
        })?;

    // 读取业务类别
    let category = payment_intent
        .metadata
        .get("category")
        .map(|s| s.as_str())
        .unwrap_or("recharge");

    // 统一交易表
    let _ = stx_service
        .record_payment_intent(
            user_id,
            match category {
                "membership" => StripeTransactionCategory::Membership,
                "monthly_card" => StripeTransactionCategory::MonthlyCard,
                _ => StripeTransactionCategory::Recharge,
            },
            payment_intent.id.as_ref(),
            Some(payment_intent.amount),
            Some(payment_intent.currency.to_string()),
            Some("canceled".to_string()),
            payment_intent.description.clone(),
        )
        .await;

    // 仅对充值分类调用取消处理
    if category == "recharge" {
        recharge_service
            .handle_payment_canceled_webhook(payment_intent.id.as_ref(), user_id)
            .await?;
    }

    Ok(())
}

/// 从事件中提取PaymentIntent对象
fn extract_payment_intent_from_event(event: Event) -> AppResult<PaymentIntent> {
    match event.data.object {
        EventObject::PaymentIntent(payment_intent) => Ok(payment_intent),
        _ => Err(AppError::ValidationError(
            "Event does not contain a PaymentIntent object".to_string(),
        )),
    }
}

/// 配置webhook路由
pub fn webhook_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/webhook").route("/stripe", web::post().to(stripe_webhook)));
}
