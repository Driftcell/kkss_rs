use actix_web::{web, HttpResponse, Result, ResponseError};
use serde_json::json;
use crate::services::SyncService;

#[utoipa::path(
    post,
    path = "/admin/sync/orders",
    tag = "admin",
    params(
        ("start_date" = Option<String>, Query, description = "开始日期 (YYYY-MM-DD HH:mm:ss)"),
        ("end_date" = Option<String>, Query, description = "结束日期 (YYYY-MM-DD HH:mm:ss)")
    ),
    responses(
        (status = 200, description = "同步订单成功"),
        (status = 500, description = "同步失败")
    )
)]
pub async fn sync_orders(
    sync_service: web::Data<SyncService>,
    query: web::Query<serde_json::Value>,
) -> Result<HttpResponse> {
    let start_date = query.get("start_date")
        .and_then(|v| v.as_str())
        .unwrap_or("2024-01-01 00:00:00");
    
    let end_date = query.get("end_date")
        .and_then(|v| v.as_str())
        .unwrap_or("2024-12-31 23:59:59");
    
    match sync_service.sync_orders(start_date, end_date).await {
        Ok(count) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": {
                "processed_count": count
            },
            "message": "订单同步完成"
        }))),
        Err(e) => Ok(e.error_response()),
    }
}

#[utoipa::path(
    post,
    path = "/admin/sync/discount-codes",
    tag = "admin",
    responses(
        (status = 200, description = "同步优惠码成功"),
        (status = 500, description = "同步失败")
    )
)]
pub async fn sync_discount_codes(
    sync_service: web::Data<SyncService>,
) -> Result<HttpResponse> {
    match sync_service.sync_discount_codes().await {
        Ok(count) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": {
                "processed_count": count
            },
            "message": "优惠码同步完成"
        }))),
        Err(e) => Ok(e.error_response()),
    }
}

pub fn admin_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            .route("/sync/orders", web::post().to(sync_orders))
            .route("/sync/discount-codes", web::post().to(sync_discount_codes))
    );
}
