use actix_web::{HttpResponse, ResponseError};
use serde_json::json;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("验证错误: {0}")]
    ValidationError(String),

    #[error("认证错误: {0}")]
    AuthError(String),

    #[error("未找到资源: {0}")]
    NotFound(String),

    #[error("权限不足")]
    Forbidden,

    #[error("权限被拒绝")]
    PermissionDenied,

    #[error("外部API错误: {0}")]
    ExternalApiError(String),

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("内部服务器错误: {0}")]
    InternalError(String),

    #[error("JWT错误: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),

    #[error("HTTP请求错误: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("JSON序列化/反序列化错误: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("迁移错误: {0}")]
    MigrateError(#[from] sqlx::migrate::MigrateError),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let (status_code, error_code, message) = match self {
            AppError::ValidationError(msg) => (
                actix_web::http::StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                msg,
            ),
            AppError::AuthError(msg) => {
                (actix_web::http::StatusCode::UNAUTHORIZED, "AUTH_ERROR", msg)
            }
            AppError::NotFound(msg) => (actix_web::http::StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            AppError::Forbidden => (
                actix_web::http::StatusCode::FORBIDDEN,
                "FORBIDDEN",
                &"权限不足".to_string(),
            ),
            AppError::PermissionDenied => (
                actix_web::http::StatusCode::FORBIDDEN,
                "FORBIDDEN",
                &"权限不足".to_string(),
            ),
            AppError::ExternalApiError(msg) => (
                actix_web::http::StatusCode::BAD_GATEWAY,
                "EXTERNAL_API_ERROR",
                msg,
            ),
            AppError::DatabaseError(err) => {
                log::error!("Database error: {}", err);
                (
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    &"数据库错误".to_string(),
                )
            }
            AppError::MigrateError(err) => {
                log::error!("Migration error: {}", err);
                (
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "MIGRATION_ERROR",
                    &"数据库迁移错误".to_string(),
                )
            }
            _ => {
                log::error!("Internal error: {}", self);
                (
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    &"内部服务器错误".to_string(),
                )
            }
        };

        HttpResponse::build(status_code).json(json!({
            "success": false,
            "error": {
                "code": error_code,
                "message": message
            }
        }))
    }
}
