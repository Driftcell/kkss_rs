use std::collections::HashMap;
use chrono::{Duration, Utc};
use sqlx::SqlitePool;
use crate::models::*;
use crate::utils::*;
use crate::external::*;
use crate::error::{AppError, AppResult};

#[derive(Clone)]
pub struct AuthService {
    pool: SqlitePool,
    jwt_service: JwtService,
    twilio_service: TwilioService,
    verification_codes: std::sync::Arc<tokio::sync::RwLock<HashMap<String, (String, chrono::DateTime<Utc>)>>>,
}

impl AuthService {
    pub fn new(
        pool: SqlitePool,
        jwt_service: JwtService,
        twilio_service: TwilioService,
    ) -> Self {
        Self {
            pool,
            jwt_service,
            twilio_service,
            verification_codes: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub async fn send_verification_code(&self, phone: &str) -> AppResult<SendCodeResponse> {
        // 验证手机号格式
        validate_us_phone(phone)?;

        // 检查发送频率限制 (60秒内最多1次)
        {
            let codes = self.verification_codes.read().await;
            if let Some((_, timestamp)) = codes.get(phone) {
                let now = Utc::now();
                if now.signed_duration_since(*timestamp) < Duration::seconds(60) {
                    return Err(AppError::ValidationError(
                        "验证码发送过于频繁，请60秒后再试".to_string()
                    ));
                }
            }
        }

        // 生成验证码
        let code = generate_verification_code();
        let expires_at = Utc::now() + Duration::minutes(5);

        // 发送短信
        self.twilio_service.send_verification_code(phone, &code).await?;

        // 存储验证码
        {
            let mut codes = self.verification_codes.write().await;
            codes.insert(phone.to_string(), (code, expires_at));
        }

        Ok(SendCodeResponse { expires_in: 300 })
    }

    pub async fn register(&self, request: CreateUserRequest) -> AppResult<AuthResponse> {
        // 验证输入参数
        validate_us_phone(&request.phone)?;
        validate_password(&request.password)?;

        // 验证验证码
        self.verify_code(&request.phone, &request.verification_code).await?;

        // 检查手机号是否已注册
        let existing_user = sqlx::query!(
            "SELECT id FROM users WHERE phone = ?",
            request.phone
        )
        .fetch_optional(&self.pool)
        .await?;

        if existing_user.is_some() {
            return Err(AppError::ValidationError("手机号已注册".to_string()));
        }

        // 解析生日
        let birthday = chrono::NaiveDate::parse_from_str(&request.birthday, "%Y-%m-%d")
            .map_err(|_| AppError::ValidationError("生日格式无效".to_string()))?;

        // 生成唯一会员号
        let member_code = generate_unique_member_code(&self.pool).await?;

        // 密码哈希
        let password_hash = hash_password(&request.password)?;

        // 处理推荐人
        let (referrer_id, member_type) = if let Some(referrer_code) = &request.referrer_code {
            let referrer = sqlx::query!(
                "SELECT id FROM users WHERE member_code = ?",
                referrer_code
            )
            .fetch_optional(&self.pool)
            .await?;

            if let Some(referrer) = referrer {
                (Some(referrer.id), MemberType::Fan)
            } else {
                return Err(AppError::ValidationError("推荐人不存在".to_string()));
            }
        } else {
            (None, MemberType::Fan)
        };

        // 生成推荐码
        let referral_code = generate_referral_code();

        // 插入用户
        let member_type_str = member_type.to_string();
        let user_id = sqlx::query!(
            r#"
            INSERT INTO users (
                member_code, phone, username, password_hash, birthday,
                member_type, referrer_id, referral_code
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            member_code,
            request.phone,
            request.username,
            password_hash,
            birthday,
            member_type_str,
            referrer_id,
            referral_code
        )
        .execute(&self.pool)
        .await?
        .last_insert_rowid();

        // 为粉丝发放欢迎优惠码（$0.5）
        if member_type == MemberType::Fan {
            self.create_welcome_discount_code(user_id, 50).await?; // 50美分
        }

        // 生成JWT令牌
        let access_token = self.jwt_service.generate_access_token(user_id, &member_code)?;
        let refresh_token = self.jwt_service.generate_refresh_token(user_id, &member_code)?;

        // 获取完整用户信息
        let user = self.get_user_by_id(user_id).await?;
        let user_response = UserResponse::from(user);

        Ok(AuthResponse {
            user: user_response,
            access_token,
            refresh_token,
            expires_in: self.jwt_service.get_access_token_expires_in(),
        })
    }

    pub async fn login(&self, request: LoginRequest) -> AppResult<AuthResponse> {
        // 验证手机号格式
        validate_us_phone(&request.phone)?;

        // 查找用户
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT
                id as "id!: i64", member_code, phone, username, password_hash, birthday,
                member_type as "member_type: MemberType",
                balance, sweet_cash, referrer_id, referral_code,
                created_at, updated_at
            FROM users
            WHERE phone = ?
            "#,
            request.phone
        )
        .fetch_optional(&self.pool)
        .await?;

        let user = user.ok_or_else(|| {
            AppError::AuthError("用户不存在或密码错误".to_string())
        })?;

        // 验证密码
        let is_valid = verify_password(&request.password, &user.password_hash)?;
        if !is_valid {
            return Err(AppError::AuthError("用户不存在或密码错误".to_string()));
        }

        // 生成JWT令牌
        let access_token = self.jwt_service.generate_access_token(user.id, &user.member_code)?;
        let refresh_token = self.jwt_service.generate_refresh_token(user.id, &user.member_code)?;

        let user_response = UserResponse::from(user);

        Ok(AuthResponse {
            user: user_response,
            access_token,
            refresh_token,
            expires_in: self.jwt_service.get_access_token_expires_in(),
        })
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> AppResult<AuthResponse> {
        // 验证刷新令牌
        let claims = self.jwt_service.verify_refresh_token(refresh_token)?;
        let user_id: i64 = claims.sub.parse()
            .map_err(|_| AppError::AuthError("无效的令牌".to_string()))?;

        // 获取用户信息
        let user = self.get_user_by_id(user_id).await?;

        // 生成新的访问令牌
        let access_token = self.jwt_service.generate_access_token(user.id, &user.member_code)?;

        let user_response = UserResponse::from(user);

        Ok(AuthResponse {
            user: user_response,
            access_token,
            refresh_token: refresh_token.to_string(),
            expires_in: self.jwt_service.get_access_token_expires_in(),
        })
    }

    async fn verify_code(&self, phone: &str, code: &str) -> AppResult<()> {
        let codes = self.verification_codes.read().await;
        
        if let Some((stored_code, expires_at)) = codes.get(phone) {
            let now = Utc::now();
            
            if now > *expires_at {
                return Err(AppError::ValidationError("验证码已过期".to_string()));
            }
            
            if stored_code != code {
                return Err(AppError::ValidationError("验证码错误".to_string()));
            }
            
            Ok(())
        } else {
            Err(AppError::ValidationError("验证码不存在或已过期".to_string()))
        }
    }

    async fn get_user_by_id(&self, user_id: i64) -> AppResult<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT
                id, member_code, phone, username, password_hash, birthday,
                member_type as "member_type: MemberType",
                balance, sweet_cash, referrer_id, referral_code,
                created_at, updated_at
            FROM users
            WHERE id = ?
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        user.ok_or_else(|| AppError::NotFound("用户不存在".to_string()))
    }

    async fn create_welcome_discount_code(&self, user_id: i64, amount: i64) -> AppResult<()> {
        let code = generate_verification_code(); // 重用验证码生成函数
        let expires_at = Utc::now() + Duration::days(365); // 1年有效期

        let code_type_str = CodeType::Welcome.to_string();
        sqlx::query!(
            r#"
            INSERT INTO discount_codes (
                user_id, code, discount_amount, code_type, expires_at
            ) VALUES (?, ?, ?, ?, ?)
            "#,
            user_id,
            code,
            amount,
            code_type_str,
            expires_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
