use crate::external::TurnstileService;
use crate::models::*;
use crate::services::AuthService;
use actix_web::{HttpRequest, HttpResponse, ResponseError, Result, web};
use serde_json::json;

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
    turnstile: web::Data<TurnstileService>,
    req: HttpRequest,
    request: web::Json<SendCodeRequest>,
) -> Result<HttpResponse> {
    // 若启用 Turnstile，则要求并校验 token
    if turnstile.as_ref().is_enabled() {
        let token = match &request.cf_turnstile_token {
            Some(t) if !t.is_empty() => t,
            _ => {
                return Ok(crate::error::AppError::ValidationError(
                    "Missing Turnstile token".into(),
                )
                .error_response());
            }
        };

        // 提取客户端 IP（优先 CF-Connecting-IP, 然后 X-Forwarded-For，再从连接信息）
        let header_ip = req
            .headers()
            .get("CF-Connecting-IP")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .or_else(|| {
                req.headers()
                    .get("X-Forwarded-For")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
            });
        let remote_ip = if let Some(ip) = header_ip {
            Some(ip)
        } else {
            req.connection_info()
                .realip_remote_addr()
                .map(|s| s.to_string())
        };
        let remote_ip_ref = remote_ip.as_deref();

        log::info!("Verifying Turnstile token: {token}, IP: {remote_ip_ref:?}");

        // 调用 Turnstile 服务验证
        if let Err(e) = turnstile.verify_token(token, remote_ip_ref, None).await {
            return Ok(e.error_response());
        }
    }

    match auth_service.send_verification_code(&request.phone).await {
        Ok(response) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": response,
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
            if let Some(bearer) = auth_str.strip_prefix("Bearer ") {
                bearer
            } else {
                return Ok(HttpResponse::Unauthorized().json(json!({
                    "success": false,
                    "error": {
                        "code": "INVALID_TOKEN_FORMAT",
                        "message": "Invalid token format"
                    }
                })));
            }
        } else {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "success": false,
                "error": {
                    "code": "INVALID_TOKEN_FORMAT",
                    "message": "Invalid token format"
                }
            })));
        }
    } else {
        return Ok(HttpResponse::Unauthorized().json(json!({
            "success": false,
            "error": {
                "code": "MISSING_TOKEN",
                "message": "Missing authorization token"
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
    path = "/auth/reset-password",
    tag = "auth",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "重设密码成功"),
        (status = 400, description = "请求参数错误"),
        (status = 404, description = "用户不存在"),
        (status = 500, description = "服务器内部错误")
    )
)]
pub async fn reset_password(
    auth_service: web::Data<AuthService>,
    request: web::Json<ResetPasswordRequest>,
) -> Result<HttpResponse> {
    let req = request.into_inner();
    match auth_service
        .reset_password_with_phone_code(&req.phone, &req.verification_code, &req.new_password)
        .await
    {
        Ok(()) => Ok(HttpResponse::Ok().json(json!({"success": true}))),
        Err(e) => Ok(e.error_response()),
    }
}

pub fn auth_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/send-code", web::post().to(send_code))
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/refresh", web::post().to(refresh))
            .route("/reset-password", web::post().to(reset_password)),
    );
}
