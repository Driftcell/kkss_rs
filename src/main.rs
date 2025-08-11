use actix_web::{App, HttpServer, middleware::Logger, web};
use env_logger::{Env, Target};
use std::io::Write; // for env_logger custom formatter
use chrono::Local;  // timestamp in log lines
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
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let ts = Local::now().format("%Y-%m-%dT%H:%M:%S%.3f%:z");
            let level = record.level().as_str().to_ascii_lowercase();
            let msg_json = serde_json::to_string(&format!("{}", record.args()))
                .unwrap_or_else(|_| "\"<invalid utf8>\"".to_string());
            writeln!(
                buf,
                "{{\"timestamp\":\"{}\",\"level\":\"{}\",\"message\":{},\"target\":\"{}\"}}",
                ts,
                level,
                msg_json,
                record.target(),
            )
        })
        .target(Target::Stdout)
        .init();

    // 加载配置
    let config = Config::from_toml().expect("Failed to load configuration file");

    // 创建数据库连接池
    let pool = create_pool(&config.database)
        .await
        .expect("Failed to create database connection pool");

    // 运行数据库迁移
    run_migrations(&pool)
        .await
        .expect("Failed to run database migrations");

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
        log::error!("SevenCloud API login failed: {:?}", e);
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
    let recharge_service = RechargeService::new(pool.clone(), stripe_service.clone());
    let sync_service = SyncService::new(pool.clone(), sevencloud_api.clone());

    // 启动后台定时同步任务 (每分钟同步最近一周订单与优惠码)
    {
        let sync_service_clone = sync_service.clone();
        tokio::spawn(async move {
            use chrono::Duration;
            use chrono::Utc;
            loop {
                let now = Utc::now();
                let start = now - Duration::days(7);
                let start_date = start.format("%Y-%m-%d %H:%M:%S").to_string();
                let end_date = format!("{} 23:59:59", now.format("%Y-%m-%d"));

                log::info!(
                    "Start syncing orders and discount codes: {} ~ {}",
                    start_date,
                    end_date
                );
                // 同步订单
                if let Err(e) = sync_service_clone.sync_orders(&start_date, &end_date).await {
                    log::error!("Failed to sync orders: {:?}", e);
                }
                // 同步优惠码
                if let Err(e) = sync_service_clone.sync_discount_codes().await {
                    log::error!("Failed to sync discount codes: {:?}", e);
                }
                // 间隔 60 秒
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        });
    }

    // 启动HTTP服务器
    log::info!(
        "Starting HTTP server at {}:{}",
        config.server.host,
        config.server.port
    );

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
            .app_data(web::Data::new(stripe_service.clone()))
            .app_data(web::Data::new(sync_service.clone()))
            .configure(swagger_config)
            .configure(handlers::webhook_config)
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
