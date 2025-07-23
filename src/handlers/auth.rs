use actix_web::{web, HttpRequest, HttpResponse, Result, ResponseError};
use serde_json::json;
use crate::models::*;
use crate::services::AuthService;

#[utoipa::path(
    post,
    path = "/auth/send-code",
    tag = "auth",
    request_body = SendCodeRequest,
    responses(
        (status = 200, description = "验证码发送成功"),
        (status = 400, description = "请求参数错误"),
        (status = 500, description = "服务器内部错误")
    )
)]
pub async fn send_code(
    auth_service: web::Data<AuthService>,
    request: web::Json<SendCodeRequest>,
) -> Result<HttpResponse> {
    match auth_service.send_verification_code(&request.phone).await {
        Ok(response) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": response,
            "message": "验证码已发送"
        }))),
        Err(e) => Ok(e.error_response()),
    }
}

#[utoipa::path(
    post,
    path = "/auth/register",
    tag = "auth",
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "注册成功", body = AuthResponse),
        (status = 400, description = "请求参数错误"),
        (status = 500, description = "服务器内部错误")
    )
)]
pub async fn register(
    auth_service: web::Data<AuthService>,
    request: web::Json<CreateUserRequest>,
) -> Result<HttpResponse> {
    match auth_service.register(request.into_inner()).await {
        Ok(response) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": response
        }))),
        Err(e) => Ok(e.error_response()),
    }
}

#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "登录成功", body = AuthResponse),
        (status = 401, description = "认证失败"),
        (status = 400, description = "请求参数错误")
    )
)]
pub async fn login(
    auth_service: web::Data<AuthService>,
    request: web::Json<LoginRequest>,
) -> Result<HttpResponse> {
    match auth_service.login(request.into_inner()).await {
        Ok(response) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": response
        }))),
        Err(e) => Ok(e.error_response()),
    }
}

#[utoipa::path(
    post,
    path = "/auth/refresh",
    tag = "auth",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "刷新令牌成功", body = AuthResponse),
        (status = 401, description = "无效的刷新令牌")
    )
)]
pub async fn refresh(
    auth_service: web::Data<AuthService>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let auth_header = req.headers().get("Authorization");
    
    let token = if let Some(auth_value) = auth_header {
        if let Ok(auth_str) = auth_value.to_str() {
            if auth_str.starts_with("Bearer ") {
                &auth_str[7..]
            } else {
                return Ok(HttpResponse::Unauthorized().json(json!({
                    "success": false,
                    "error": {
                        "code": "INVALID_TOKEN_FORMAT",
                        "message": "无效的令牌格式"
                    }
                })));
            }
        } else {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "success": false,
                "error": {
                    "code": "INVALID_TOKEN_FORMAT",
                    "message": "无效的令牌格式"
                }
            })));
        }
    } else {
        return Ok(HttpResponse::Unauthorized().json(json!({
            "success": false,
            "error": {
                "code": "MISSING_TOKEN",
                "message": "缺少刷新令牌"
            }
        })));
    };

    match auth_service.refresh_token(token).await {
        Ok(response) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": response
        }))),
        Err(e) => Ok(e.error_response()),
    }
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    tag = "auth",
    responses(
        (status = 200, description = "登出成功")
    )
)]
pub async fn logout() -> Result<HttpResponse> {
    // 简单的登出响应，实际的令牌失效应该在客户端处理
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "已成功登出"
    })))
}

pub fn auth_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/send-code", web::post().to(send_code))
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/refresh", web::post().to(refresh))
            .route("/logout", web::post().to(logout))
    );
}
