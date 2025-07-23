use actix_web::{web, HttpRequest, HttpResponse, Result, HttpMessage, ResponseError};
use serde_json::json;
use crate::models::*;
use crate::services::RechargeService;

fn get_user_id_from_request(req: &HttpRequest) -> Option<i64> {
    req.extensions().get::<i64>().copied()
}

#[utoipa::path(
    post,
    path = "/recharge/create-payment-intent",
    tag = "recharge",
    request_body = CreatePaymentIntentRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "创建支付意图成功", body = CreatePaymentIntentResponse),
        (status = 401, description = "未授权"),
        (status = 400, description = "请求参数错误")
    )
)]
pub async fn create_payment_intent(
    recharge_service: web::Data<RechargeService>,
    req: HttpRequest,
    request: web::Json<CreatePaymentIntentRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id_from_request(&req).unwrap_or(0);
    
    match recharge_service.create_payment_intent(user_id, request.into_inner()).await {
        Ok(response) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": response
        }))),
        Err(e) => Ok(e.error_response()),
    }
}

#[utoipa::path(
    post,
    path = "/recharge/confirm",
    tag = "recharge",
    request_body = ConfirmRechargeRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "确认充值成功", body = ConfirmRechargeResponse),
        (status = 401, description = "未授权"),
        (status = 400, description = "请求参数错误")
    )
)]
pub async fn confirm_recharge(
    recharge_service: web::Data<RechargeService>,
    req: HttpRequest,
    request: web::Json<ConfirmRechargeRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id_from_request(&req).unwrap_or(0);
    
    match recharge_service.confirm_recharge(user_id, request.into_inner()).await {
        Ok(response) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": response
        }))),
        Err(e) => Ok(e.error_response()),
    }
}

#[utoipa::path(
    get,
    path = "/recharge/history",
    tag = "recharge",
    params(
        ("page" = Option<u32>, Query, description = "页码"),
        ("per_page" = Option<u32>, Query, description = "每页数量")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "获取充值历史成功"),
        (status = 401, description = "未授权")
    )
)]
pub async fn get_history(
    recharge_service: web::Data<RechargeService>,
    req: HttpRequest,
    query: web::Query<RechargeQuery>,
) -> Result<HttpResponse> {
    let user_id = get_user_id_from_request(&req).unwrap_or(0);
    
    match recharge_service.get_recharge_history(user_id, &query).await {
        Ok(response) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": response
        }))),
        Err(e) => Ok(e.error_response()),
    }
}

pub fn recharge_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/recharge")
            .route("/create-payment-intent", web::post().to(create_payment_intent))
            .route("/confirm", web::post().to(confirm_recharge))
            .route("/history", web::get().to(get_history))
    );
}
