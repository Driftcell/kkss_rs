use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::config::StripeConfig;
use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePaymentIntentRequest {
    pub amount: i64,
    pub currency: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct PaymentIntent {
    pub id: String,
    pub client_secret: String,
    pub amount: i64,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct WebhookEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: WebhookEventData,
}

#[derive(Debug, Deserialize)]
pub struct WebhookEventData {
    pub object: serde_json::Value,
}

#[derive(Clone)]
pub struct StripeService {
    client: Client,
    config: StripeConfig,
}

impl StripeService {
    pub fn new(config: StripeConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub async fn create_payment_intent(
        &self,
        amount: i64,
        user_id: i64,
    ) -> AppResult<PaymentIntent> {
        let url = "https://api.stripe.com/v1/payment_intents";

        let params = [
            ("amount", amount.to_string()),
            ("currency", "usd".to_string()),
            ("metadata[user_id]", user_id.to_string()),
        ];

        let response = self.client
            .post(url)
            .bearer_auth(&self.config.secret_key)
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let payment_intent: PaymentIntent = response.json().await?;
            Ok(payment_intent)
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "未知错误".to_string());
            Err(AppError::ExternalApiError(
                format!("创建支付意图失败: {}", error_text)
            ))
        }
    }

    pub async fn retrieve_payment_intent(&self, payment_intent_id: &str) -> AppResult<PaymentIntent> {
        let url = format!("https://api.stripe.com/v1/payment_intents/{}", payment_intent_id);

        let response = self.client
            .get(&url)
            .bearer_auth(&self.config.secret_key)
            .send()
            .await?;

        if response.status().is_success() {
            let payment_intent: PaymentIntent = response.json().await?;
            Ok(payment_intent)
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "未知错误".to_string());
            Err(AppError::ExternalApiError(
                format!("获取支付意图失败: {}", error_text)
            ))
        }
    }

    pub fn verify_webhook_signature(
        &self,
        _payload: &str,
        signature: &str,
        _timestamp: i64,
    ) -> AppResult<()> {
        // 简化的webhook签名验证
        // 实际生产环境中应该使用更严格的验证逻辑
        if signature.is_empty() {
            return Err(AppError::AuthError("无效的webhook签名".to_string()));
        }
        
        // 这里应该实现真正的Stripe webhook签名验证
        // 目前简化处理
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stripe_service_creation() {
        let config = StripeConfig {
            secret_key: "sk_test_123".to_string(),
            webhook_secret: "whsec_123".to_string(),
        };
        let service = StripeService::new(config);
        assert!(!service.config.secret_key.is_empty());
    }
}
