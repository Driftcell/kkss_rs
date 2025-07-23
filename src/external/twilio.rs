use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::config::TwilioConfig;
use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct SendSmsRequest {
    #[serde(rename = "To")]
    pub to: String,
    #[serde(rename = "From")]
    pub from: String,
    #[serde(rename = "Body")]
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct SendSmsResponse {
    pub sid: String,
    pub status: String,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Clone)]
pub struct TwilioService {
    client: Client,
    config: TwilioConfig,
}

impl TwilioService {
    pub fn new(config: TwilioConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub async fn send_verification_code(&self, phone: &str, code: &str) -> AppResult<()> {
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            self.config.account_sid
        );

        let body = format!("您的验证码是: {}，有效期5分钟。", code);

        let params = [
            ("To", phone),
            ("From", &self.config.from_phone),
            ("Body", &body),
        ];

        let response = self.client
            .post(&url)
            .basic_auth(&self.config.account_sid, Some(&self.config.auth_token))
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            log::info!("验证码短信发送成功: {}", phone);
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "未知错误".to_string());
            log::error!("验证码短信发送失败: {}, 错误: {}", phone, error_text);
            Err(AppError::ExternalApiError(
                format!("短信发送失败: {}", error_text)
            ))
        }
    }
}

/// 生成6位数字验证码
pub fn generate_verification_code() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.gen_range(100000..=999999))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_verification_code() {
        let code = generate_verification_code();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_digit(10)));
    }
}
