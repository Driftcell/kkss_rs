use crate::models::*;
use crate::services::StripeTransactionService;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, ResponseError, Result, web};
use serde_json::json;

fn get_user_id_from_request(req: &HttpRequest) -> Option<i64> {
    req.extensions().get::<i64>().copied()
}

#[utoipa::path(
    post,
    path = "/month-card/create-payment-intent",
    tag = "month_card",
    request_body = CreateMonthCardIntentRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "创建月卡支付意图成功", body = CreateMonthCardIntentResponse),
        (status = 401, description = "未授权"),
        (status = 400, description = "请求参数错误")
    )
)]
pub async fn create_month_card_intent(
    stripe_transaction_service: web::Data<StripeTransactionService>,
    req: HttpRequest,
    request: web::Json<CreateMonthCardIntentRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id_from_request(&req).unwrap_or(0);
    match stripe_transaction_service
        .create_month_card_intent(user_id, request.into_inner())
        .await
    {
        Ok(resp) => Ok(HttpResponse::Ok().json(json!({"success": true, "data": resp}))),
        Err(e) => Ok(e.error_response()),
    }
}

#[utoipa::path(
    post,
    path = "/month-card/confirm",
    tag = "month_card",
    request_body = ConfirmMonthCardRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "确认月卡支付成功", body = ConfirmMonthCardResponse),
        (status = 401, description = "未授权"),
        (status = 400, description = "请求参数错误")
    )
)]
pub async fn confirm_month_card(
    stripe_transaction_service: web::Data<StripeTransactionService>,
    req: HttpRequest,
    request: web::Json<ConfirmMonthCardRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id_from_request(&req).unwrap_or(0);
    match stripe_transaction_service
        .confirm_month_card(user_id, request.into_inner())
        .await
    {
        Ok(resp) => Ok(HttpResponse::Ok().json(json!({"success": true, "data": resp}))),
        Err(e) => Ok(e.error_response()),
    }
}

#[utoipa::path(
    post,
    path = "/payment/confirm",
    tag = "payment",
    request_body = UnifiedConfirmRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "统一确认支付成功", body = UnifiedConfirmResponse),
        (status = 401, description = "未授权"),
        (status = 400, description = "请求参数错误")
    )
)]
pub async fn unified_confirm(
    stripe_transaction_service: web::Data<StripeTransactionService>,
    req: HttpRequest,
    request: web::Json<UnifiedConfirmRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id_from_request(&req).unwrap_or(0);
    match stripe_transaction_service
        .unified_confirm(user_id, request.into_inner())
        .await
    {
        Ok(resp) => Ok(HttpResponse::Ok().json(json!({"success": true, "data": resp}))),
        Err(e) => Ok(e.error_response()),
    }
}

pub fn month_card_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/month-card")
            .route("/create-payment-intent", web::post().to(create_month_card_intent))
            .route("/confirm", web::post().to(confirm_month_card)),
    );
}

pub fn payment_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/payment")
            .route("/confirm", web::post().to(unified_confirm)),
    );
}