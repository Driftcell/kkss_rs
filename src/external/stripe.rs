use crate::config::StripeConfig;
use crate::error::{AppError, AppResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePaymentIntentRequest {
    pub amount: i64, // 以最小货币单位计算 (如美分)
    pub currency: String,
    pub customer_id: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub automatic_payment_methods: Option<AutomaticPaymentMethods>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutomaticPaymentMethods {
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct PaymentIntent {
    pub id: String,
    pub client_secret: String,
    pub amount: i64,
    pub currency: String,
    pub status: String,
    pub created: i64,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
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

/// Stripe服务，用于处理支付意图和webhook验证
///
/// 这个服务专为任意金额充值设计，而不是预制商品类型的支付。
/// 支持多种货币，自动启用多种支付方式，并包含完整的金额验证。
///
/// # 示例
///
/// ```rust
/// use kkss_backend::external::stripe::{StripeService, StripeConfig};
///
/// let config = StripeConfig {
///     secret_key: "sk_test_...".to_string(),
///     webhook_secret: "whsec_...".to_string(),
/// };
/// let stripe_service = StripeService::new(config);
///
/// // 创建$10.00的充值支付意图
/// let amount_cents = StripeService::dollars_to_cents(10.00);
/// let payment_intent = stripe_service.create_payment_intent(
///     amount_cents,
///     123, // user_id
///     Some("usd".to_string()),
///     Some("用户充值".to_string())
/// ).await?;
/// ```
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

    /// 创建用于任意金额充值的支付意图
    ///
    /// # 参数
    ///
    /// * `amount` - 充值金额，以最小货币单位计算（如美分）
    /// * `user_id` - 用户ID，会存储在metadata中
    /// * `currency` - 货币代码（如"usd", "eur"），默认为"usd"
    /// * `description` - 支付描述，如果为None会自动生成
    ///
    /// # 返回
    ///
    /// 返回包含client_secret的PaymentIntent，客户端可用此完成支付
    ///
    /// # 错误
    ///
    /// * 如果金额小于最小值（$0.50）会返回ValidationError
    /// * 如果Stripe API调用失败会返回ExternalApiError
    pub async fn create_payment_intent(
        &self,
        amount: i64,
        user_id: i64,
        currency: Option<String>,
        description: Option<String>,
    ) -> AppResult<PaymentIntent> {
        let url = "https://api.stripe.com/v1/payment_intents";

        // 验证最小金额 (50美分 = $0.50)
        if amount < 50 {
            return Err(AppError::ValidationError(
                "Minimum amount is $0.50".to_string(),
            ));
        }

        // 构建请求参数
        let mut params = vec![
            ("amount", amount.to_string()),
            ("currency", currency.unwrap_or_else(|| "usd".to_string())),
            ("automatic_payment_methods[enabled]", "true".to_string()),
        ];

        // 添加用户ID到元数据
        params.push(("metadata[user_id]", user_id.to_string()));
        params.push(("metadata[type]", "recharge".to_string()));

        // 如果提供了描述，添加到请求中
        if let Some(desc) = description {
            params.push(("description", desc));
        } else {
            params.push((
                "description",
                format!("Recharge ${:.2} to account", amount as f64 / 100.0),
            ));
        }

        let response = self
            .client
            .post(url)
            .bearer_auth(&self.config.secret_key)
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let payment_intent: PaymentIntent = response.json().await?;
            Ok(payment_intent)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(AppError::ExternalApiError(format!(
                "Failed to create payment intent: {}",
                error_text
            )))
        }
    }

    /// 检索已存在的支付意图
    ///
    /// # 参数
    ///
    /// * `payment_intent_id` - Stripe支付意图ID
    ///
    /// # 返回
    ///
    /// 返回PaymentIntent对象，包含当前状态和详细信息
    pub async fn retrieve_payment_intent(
        &self,
        payment_intent_id: &str,
    ) -> AppResult<PaymentIntent> {
        let url = format!(
            "https://api.stripe.com/v1/payment_intents/{}",
            payment_intent_id
        );

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.config.secret_key)
            .send()
            .await?;

        if response.status().is_success() {
            let payment_intent: PaymentIntent = response.json().await?;
            Ok(payment_intent)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(AppError::ExternalApiError(format!(
                "Failed to retrieve payment intent: {}",
                error_text
            )))
        }
    }

    pub fn verify_webhook_signature(
        &self,
        _payload: &str,
        signature: &str,
        _timestamp: i64,
    ) -> AppResult<()> {
        // 验证webhook签名头是否存在
        if signature.is_empty() {
            return Err(AppError::AuthError("Invalid webhook signature".to_string()));
        }

        // 解析签名头格式: t=timestamp,v1=signature
        let mut timestamp = None;
        let mut v1_signature = None;

        for part in signature.split(',') {
            if let Some((key, value)) = part.split_once('=') {
                match key {
                    "t" => timestamp = Some(value),
                    "v1" => v1_signature = Some(value),
                    _ => {}
                }
            }
        }

        if timestamp.is_none() || v1_signature.is_none() {
            return Err(AppError::AuthError("Invalid webhook signature format".to_string()));
        }

        // 在生产环境中，这里应该使用HMAC-SHA256验证签名
        // 使用webhook_secret和payload生成期望的签名，然后与v1_signature比较
        //
        // 示例实现 (需要添加hmac和sha2依赖):
        // use hmac::{Hmac, Mac};
        // use sha2::Sha256;
        //
        // let signed_payload = format!("{}.{}", timestamp.unwrap(), payload);
        // let mut mac = Hmac::<Sha256>::new_from_slice(self.config.webhook_secret.as_bytes())
        //     .map_err(|_| AppError::AuthError("无效的webhook密钥".to_string()))?;
        // mac.update(signed_payload.as_bytes());
        // let expected_signature = hex::encode(mac.finalize().into_bytes());
        //
        // if expected_signature != v1_signature.unwrap() {
        //     return Err(AppError::AuthError("webhook签名验证失败".to_string()));
        // }

        Ok(())
    }

    /// 将美元金额转换为美分
    ///
    /// # 示例
    ///
    /// ```
    /// use kkss_backend::external::stripe::StripeService;
    ///
    /// assert_eq!(StripeService::dollars_to_cents(10.99), 1099);
    /// assert_eq!(StripeService::dollars_to_cents(0.50), 50);
    /// ```
    pub fn dollars_to_cents(dollars: f64) -> i64 {
        (dollars * 100.0).round() as i64
    }

    /// 将美分转换为美元金额
    ///
    /// # 示例
    ///
    /// ```
    /// use kkss_backend::external::stripe::StripeService;
    ///
    /// assert_eq!(StripeService::cents_to_dollars(1099), 10.99);
    /// assert_eq!(StripeService::cents_to_dollars(50), 0.50);
    /// ```
    pub fn cents_to_dollars(cents: i64) -> f64 {
        cents as f64 / 100.0
    }

    /// 验证金额是否符合Stripe的要求
    ///
    /// 根据不同货币检查最小和最大金额限制。
    ///
    /// # 参数
    ///
    /// * `amount` - 金额，以最小货币单位计算
    /// * `currency` - 货币代码（如"usd", "eur", "jpy"）
    ///
    /// # 错误
    ///
    /// * 如果金额小于最小值会返回ValidationError
    /// * 如果金额超过最大值会返回ValidationError
    pub fn validate_amount(amount: i64, currency: &str) -> AppResult<()> {
        let min_amount = match currency.to_lowercase().as_str() {
            "usd" | "eur" | "cad" | "aud" | "gbp" => 50, // $0.50
            "jpy" => 50,                                 // ¥50 (日元没有小数)
            _ => 50,                                     // 默认最小值
        };

        if amount < min_amount {
            return Err(AppError::ValidationError(format!(
                "Minimum recharge amount is {} {}",
                if currency == "jpy" {
                    format!("{}", min_amount)
                } else {
                    format!("{:.2}", min_amount as f64 / 100.0)
                },
                currency.to_uppercase()
            )));
        }

        // Stripe支持的最大金额是99999999 (约$999,999.99)
        if amount > 99999999 {
            return Err(AppError::ValidationError(
                "Maximum recharge amount is $999,999.99".to_string(),
            ));
        }

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

    #[test]
    fn test_dollars_to_cents_conversion() {
        assert_eq!(StripeService::dollars_to_cents(1.00), 100);
        assert_eq!(StripeService::dollars_to_cents(0.50), 50);
        assert_eq!(StripeService::dollars_to_cents(10.99), 1099);
        assert_eq!(StripeService::dollars_to_cents(0.01), 1);
    }

    #[test]
    fn test_cents_to_dollars_conversion() {
        assert_eq!(StripeService::cents_to_dollars(100), 1.00);
        assert_eq!(StripeService::cents_to_dollars(50), 0.50);
        assert_eq!(StripeService::cents_to_dollars(1099), 10.99);
        assert_eq!(StripeService::cents_to_dollars(1), 0.01);
    }

    #[test]
    fn test_amount_validation() {
        // 测试有效金额
        assert!(StripeService::validate_amount(100, "usd").is_ok()); // $1.00
        assert!(StripeService::validate_amount(50, "usd").is_ok()); // $0.50 (最小值)

        // 测试无效金额 (小于最小值)
        assert!(StripeService::validate_amount(49, "usd").is_err());
        assert!(StripeService::validate_amount(0, "usd").is_err());

        // 测试超大金额
        assert!(StripeService::validate_amount(100000000, "usd").is_err());

        // 测试日元 (无小数位)
        assert!(StripeService::validate_amount(50, "jpy").is_ok());
        assert!(StripeService::validate_amount(49, "jpy").is_err());
    }

    #[test]
    fn test_webhook_signature_validation() {
        let config = StripeConfig {
            secret_key: "sk_test_123".to_string(),
            webhook_secret: "whsec_123".to_string(),
        };
        let service = StripeService::new(config);

        // 测试空签名
        assert!(service.verify_webhook_signature("payload", "", 0).is_err());

        // 测试有效格式的签名
        let valid_signature = "t=1634025600,v1=abcdef123456";
        assert!(
            service
                .verify_webhook_signature("payload", valid_signature, 0)
                .is_ok()
        );

        // 测试无效格式的签名
        let invalid_signature = "invalid_format";
        assert!(
            service
                .verify_webhook_signature("payload", invalid_signature, 0)
                .is_err()
        );
    }
}
