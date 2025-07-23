use actix_cors::Cors;
use actix_web::http;

pub fn create_cors() -> Cors {
    Cors::default()
        .allowed_origin_fn(|_, _req_head| {
            // 在生产环境中应该限制允许的域名
            true
        })
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allowed_headers(vec![
            http::header::AUTHORIZATION,
            http::header::ACCEPT,
            http::header::CONTENT_TYPE,
        ])
        .max_age(3600)
}
