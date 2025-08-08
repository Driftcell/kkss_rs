use actix_cors::Cors;

pub fn create_cors() -> Cors {
    Cors::default()
        .allowed_origin_fn(|_, _req_head| {
            // 在生产环境中应该限制允许的域名
            true
        })
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        // 本地开发放宽，防止前端自定义 Header 导致预检失败
        .allow_any_header()
        // 如果前端使用 Cookie（如刷新令牌）、或需要携带凭据，需开启
        .supports_credentials()
        .max_age(3600)
}
