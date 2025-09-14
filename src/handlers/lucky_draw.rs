use crate::models::*;
use crate::services::LuckyDrawService;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, ResponseError, Result, web};
use serde_json::json;

/// 从请求扩展中获取用户ID（中间件在鉴权后注入）
fn get_user_id_from_request(req: &HttpRequest) -> Option<i64> {
    req.extensions().get::<i64>().copied()
}

#[utoipa::path(
    get,
    path = "/lucky-draw/chances",
    tag = "lucky_draw",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "获取用户抽奖次数成功", body = LuckyDrawChancesResponse),
        (status = 401, description = "未授权")
    )
)]
/// 获取用户当前抽奖次数信息（累计 / 已用 / 剩余）
/// 如果用户从未产生过记录，会自动初始化为0
pub async fn get_chances(
    service: web::Data<LuckyDrawService>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = get_user_id_from_request(&req).unwrap_or(0);
    match service.get_user_chances(user_id).await {
        Ok(data) => Ok(HttpResponse::Ok().json(json!({ "success": true, "data": data }))),
        Err(e) => Ok(e.error_response()),
    }
}

#[utoipa::path(
    get,
    path = "/lucky-draw/prizes",
    tag = "lucky_draw",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "获取奖品列表成功", body = [LuckyDrawPrizeResponse]),
        (status = 401, description = "未授权")
    )
)]
/// 获取当前启用的奖品配置（仅展示基本信息）
pub async fn get_prizes(service: web::Data<LuckyDrawService>) -> Result<HttpResponse> {
    match service.list_prizes().await {
        Ok(list) => Ok(HttpResponse::Ok().json(json!({ "success": true, "data": list }))),
        Err(e) => Ok(e.error_response()),
    }
}

#[utoipa::path(
    get,
    path = "/lucky-draw/records",
    tag = "lucky_draw",
    params(
        ("page" = Option<u32>, Query, description = "页码 (默认1)"),
        ("per_page" = Option<u32>, Query, description = "每页数量 (默认20)")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "获取抽奖记录成功", body = PaginatedResponse<LuckyDrawRecordResponse>),
        (status = 401, description = "未授权")
    )
)]
/// 分页获取用户抽奖记录（倒序）
pub async fn get_records(
    service: web::Data<LuckyDrawService>,
    req: HttpRequest,
    query: web::Query<LuckyDrawRecordQuery>,
) -> Result<HttpResponse> {
    let user_id = get_user_id_from_request(&req).unwrap_or(0);
    match service.list_records(user_id, &query.into_inner()).await {
        Ok(page) => Ok(HttpResponse::Ok().json(json!({ "success": true, "data": page }))),
        Err(e) => Ok(e.error_response()),
    }
}

#[utoipa::path(
    post,
    path = "/lucky-draw/spin",
    tag = "lucky_draw",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "抽奖成功", body = LuckyDrawSpinResponse),
        (status = 400, description = "没有可用次数或其它业务错误"),
        (status = 401, description = "未授权")
    )
)]
/// 进行一次抽奖:
/// 1. 检查剩余次数
/// 2. 根据概率选择奖品（过滤无库存奖品）
/// 3. 限量奖品使用乐观锁扣减库存
/// 4. 生成抽奖记录并返回结果
pub async fn spin(service: web::Data<LuckyDrawService>, req: HttpRequest) -> Result<HttpResponse> {
    let user_id = get_user_id_from_request(&req).unwrap_or(0);
    match service.spin(user_id).await {
        Ok(result) => Ok(HttpResponse::Ok().json(json!({ "success": true, "data": result }))),
        Err(e) => Ok(e.error_response()),
    }
}

/// 路由配置
pub fn lucky_draw_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/lucky-draw")
            .route("/chances", web::get().to(get_chances))
            .route("/prizes", web::get().to(get_prizes))
            .route("/records", web::get().to(get_records))
            .route("/spin", web::post().to(spin)),
    );
}
