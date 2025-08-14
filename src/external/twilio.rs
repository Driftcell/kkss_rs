use crate::config::TwilioConfig;
use crate::error::{AppError, AppResult};
use reqwest::Client;
use serde::Deserialize;

#[derive(Clone)]
pub struct TwilioService {
    client: Client,
    config: TwilioConfig,
}

#[derive(Debug, Deserialize)]
struct VerifyStartResponse {
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VerifyCheckResponse {
    status: Option<String>,
    valid: Option<bool>,
}

impl TwilioService {
    pub fn new(config: TwilioConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// Start a Verify verification via SMS.
    /// Docs: POST https://verify.twilio.com/v2/Services/{ServiceSid}/Verifications
    pub async fn start_verification_sms(&self, phone: &str) -> AppResult<()> {
        if self.config.verify_service_sid.is_empty() {
            return Err(AppError::InternalError(
                "Missing TWILIO_VERIFY_SERVICE_SID config".to_string(),
            ));
        }

        let url = format!(
            "https://verify.twilio.com/v2/Services/{}/Verifications",
            self.config.verify_service_sid
        );

        // Twilio Verify expects x-www-form-urlencoded with keys To/Channel
        let params = [("To", phone), ("Channel", "sms")];

        let resp = self
            .client
            .post(url)
            .basic_auth(&self.config.account_sid, Some(&self.config.auth_token))
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::ExternalApiError(format!("Twilio request error: {e}")))?;

        if !resp.status().is_success() {
            let txt = resp
                .text()
                .await
                .unwrap_or_else(|_| "Unknown Twilio error".to_string());
            return Err(AppError::ExternalApiError(format!(
                "Twilio Verify start failed: {txt}"
            )));
        }

        // Optionally inspect status (pending)
        let body: VerifyStartResponse = resp
            .json()
            .await
            .unwrap_or(VerifyStartResponse { status: None });
        log::info!("Twilio Verify started for {}: {:?}", phone, body.status);
        Ok(())
    }

    /// Check a verification code.
    /// Docs: POST https://verify.twilio.com/v2/Services/{ServiceSid}/VerificationCheck
    pub async fn check_verification_code(&self, phone: &str, code: &str) -> AppResult<bool> {
        if self.config.verify_service_sid.is_empty() {
            return Err(AppError::InternalError(
                "Missing TWILIO_VERIFY_SERVICE_SID config".to_string(),
            ));
        }

        let url = format!(
            "https://verify.twilio.com/v2/Services/{}/VerificationCheck",
            self.config.verify_service_sid
        );

        let params = [("To", phone), ("Code", code)];

        let resp = self
            .client
            .post(url)
            .basic_auth(&self.config.account_sid, Some(&self.config.auth_token))
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::ExternalApiError(format!("Twilio request error: {e}")))?;

        if !resp.status().is_success() {
            let txt = resp
                .text()
                .await
                .unwrap_or_else(|_| "Unknown Twilio error".to_string());
            return Err(AppError::ExternalApiError(format!(
                "Twilio Verify check failed: {txt}"
            )));
        }

        let body: VerifyCheckResponse = resp.json().await.map_err(|e| {
            AppError::ExternalApiError(format!("Failed to parse Twilio response: {e}"))
        })?;
        let approved = body.status.as_deref() == Some("approved") && body.valid.unwrap_or(false);
        Ok(approved)
    }
}
