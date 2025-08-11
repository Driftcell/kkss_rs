use crate::config::TwilioConfig;
use crate::error::{AppError, AppResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};

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

        let body = format!("Your verification code is: {}，valid for 5 minutes.", code);

        let params = [
            ("To", phone),
            ("From", &self.config.from_phone),
            ("Body", &body),
        ];

        let response = self
            .client
            .post(&url)
            .basic_auth(&self.config.account_sid, Some(&self.config.auth_token))
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            log::info!("Verification code SMS sent successfully: {}", phone);
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            log::error!("Verification code SMS failed to send: {}, Error: {}", phone, error_text);
            Err(AppError::ExternalApiError(format!(
                "SMS sending failed: {}",
                error_text
            )))
        }
    }
}

// 导出generate_six_digit_code以保持向后兼容
pub use crate::utils::generate_six_digit_code as generate_verification_code;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_verification_code() {
        let code = generate_verification_code();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));

        // 确保代码在有效范围内
        let code_num: u32 = code.parse().unwrap();
        assert!(code_num >= 100000 && code_num <= 999999);
    }
}
