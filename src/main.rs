use actix_web::{App, HttpServer, middleware::Logger, web};
use env_logger::Env;
use std::sync::Arc;
use tokio::sync::Mutex;

use kkss_backend::{
    config::Config,
    database::{create_pool, run_migrations},
    external::{SevenCloudAPI, StripeService, TwilioService},
    handlers,
    middlewares::{AuthMiddleware, create_cors},
    services::*,
    swagger::swagger_config,
    utils::JwtService,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    // 加载配置
    let config = Config::from_toml().expect("无法加载配置文件");

    // 创建数据库连接池
    let pool = create_pool(&config.database)
        .await
        .expect("无法创建数据库连接池");

    // 运行数据库迁移
    run_migrations(&pool).await.expect("无法运行数据库迁移");

    // 创建JWT服务
    let jwt_service = JwtService::new(
        &config.jwt.secret,
        config.jwt.access_token_expires_in,
        config.jwt.refresh_token_expires_in,
    );

    // 创建外部服务
    let twilio_service = TwilioService::new(config.twilio.clone());
    let stripe_service = StripeService::new(config.stripe.clone());

    let mut sevencloud_api = SevenCloudAPI::new(config.sevencloud.clone());
    if let Err(e) = sevencloud_api.login().await {
        log::error!("七云API登录失败: {:?}", e);
    }
    let sevencloud_api = Arc::new(Mutex::new(sevencloud_api));

    // 创建服务
    let auth_service = AuthService::new(
        pool.clone(),
        jwt_service.clone(),
        twilio_service,
        sevencloud_api.clone(),
    );

    let user_service = UserService::new(pool.clone());
    let order_service = OrderService::new(pool.clone());
    let discount_code_service = DiscountCodeService::new(pool.clone(), sevencloud_api.clone());
    let recharge_service = RechargeService::new(pool.clone(), stripe_service);
    let sync_service = SyncService::new(pool.clone(), sevencloud_api.clone());

    // 启动HTTP服务器
    log::info!("服务器启动在 {}:{}", config.server.host, config.server.port);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(create_cors())
            .wrap(AuthMiddleware::new(jwt_service.clone()))
            .app_data(web::Data::new(auth_service.clone()))
            .app_data(web::Data::new(user_service.clone()))
            .app_data(web::Data::new(order_service.clone()))
            .app_data(web::Data::new(discount_code_service.clone()))
            .app_data(web::Data::new(recharge_service.clone()))
            .app_data(web::Data::new(sync_service.clone()))
            .configure(swagger_config)
            .service(
                web::scope("/api/v1")
                    .configure(handlers::auth_config)
                    .configure(handlers::user_config)
                    .configure(handlers::order_config)
                    .configure(handlers::discount_code_config)
                    .configure(handlers::recharge_config)
                    .configure(handlers::admin_config),
            )
    })
    .bind((config.server.host.as_str(), config.server.port))?
    .run()
    .await
}
