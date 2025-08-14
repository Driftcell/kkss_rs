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
    pub verify_service_sid: String,
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
        use std::io::ErrorKind;

        // 尝试读取配置文件，如果不存在则完全依赖环境变量
        let config_result = std::fs::read_to_string(&config_path);

        let mut config: Config = match config_result {
            Ok(config_str) => {
                // 有配置文件：先解析再用环境变量覆盖
                toml::from_str(&config_str).map_err(|e| format!("解析配置文件失败: {e}"))?
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {
                // 无配置文件：使用环境变量与默认值构建
                fn get_env(name: &str) -> Option<String> {
                    env::var(name).ok()
                }
                fn get_env_parse<T: std::str::FromStr>(name: &str, default: T) -> T {
                    env::var(name)
                        .ok()
                        .and_then(|v| v.parse::<T>().ok())
                        .unwrap_or(default)
                }

                // 数据库 URL 在无配置文件时必须提供
                let database_url = get_env("DATABASE_URL")
                    .ok_or("缺少 DATABASE_URL 环境变量，且未找到配置文件 config.toml")?;

                Config {
                    server: ServerConfig {
                        host: get_env("SERVER_HOST").unwrap_or_else(|| "0.0.0.0".to_string()),
                        port: get_env_parse("SERVER_PORT", 8080u16),
                    },
                    database: DatabaseConfig {
                        url: database_url,
                        max_connections: get_env_parse("DB_MAX_CONNECTIONS", 10u32),
                    },
                    jwt: JwtConfig {
                        secret: get_env("JWT_SECRET")
                            .unwrap_or_else(|| "change-me-in-production".to_string()),
                        access_token_expires_in: get_env_parse("JWT_ACCESS_EXPIRES_IN", 7200i64),
                        refresh_token_expires_in: get_env_parse(
                            "JWT_REFRESH_EXPIRES_IN",
                            2_592_000i64,
                        ),
                    },
                    twilio: TwilioConfig {
                        account_sid: get_env("TWILIO_ACCOUNT_SID").unwrap_or_default(),
                        auth_token: get_env("TWILIO_AUTH_TOKEN").unwrap_or_default(),
                        from_phone: get_env("TWILIO_FROM_PHONE").unwrap_or_default(),
                        verify_service_sid: get_env("TWILIO_VERIFY_SERVICE_SID")
                            .unwrap_or_default(),
                    },
                    stripe: StripeConfig {
                        secret_key: get_env("STRIPE_SECRET_KEY").unwrap_or_default(),
                        webhook_secret: get_env("STRIPE_WEBHOOK_SECRET").unwrap_or_default(),
                    },
                    sevencloud: SevenCloudConfig {
                        username: get_env("SEVENCLOUD_USERNAME").unwrap_or_default(),
                        password: get_env("SEVENCLOUD_PASSWORD").unwrap_or_default(),
                        base_url: get_env("SEVENCLOUD_BASE_URL")
                            .unwrap_or_else(|| "https://sz.sunzee.com.cn".to_string()),
                    },
                }
            }
            Err(e) => {
                return Err(format!("无法读取配置文件 {config_path}: {e}").into());
            }
        };

        // 环境变量覆盖（即便文件存在时也覆盖）
        if let Ok(v) = env::var("SERVER_HOST") {
            config.server.host = v;
        }
        if let Ok(v) = env::var("SERVER_PORT") {
            if let Ok(p) = v.parse() {
                config.server.port = p;
            }
        }
        if let Ok(v) = env::var("DATABASE_URL") {
            config.database.url = v;
        }
        if let Ok(v) = env::var("DB_MAX_CONNECTIONS") {
            if let Ok(mc) = v.parse() {
                config.database.max_connections = mc;
            }
        }
        if let Ok(v) = env::var("JWT_SECRET") {
            config.jwt.secret = v;
        }
        if let Ok(v) = env::var("JWT_ACCESS_EXPIRES_IN") {
            if let Ok(n) = v.parse() {
                config.jwt.access_token_expires_in = n;
            }
        }
        if let Ok(v) = env::var("JWT_REFRESH_EXPIRES_IN") {
            if let Ok(n) = v.parse() {
                config.jwt.refresh_token_expires_in = n;
            }
        }
        if let Ok(v) = env::var("TWILIO_ACCOUNT_SID") {
            config.twilio.account_sid = v;
        }
        if let Ok(v) = env::var("TWILIO_AUTH_TOKEN") {
            config.twilio.auth_token = v;
        }
        if let Ok(v) = env::var("TWILIO_FROM_PHONE") {
            config.twilio.from_phone = v;
        }
        if let Ok(v) = env::var("TWILIO_VERIFY_SERVICE_SID") {
            config.twilio.verify_service_sid = v;
        }
        if let Ok(v) = env::var("STRIPE_SECRET_KEY") {
            config.stripe.secret_key = v;
        }
        if let Ok(v) = env::var("STRIPE_WEBHOOK_SECRET") {
            config.stripe.webhook_secret = v;
        }
        if let Ok(v) = env::var("SEVENCLOUD_USERNAME") {
            config.sevencloud.username = v;
        }
        if let Ok(v) = env::var("SEVENCLOUD_PASSWORD") {
            config.sevencloud.password = v;
        }
        if let Ok(v) = env::var("SEVENCLOUD_BASE_URL") {
            config.sevencloud.base_url = v;
        }

        Ok(config)
    }
}
