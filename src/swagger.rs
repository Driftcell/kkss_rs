use actix_web::web;
use utoipa::OpenApi;
use utoipa::{
    Modify,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers;
use crate::models::*;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        )
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::auth::send_code,
        handlers::auth::register,
        handlers::auth::login,
        handlers::auth::refresh,
        handlers::auth::logout,
        handlers::user::get_profile,
        handlers::user::update_profile,
        handlers::user::get_referrals,
        handlers::order::get_orders,
        handlers::discount_code::get_discount_codes,
        handlers::discount_code::redeem_discount_code,
        handlers::recharge::create_payment_intent,
        handlers::recharge::confirm_recharge,
        handlers::recharge::get_history,
        handlers::admin::sync_orders,
        handlers::admin::sync_discount_codes,
    ),
    components(
        schemas(
            User,
            UserResponse,
            UserStatistics,
            CreateUserRequest,
            LoginRequest,
            UpdateUserRequest,
            AuthResponse,
            SendCodeRequest,
            SendCodeResponse,
            MemberType,
            Order,
            OrderResponse,
            OrderQuery,
            DiscountCode,
            DiscountCodeResponse,
            DiscountCodeQuery,
            RedeemDiscountCodeRequest,
            RedeemDiscountCodeResponse,
            CodeType,
            RechargeRecord,
            RechargeRecordResponse,
            CreatePaymentIntentRequest,
            CreatePaymentIntentResponse,
            ConfirmRechargeRequest,
            ConfirmRechargeResponse,
            RechargeQuery,
            RechargeStatus,
            ApiError,
            PaginatedOrderResponse,
            AuthApiResponse,
            SendCodeApiResponse,
            UserApiResponse,
            OrderListApiResponse,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "Authentication API"),
        (name = "user", description = "User management API"),
        (name = "order", description = "Order management API"),
        (name = "discount", description = "Discount code API"),
        (name = "recharge", description = "Recharge API"),
        (name = "admin", description = "Admin API"),
    ),
    info(
        title = "KKSS Backend API",
        version = "1.0.0",
        description = "KKSS Backend REST API documentation",
        contact(
            name = "API Support",
            email = "driftcell@icloud.com"
        )
    ),
    servers(
        (url = "/api/v1", description = "Local server")
    )
)]
pub struct ApiDoc;

pub fn swagger_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()),
    )
    .route(
        "/swagger-ui",
        web::get().to(|| async {
            actix_web::HttpResponse::Found()
                .append_header(("Location", "/swagger-ui/"))
                .finish()
        }),
    );
}
