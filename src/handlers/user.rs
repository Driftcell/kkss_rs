use actix_web::{web, HttpRequest, HttpResponse, Result, ResponseError, HttpMessage};
use serde_json::json;
use crate::models::*;
use crate::models::pagination::PaginationParams;
use crate::services::UserService;

fn get_user_id_from_request(req: &HttpRequest) -> Option<i64> {
    req.extensions().get::<i64>().copied()
}

#[utoipa::path(
    get,
    path = "/user/profile",
    tag = "user",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "获取用户资料成功", body = UserResponse),
        (status = 401, description = "未授权"),
        (status = 404, description = "用户不存在")
    )
)]
pub async fn get_profile(
    user_service: web::Data<UserService>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = get_user_id_from_request(&req).unwrap_or(0);
    
    match user_service.get_user_profile(user_id).await {
        Ok((user, statistics)) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": {
                "user": user,
                "statistics": statistics
            }
        }))),
        Err(e) => Ok(e.error_response()),
    }
}

#[utoipa::path(
    put,
    path = "/user/profile",
    tag = "user",
    request_body = UpdateUserRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "更新用户资料成功", body = UserResponse),
        (status = 401, description = "未授权"),
        (status = 400, description = "请求参数错误")
    )
)]
pub async fn update_profile(
    user_service: web::Data<UserService>,
    req: HttpRequest,
    request: web::Json<UpdateUserRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id_from_request(&req).unwrap_or(0);
    
    match user_service.update_user_profile(user_id, request.into_inner()).await {
        Ok(user) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": {
                "user": user
            }
        }))),
        Err(e) => Ok(e.error_response()),
    }
}

#[utoipa::path(
    get,
    path = "/user/referrals",
    tag = "user",
    params(
        ("page" = Option<u32>, Query, description = "页码"),
        ("per_page" = Option<u32>, Query, description = "每页数量")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "获取推荐列表成功"),
        (status = 401, description = "未授权")
    )
)]
pub async fn get_referrals(
    user_service: web::Data<UserService>,
    req: HttpRequest,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse> {
    let user_id = get_user_id_from_request(&req).unwrap_or(0);
    
    match user_service.get_user_referrals(user_id, &query.into_inner()).await {
        Ok(response) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": response
        }))),
        Err(e) => Ok(e.error_response()),
    }
}

pub fn user_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .route("/profile", web::get().to(get_profile))
            .route("/profile", web::put().to(update_profile))
            .route("/referrals", web::get().to(get_referrals))
    );
}
