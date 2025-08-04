use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub twilio: TwilioConfig,
    pub stripe: StripeConfig,
    pub sevencloud: SevenCloudConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub access_token_expires_in: i64,  // seconds
    pub refresh_token_expires_in: i64, // seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwilioConfig {
    pub account_sid: String,
    pub auth_token: String,
    pub from_phone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeConfig {
    pub secret_key: String,
    pub webhook_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SevenCloudConfig {
    pub username: String,
    pub password: String,
    pub base_url: String,
}

impl Config {
    pub fn from_toml() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());

        let config_str = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("无法读取配置文件 {}: {}", config_path, e))?;

        let mut config: Config =
            toml::from_str(&config_str).map_err(|e| format!("解析配置文件失败: {}", e))?;

        // 环境变量覆盖配置
        if let Ok(database_url) = env::var("DATABASE_URL") {
            config.database.url = database_url;
        }
        if let Ok(jwt_secret) = env::var("JWT_SECRET") {
            config.jwt.secret = jwt_secret;
        }
        if let Ok(twilio_account_sid) = env::var("TWILIO_ACCOUNT_SID") {
            config.twilio.account_sid = twilio_account_sid;
        }
        if let Ok(twilio_auth_token) = env::var("TWILIO_AUTH_TOKEN") {
            config.twilio.auth_token = twilio_auth_token;
        }
        if let Ok(stripe_secret_key) = env::var("STRIPE_SECRET_KEY") {
            config.stripe.secret_key = stripe_secret_key;
        }
        if let Ok(stripe_webhook_secret) = env::var("STRIPE_WEBHOOK_SECRET") {
            config.stripe.webhook_secret = stripe_webhook_secret;
        }
        if let Ok(sevencloud_username) = env::var("SEVENCLOUD_USERNAME") {
            config.sevencloud.username = sevencloud_username;
        }
        if let Ok(sevencloud_password) = env::var("SEVENCLOUD_PASSWORD") {
            config.sevencloud.password = sevencloud_password;
        }

        Ok(config)
    }
}
