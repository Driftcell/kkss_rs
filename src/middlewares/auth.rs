use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use crate::utils::JwtService;
use crate::error::AppError;

pub struct AuthMiddleware {
    jwt_service: JwtService,
}

impl AuthMiddleware {
    pub fn new(jwt_service: JwtService) -> Self {
        Self { jwt_service }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service,
            jwt_service: self.jwt_service.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
    jwt_service: JwtService,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // 跳过不需要认证的路径
        let path = req.path();
        let is_public_path = 
            // 认证相关路径（除了refresh和logout）
            (path.starts_with("/api/v1/auth/") && !path.contains("/refresh") && !path.contains("/logout"))
            // Swagger UI 相关路径 - 更宽松的匹配
            || path.starts_with("/swagger-ui")
            || path == "/swagger-ui"
            || path == "/swagger-ui/"
            || path.starts_with("/api-docs")
            || path == "/api-docs/openapi.json"
            // 静态资源
            || path.ends_with(".css")
            || path.ends_with(".js")
            || path.ends_with(".png")
            || path.ends_with(".ico")
            || path.ends_with(".html")
            || path.ends_with(".woff")
            || path.ends_with(".woff2")
            || path.ends_with(".ttf")
            || path.ends_with(".svg");
            
        if is_public_path {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await });
        }

        // 提取Authorization header
        let auth_header = req.headers().get("Authorization");
        
        let token = if let Some(auth_value) = auth_header {
            if let Ok(auth_str) = auth_value.to_str() {
                if auth_str.starts_with("Bearer ") {
                    Some(&auth_str[7..])
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let jwt_service = self.jwt_service.clone();
        
        if let Some(token) = token {
            match jwt_service.verify_access_token(token) {
                Ok(claims) => {
                    // 将用户ID添加到请求扩展中
                    req.extensions_mut().insert(claims.sub.parse::<i64>().unwrap_or(0));
                    let fut = self.service.call(req);
                    Box::pin(async move { fut.await })
                }
                Err(_) => {
                    let error = AppError::AuthError("无效的访问令牌".to_string());
                    Box::pin(async move { 
                        Err(error.into()) 
                    })
                }
            }
        } else {
            let error = AppError::AuthError("缺少访问令牌".to_string());
            Box::pin(async move { 
                Err(error.into()) 
            })
        }
    }
}

// 用于获取当前用户ID的辅助函数
pub fn get_current_user_id(req: &ServiceRequest) -> Option<i64> {
    req.extensions().get::<i64>().copied()
}
