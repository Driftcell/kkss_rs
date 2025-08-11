use crate::models::*;
use crate::services::DiscountCodeService;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, ResponseError, Result, web};
use serde_json::json;

fn get_user_id_from_request(req: &HttpRequest) -> Option<i64> {
    req.extensions().get::<i64>().copied()
}

#[utoipa::path(
    get,
    path = "/discount-codes",
    tag = "discount",
    params(
        ("page" = Option<u32>, Query, description = "页码"),
        ("per_page" = Option<u32>, Query, description = "每页数量"),
        ("status" = Option<String>, Query, description = "状态: available/used/expired"),
        ("code_type" = Option<String>, Query, description = "类型: welcome/referral/purchase_reward/redeemed")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "获取优惠码列表成功"),
        (status = 401, description = "未授权")
    )
)]
pub async fn get_discount_codes(
    discount_service: web::Data<DiscountCodeService>,
    req: HttpRequest,
    query: web::Query<DiscountCodeQuery>,
) -> Result<HttpResponse> {
    let user_id = get_user_id_from_request(&req).unwrap_or(0);

    match discount_service
        .get_user_discount_codes(user_id, &query)
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
    path = "/discount-codes/redeem",
    tag = "discount",
    request_body = RedeemDiscountCodeRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "兑换优惠码成功", body = RedeemDiscountCodeResponse),
        (status = 401, description = "未授权"),
        (status = 400, description = "请求参数错误")
    )
)]
pub async fn redeem_discount_code(
    discount_service: web::Data<DiscountCodeService>,
    req: HttpRequest,
    request: web::Json<RedeemDiscountCodeRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id_from_request(&req).unwrap_or(0);

    match discount_service
        .redeem_discount_code(user_id, request.into_inner())
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
    path = "/discount-codes/redeem-balance",
    tag = "discount",
    request_body = RedeemBalanceDiscountCodeRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "使用余额兑换优惠码成功", body = RedeemBalanceDiscountCodeResponse),
        (status = 401, description = "未授权"),
        (status = 400, description = "请求参数错误")
    )
)]
pub async fn redeem_balance_discount_code(
    discount_service: web::Data<DiscountCodeService>,
    req: HttpRequest,
    request: web::Json<RedeemBalanceDiscountCodeRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id_from_request(&req).unwrap_or(0);

    match discount_service
        .redeem_balance_discount_code(user_id, request.into_inner())
        .await
    {
        Ok(response) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": response
        }))),
        Err(e) => Ok(e.error_response()),
    }
}

pub fn discount_code_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/discount-codes")
            .route("", web::get().to(get_discount_codes))
            .route("/redeem", web::post().to(redeem_discount_code))
            .route("/redeem-balance", web::post().to(redeem_balance_discount_code)),
    );
}
