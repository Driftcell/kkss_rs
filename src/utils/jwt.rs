use crate::error::{AppError, AppResult};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub member_code: String,
    pub exp: i64,
    pub iat: i64,
    pub token_type: String, // "access" or "refresh"
}

#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_token_expires_in: i64,
    refresh_token_expires_in: i64,
}

impl JwtService {
    pub fn new(secret: &str, access_expires_in: i64, refresh_expires_in: i64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            access_token_expires_in: access_expires_in,
            refresh_token_expires_in: refresh_expires_in,
        }
    }

    pub fn generate_access_token(&self, user_id: i64, member_code: &str) -> AppResult<String> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.access_token_expires_in);

        let claims = Claims {
            sub: user_id.to_string(),
            member_code: member_code.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            token_type: "access".to_string(),
        };

        encode(&Header::default(), &claims, &self.encoding_key).map_err(|e| AppError::JwtError(e))
    }

    pub fn generate_refresh_token(&self, user_id: i64, member_code: &str) -> AppResult<String> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.refresh_token_expires_in);

        let claims = Claims {
            sub: user_id.to_string(),
            member_code: member_code.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            token_type: "refresh".to_string(),
        };

        encode(&Header::default(), &claims, &self.encoding_key).map_err(|e| AppError::JwtError(e))
    }

    pub fn verify_token(&self, token: &str) -> AppResult<Claims> {
        let validation = Validation::new(Algorithm::HS256);
        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| AppError::JwtError(e))
    }

    pub fn verify_access_token(&self, token: &str) -> AppResult<Claims> {
        let claims = self.verify_token(token)?;

        if claims.token_type != "access" {
            return Err(AppError::AuthError("Invalid access token type".to_string()));
        }

        Ok(claims)
    }

    pub fn verify_refresh_token(&self, token: &str) -> AppResult<Claims> {
        let claims = self.verify_token(token)?;

        if claims.token_type != "refresh" {
            return Err(AppError::AuthError("Invalid refresh token type".to_string()));
        }

        Ok(claims)
    }

    pub fn get_access_token_expires_in(&self) -> i64 {
        self.access_token_expires_in
    }
}
