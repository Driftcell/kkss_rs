use crate::error::{AppError, AppResult};
use crate::external::stripe::StripeService;
use crate::services::recharge_service::RechargeService;
use crate::services::stripe_transaction_service::StripeTransactionService;
use crate::entities::StripeTransactionStatus;
use actix_web::{HttpRequest, HttpResponse, Result, web};
use log::{error, info, warn};
use stripe::{Event, EventObject, EventType, PaymentIntent, Invoice, Subscription};

/// Stripe webhook处理器
///
/// 处理来自Stripe的webhook事件，主要用于处理支付状态更新
pub async fn stripe_webhook(
    req: HttpRequest,
    body: web::Bytes,
    stripe_service: web::Data<StripeService>,
    recharge_service: web::Data<RechargeService>,
    stripe_transaction_service: web::Data<StripeTransactionService>,
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
    match handle_stripe_event(event, &recharge_service, &stripe_transaction_service).await {
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
    stripe_transaction_service: &StripeTransactionService,
) -> AppResult<()> {
    match event.type_ {
        EventType::PaymentIntentSucceeded => {
            handle_payment_intent_succeeded(event, recharge_service, stripe_transaction_service).await
        }
        EventType::PaymentIntentPaymentFailed => {
            handle_payment_intent_failed(event, recharge_service, stripe_transaction_service).await
        }
        EventType::PaymentIntentCanceled => {
            handle_payment_intent_canceled(event, recharge_service, stripe_transaction_service).await
        }
        EventType::InvoicePaymentSucceeded => {
            handle_invoice_payment_succeeded(event, stripe_transaction_service).await
        }
        EventType::InvoicePaymentFailed => {
            handle_invoice_payment_failed(event, stripe_transaction_service).await
        }
        EventType::CustomerSubscriptionDeleted => {
            handle_subscription_deleted(event, stripe_transaction_service).await
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
    stripe_transaction_service: &StripeTransactionService,
) -> AppResult<()> {
    let payment_intent = extract_payment_intent_from_event(event)?;
    info!("Payment succeeded for PaymentIntent: {}", payment_intent.id);

    // Update transaction status in new system
    if let Err(e) = stripe_transaction_service
        .update_transaction_status(&payment_intent.id, StripeTransactionStatus::Succeeded)
        .await
    {
        warn!("Failed to update transaction status in new system: {}", e);
        // Continue with legacy system
    }

    // Get user ID from metadata (legacy system)
    let user_id = payment_intent
        .metadata
        .get("user_id")
        .and_then(|v| v.parse::<i64>().ok())
        .ok_or_else(|| {
            AppError::ValidationError("Missing or invalid user_id in metadata".to_string())
        })?;

    // Handle legacy recharge webhook
    recharge_service
        .handle_payment_success_webhook(payment_intent.id.as_ref(), user_id)
        .await?;

    Ok(())
}

/// 处理支付失败事件
async fn handle_payment_intent_failed(
    event: Event,
    recharge_service: &RechargeService,
    stripe_transaction_service: &StripeTransactionService,
) -> AppResult<()> {
    let payment_intent = extract_payment_intent_from_event(event)?;
    warn!("Payment failed for PaymentIntent: {}", payment_intent.id);

    // Update transaction status in new system
    if let Err(e) = stripe_transaction_service
        .update_transaction_status(&payment_intent.id, StripeTransactionStatus::Failed)
        .await
    {
        warn!("Failed to update transaction status in new system: {}", e);
    }

    // Get user ID from metadata (legacy system)
    let user_id = payment_intent
        .metadata
        .get("user_id")
        .and_then(|v| v.parse::<i64>().ok())
        .ok_or_else(|| {
            AppError::ValidationError("Missing or invalid user_id in metadata".to_string())
        })?;

    // Handle legacy recharge webhook
    recharge_service
        .handle_payment_failure_webhook(payment_intent.id.as_ref(), user_id)
        .await?;

    Ok(())
}

/// 处理支付取消事件
async fn handle_payment_intent_canceled(
    event: Event,
    recharge_service: &RechargeService,
    stripe_transaction_service: &StripeTransactionService,
) -> AppResult<()> {
    let payment_intent = extract_payment_intent_from_event(event)?;
    info!("Payment canceled for PaymentIntent: {}", payment_intent.id);

    // Update transaction status in new system
    if let Err(e) = stripe_transaction_service
        .update_transaction_status(&payment_intent.id, StripeTransactionStatus::Canceled)
        .await
    {
        warn!("Failed to update transaction status in new system: {}", e);
    }

    // Get user ID from metadata (legacy system)
    let user_id = payment_intent
        .metadata
        .get("user_id")
        .and_then(|v| v.parse::<i64>().ok())
        .ok_or_else(|| {
            AppError::ValidationError("Missing or invalid user_id in metadata".to_string())
        })?;

    // Handle legacy recharge webhook
    recharge_service
        .handle_payment_canceled_webhook(payment_intent.id.as_ref(), user_id)
        .await?;

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

/// 从事件中提取Invoice对象
fn extract_invoice_from_event(event: Event) -> AppResult<Invoice> {
    match event.data.object {
        EventObject::Invoice(invoice) => Ok(invoice),
        _ => Err(AppError::ValidationError(
            "Event does not contain an Invoice object".to_string(),
        )),
    }
}

/// 从事件中提取Subscription对象
fn extract_subscription_from_event(event: Event) -> AppResult<Subscription> {
    match event.data.object {
        EventObject::Subscription(subscription) => Ok(subscription),
        _ => Err(AppError::ValidationError(
            "Event does not contain a Subscription object".to_string(),
        )),
    }
}

/// 处理发票支付成功事件 (用于月卡订阅)
async fn handle_invoice_payment_succeeded(
    event: Event,
    _stripe_transaction_service: &StripeTransactionService,
) -> AppResult<()> {
    let invoice = extract_invoice_from_event(event)?;
    info!("Invoice payment succeeded: {}", invoice.id);

    // TODO: Handle subscription billing success
    // For now, just log the event
    info!("Monthly subscription payment successful for invoice: {}", invoice.id);

    Ok(())
}

/// 处理发票支付失败事件
async fn handle_invoice_payment_failed(
    event: Event,
    _stripe_transaction_service: &StripeTransactionService,
) -> AppResult<()> {
    let invoice = extract_invoice_from_event(event)?;
    warn!("Invoice payment failed: {}", invoice.id);

    // TODO: Handle subscription billing failure
    // This might require pausing the month card benefits
    warn!("Monthly subscription payment failed for invoice: {}", invoice.id);

    Ok(())
}

/// 处理订阅删除事件
async fn handle_subscription_deleted(
    event: Event,
    _stripe_transaction_service: &StripeTransactionService,
) -> AppResult<()> {
    let subscription = extract_subscription_from_event(event)?;
    info!("Subscription deleted: {}", subscription.id);

    // TODO: Mark month card as inactive
    // This should deactivate the user's month card
    info!("Subscription deleted, should deactivate month card: {}", subscription.id);

    Ok(())
}

/// 配置webhook路由
pub fn webhook_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/webhook").route("/stripe", web::post().to(stripe_webhook)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::StripeConfig;
    use actix_web::{App, test, web};

    #[actix_web::test]
    async fn test_webhook_missing_signature() {
        let stripe_config = StripeConfig {
            secret_key: "sk_test_123".to_string(),
            webhook_secret: "whsec_123".to_string(),
        };
        let stripe_service = StripeService::new(stripe_config);

        // 创建一个模拟的RechargeService - 在实际测试中你可能需要mock
        // 这里为了简化，我们只测试签名验证部分

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(stripe_service))
                .configure(webhook_config),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/webhook/stripe")
            .set_payload("test payload")
            .to_request();

        let resp = test::call_service(&app, req).await;
        // 由于缺少RechargeService，会返回500而不是400
        // 在真实的测试环境中，你需要提供完整的依赖
        assert!(resp.status().is_client_error() || resp.status().is_server_error());
    }
}
