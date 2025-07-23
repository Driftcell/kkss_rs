use actix_web::{web, HttpRequest, HttpResponse, Result, HttpMessage, ResponseError};
use serde_json::json;
use crate::models::*;
use crate::services::OrderService;

fn get_user_id_from_request(req: &HttpRequest) -> Option<i64> {
    req.extensions().get::<i64>().copied()
}

#[utoipa::path(
    get,
    path = "/orders",
    tag = "order",
    params(
        ("page" = Option<u32>, Query, description = "页码"),
        ("per_page" = Option<u32>, Query, description = "每页数量"),
        ("status" = Option<i32>, Query, description = "订单状态"),
        ("start_date" = Option<String>, Query, description = "开始日期"),
        ("end_date" = Option<String>, Query, description = "结束日期")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "获取订单列表成功"),
        (status = 401, description = "未授权")
    )
)]
pub async fn get_orders(
    order_service: web::Data<OrderService>,
    req: HttpRequest,
    query: web::Query<OrderQuery>,
) -> Result<HttpResponse> {
    let user_id = get_user_id_from_request(&req).unwrap_or(0);
    
    match order_service.get_user_orders(user_id, &query).await {
        Ok(response) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "data": response
        }))),
        Err(e) => Ok(e.error_response()),
    }
}

pub fn order_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/orders")
            .route("", web::get().to(get_orders))
    );
}
