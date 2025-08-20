use crate::config::TurnstileConfig;
use crate::error::{AppError, AppResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};

const VERIFY_ENDPOINT: &str = "https://challenges.cloudflare.com/turnstile/v0/siteverify";

#[derive(Clone)]
pub struct TurnstileService {
    http: Client,
    cfg: TurnstileConfig,
}

impl TurnstileService {
    pub fn new(cfg: TurnstileConfig) -> Self {
        let http = Client::builder()
            .user_agent("kkss-backend/turnstile")
            .build()
            .expect("reqwest client");
        Self { http, cfg }
    }

    pub fn is_enabled(&self) -> bool {
        !self.cfg.secret_key.is_empty()
    }

    /// 校验从前端提交的 Turnstile token。
    /// 如果 expected_* 在配置中设置，将进行额外校验。
    pub async fn verify_token(
        &self,
        token: &str,
        remote_ip: Option<&str>,
        idempotency_key: Option<&str>,
    ) -> AppResult<()> {
        if token.is_empty() {
            return Err(AppError::ValidationError("Missing Turnstile token".into()));
        }
        if token.len() > 2048 {
            return Err(AppError::ValidationError("Invalid Turnstile token".into()));
        }

        let mut req_body = serde_json::json!({
            "secret": self.cfg.secret_key,
            "response": token,
        });

        if let Some(ip) = remote_ip {
            req_body["remoteip"] = serde_json::json!(ip);
        }
        if let Some(key) = idempotency_key {
            req_body["idempotency_key"] = serde_json::json!(key);
        }

        let resp = self
            .http
            .post(VERIFY_ENDPOINT)
            .json(&req_body)
            .send()
            .await?;

        let status = resp.status();
        let body: VerifyResponse = resp.json().await?;

        if !status.is_success() || !body.success {
            let errs = body.error_codes.unwrap_or_default().join(",");
            return Err(AppError::ExternalApiError(format!(
                "Turnstile verification failed: HTTP {}: {}",
                status.as_u16(),
                errs
            )));
        }

        if let Some(expected) = &self.cfg.expected_hostname
            && let Some(host) = body.hostname.as_ref()
                && host != expected {
                    return Err(AppError::ValidationError(
                        "Turnstile hostname mismatch".into(),
                    ));
                }
        if let Some(expected) = &self.cfg.expected_action
            && let Some(action) = body.action.as_ref()
                && action != expected {
                    return Err(AppError::ValidationError(
                        "Turnstile action mismatch".into(),
                    ));
                }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct VerifyResponse {
    success: bool,
    #[serde(default)]
    challenge_ts: Option<String>,
    #[serde(default)]
    hostname: Option<String>,
    #[serde(rename = "error-codes")]
    #[serde(default)]
    error_codes: Option<Vec<String>>,
    #[serde(default)]
    action: Option<String>,
    #[serde(default)]
    cdata: Option<String>,
}
