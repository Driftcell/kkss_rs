use crate::models::*;
use crate::services::{MembershipService, MonthlyCardService, RechargeService};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, ResponseError, Result, web};
use serde_json::json;

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

    match recharge_service
        .create_payment_intent(user_id, request.into_inner())
        .await
    {
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

    match recharge_service
        .confirm_recharge(user_id, request.into_inner())
        .await
    {
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
            .route(
                "/create-payment-intent",
                web::post().to(create_payment_intent),
            )
            .route("/confirm", web::post().to(confirm_recharge))
            .route("/history", web::get().to(get_history)),
    );
}

#[utoipa::path(
    post,
    path = "/membership/create-payment-intent",
    tag = "membership",
    request_body = CreateMembershipIntentRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "创建会员支付意图成功", body = CreateMembershipIntentResponse),
        (status = 401, description = "未授权"),
        (status = 400, description = "请求参数错误")
    )
)]
pub async fn create_membership_payment_intent(
    membership_service: web::Data<MembershipService>,
    req: HttpRequest,
    request: web::Json<CreateMembershipIntentRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id_from_request(&req).unwrap_or(0);
    match membership_service
        .create_membership_intent(user_id, request.into_inner())
        .await
    {
        Ok(resp) => Ok(HttpResponse::Ok().json(json!({"success": true, "data": resp}))),
        Err(e) => Ok(e.error_response()),
    }
}

#[utoipa::path(
    post,
    path = "/membership/confirm",
    tag = "membership",
    request_body = ConfirmMembershipRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "确认会员支付成功", body = ConfirmMembershipResponse),
        (status = 401, description = "未授权"),
        (status = 400, description = "请求参数错误")
    )
)]
pub async fn confirm_membership(
    membership_service: web::Data<MembershipService>,
    req: HttpRequest,
    request: web::Json<ConfirmMembershipRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id_from_request(&req).unwrap_or(0);
    match membership_service
        .confirm_membership(user_id, request.into_inner())
        .await
    {
        Ok(resp) => Ok(HttpResponse::Ok().json(json!({"success": true, "data": resp}))),
        Err(e) => Ok(e.error_response()),
    }
}

pub fn membership_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/membership")
            .route(
                "/create-payment-intent",
                web::post().to(create_membership_payment_intent),
            )
            .route("/confirm", web::post().to(confirm_membership)),
    );
}

#[utoipa::path(
    post,
    path = "/monthly-card/create-payment-intent",
    tag = "monthly_card",
    request_body = CreateMonthlyCardIntentRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "创建月卡支付意图成功", body = CreateMonthlyCardIntentResponse),
        (status = 401, description = "未授权"),
        (status = 400, description = "请求参数错误")
    )
)]
pub async fn create_monthly_card_payment_intent(
    monthly_service: web::Data<MonthlyCardService>,
    req: HttpRequest,
    request: web::Json<CreateMonthlyCardIntentRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id_from_request(&req).unwrap_or(0);
    match monthly_service
        .create_monthly_card_intent(user_id, request.into_inner())
        .await
    {
        Ok(resp) => Ok(HttpResponse::Ok().json(json!({"success": true, "data": resp}))),
        Err(e) => Ok(e.error_response()),
    }
}

#[utoipa::path(
    post,
    path = "/monthly-card/confirm",
    tag = "monthly_card",
    request_body = ConfirmMonthlyCardRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "确认月卡支付成功", body = ConfirmMonthlyCardResponse),
        (status = 401, description = "未授权"),
        (status = 400, description = "请求参数错误")
    )
)]
pub async fn confirm_monthly_card(
    monthly_service: web::Data<MonthlyCardService>,
    req: HttpRequest,
    request: web::Json<ConfirmMonthlyCardRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id_from_request(&req).unwrap_or(0);
    match monthly_service
        .confirm_monthly_card(user_id, request.into_inner())
        .await
    {
        Ok(resp) => Ok(HttpResponse::Ok().json(json!({"success": true, "data": resp}))),
        Err(e) => Ok(e.error_response()),
    }
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct UnifiedConfirmRequest {
    pub category: String,
    pub payment_intent_id: String,
}

#[utoipa::path(
    post,
    path = "/payments/confirm",
    tag = "payments",
    request_body = UnifiedConfirmRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "统一确认成功"),
        (status = 400, description = "请求参数错误")
    )
)]
pub async fn confirm_unified(
    recharge_service: web::Data<RechargeService>,
    membership_service: web::Data<MembershipService>,
    monthly_service: web::Data<MonthlyCardService>,
    req: HttpRequest,
    body: web::Json<UnifiedConfirmRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id_from_request(&req).unwrap_or(0);
    let payload = body.into_inner();
    let resp = match payload.category.as_str() {
        "recharge" => serde_json::to_value(
            recharge_service
                .confirm_recharge(
                    user_id,
                    ConfirmRechargeRequest {
                        payment_intent_id: payload.payment_intent_id,
                    },
                )
                .await?,
        )?,
        "membership" => serde_json::to_value(
            membership_service
                .confirm_membership(
                    user_id,
                    ConfirmMembershipRequest {
                        payment_intent_id: payload.payment_intent_id,
                    },
                )
                .await?,
        )?,
        "monthly_card" => serde_json::to_value(
            monthly_service
                .confirm_monthly_card(
                    user_id,
                    ConfirmMonthlyCardRequest {
                        payment_intent_id: payload.payment_intent_id,
                    },
                )
                .await?,
        )?,
        _ => return Ok(HttpResponse::BadRequest().json(json!({"error": "invalid category"}))),
    };
    Ok(HttpResponse::Ok().json(json!({"success": true, "data": resp})))
}

pub fn monthly_card_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/monthly-card")
            .route(
                "/create-payment-intent",
                web::post().to(create_monthly_card_payment_intent),
            )
            .route("/confirm", web::post().to(confirm_monthly_card)),
    );
}
