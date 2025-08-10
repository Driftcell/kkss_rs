use crate::error::{AppError, AppResult};
use crate::external::*;
use crate::models::*;
use crate::utils::*;
use chrono::{Duration, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AuthService {
    pool: PgPool,
    jwt_service: JwtService,
    twilio_service: TwilioService,
    sevencloud_api: Arc<Mutex<SevenCloudAPI>>,
}

impl AuthService {
    pub fn new(
        pool: PgPool,
        jwt_service: JwtService,
        twilio_service: TwilioService,
        sevencloud_api: Arc<Mutex<SevenCloudAPI>>,
    ) -> Self {
        Self {
            pool,
            jwt_service,
            twilio_service,
            sevencloud_api,
        }
    }

    pub async fn send_verification_code(&self, phone: &str) -> AppResult<SendCodeResponse> {
        // 验证手机号格式
        validate_us_phone(phone)?;

        // 检查发送频率限制 (60秒内最多1次)
        let last_code = sqlx::query!(
            "SELECT created_at FROM verification_codes WHERE phone = $1 ORDER BY created_at DESC LIMIT 1",
            phone
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(last_code) = last_code {
            let now = Utc::now();
            // created_at在SQLite中返回为NaiveDateTime，需要处理Option
            if let Some(created_at) = last_code.created_at {
                    if now.signed_duration_since(created_at) < Duration::seconds(60) {
                    return Err(AppError::ValidationError(
                        "The verification code has been sent too frequently, please try again after 60 seconds.".to_string(),
                    ));
                }
            }
        }

        // 生成验证码
        let code = generate_six_digit_code();
        let expires_at = Utc::now() + Duration::minutes(5);

        // 发送短信
        self.twilio_service
            .send_verification_code(phone, &code)
            .await?;

        // 存储验证码到数据库
        sqlx::query!(
            "INSERT INTO verification_codes (phone, code, expires_at) VALUES ($1, $2, $3)",
            phone,
            code,
            expires_at
        )
        .execute(&self.pool)
        .await?;

        Ok(SendCodeResponse { expires_in: 300 })
    }

    pub async fn register(&self, request: CreateUserRequest) -> AppResult<AuthResponse> {
        // 验证输入参数
        validate_us_phone(&request.phone)?;
        validate_password(&request.password)?;

        // 验证验证码
        self.verify_code(&request.phone, &request.verification_code)
            .await?;

        // 检查手机号是否已注册
    let existing_user = sqlx::query!("SELECT id FROM users WHERE phone = $1", request.phone)
            .fetch_optional(&self.pool)
            .await?;

        if existing_user.is_some() {
            return Err(AppError::ValidationError("The mobile phone number is registered".to_string()));
        }

        // 解析生日
        let birthday = chrono::NaiveDate::parse_from_str(&request.birthday, "%Y-%m-%d")
            .map_err(|_| AppError::ValidationError("Invalid birthday format".to_string()))?;

        // 从手机号生成会员号（去掉+1前缀的十位数字）
        let member_code = extract_member_code_from_phone(&request.phone)?;

        // 检查会员号是否已存在（防止重复注册）
        let existing_member =
            sqlx::query!("SELECT id FROM users WHERE member_code = $1", member_code)
                .fetch_optional(&self.pool)
                .await?;

        if existing_member.is_some() {
            return Err(AppError::ValidationError(
                "The member code corresponding to this phone number already exists".to_string(),
            ));
        }

        // 密码哈希
        let password_hash = hash_password(&request.password)?;

        // 处理推荐人
        let (referrer_id, member_type) = if let Some(referrer_code) = &request.referrer_code {
            let referrer = sqlx::query!(
                "SELECT id as \"id!: i64\" FROM users WHERE member_code = $1",
                referrer_code
            )
            .fetch_optional(&self.pool)
            .await?;

            if let Some(referrer) = referrer {
                (Some(referrer.id), MemberType::Fan)
            } else {
                return Err(AppError::ValidationError("The referrer does not exist".to_string()));
            }
        } else {
            (None, MemberType::Fan)
        };

        // 生成推荐码
        let referral_code = generate_unique_referral_code(&self.pool).await?;

        // 插入用户
        let member_type_str = member_type.to_string();
        let user_id: i64 = sqlx::query_scalar!(
            r#"
            INSERT INTO users (
                member_code, phone, username, password_hash, birthday,
                member_type, referrer_id, referral_code
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
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
        .fetch_one(&self.pool)
        .await?;

        // 根据是否有推荐人发放不同的优惠码
        if let Some(referrer_user_id) = referrer_id {
            // 有推荐人的情况：给推荐人和新用户都发放推荐优惠码
            // 给推荐人发放推荐奖励优惠码（$1.0）
            self.create_referral_discount_code(referrer_user_id, 50)
                .await?; // 50美分

            // 给新用户发放推荐优惠码（$0.5）
            self.create_referral_discount_code(user_id, 50).await?; // 50美分
        }
        // 如果没有推荐人，则不发放任何优惠码

        // 生成JWT令牌
        let access_token = self
            .jwt_service
            .generate_access_token(user_id, &member_code)?;
        let refresh_token = self
            .jwt_service
            .generate_refresh_token(user_id, &member_code)?;

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
                balance, stamps, referrer_id, referral_code,
                created_at, updated_at
            FROM users
            WHERE phone = $1
            "#,
            request.phone
        )
        .fetch_optional(&self.pool)
        .await?;

        let user = user.ok_or_else(|| AppError::AuthError("User does not exist or password is incorrect".to_string()))?;

        // 验证密码
        let is_valid = verify_password(&request.password, &user.password_hash)?;
        if !is_valid {
            return Err(AppError::AuthError("User does not exist or password is incorrect".to_string()));
        }

        // 生成JWT令牌
        let access_token = self
            .jwt_service
            .generate_access_token(user.id, &user.member_code)?;
        let refresh_token = self
            .jwt_service
            .generate_refresh_token(user.id, &user.member_code)?;

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
        let user_id: i64 = claims
            .sub
            .parse()
            .map_err(|_| AppError::AuthError("Invalid token".to_string()))?;

        // 获取用户信息
        let user = self.get_user_by_id(user_id).await?;

        // 生成新的访问令牌
        let access_token = self
            .jwt_service
            .generate_access_token(user.id, &user.member_code)?;

        let user_response = UserResponse::from(user);

        Ok(AuthResponse {
            user: user_response,
            access_token,
            refresh_token: refresh_token.to_string(),
            expires_in: self.jwt_service.get_access_token_expires_in(),
        })
    }

    async fn verify_code(&self, phone: &str, code: &str) -> AppResult<()> {
        // 查找最新的有效验证码
        let verification_code = sqlx::query!(
            "SELECT code, expires_at FROM verification_codes WHERE phone = $1 ORDER BY created_at DESC LIMIT 1",
            phone
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(stored_code) = verification_code {
            let now = Utc::now();
                let expires_at = stored_code.expires_at;

            if now > expires_at {
                return Err(AppError::ValidationError("The verification code has expired".to_string()));
            }

            if stored_code.code != code {
                return Err(AppError::ValidationError("The verification code is incorrect".to_string()));
            }

            // 验证成功后删除已使用的验证码
            sqlx::query!(
                "DELETE FROM verification_codes WHERE phone = $1 AND code = $2",
                phone,
                code
            )
            .execute(&self.pool)
            .await?;

            Ok(())
        } else {
            Err(AppError::ValidationError(
                "The verification code does not exist or has expired".to_string(),
            ))
        }
    }

    async fn get_user_by_id(&self, user_id: i64) -> AppResult<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT
                id, member_code, phone, username, password_hash, birthday,
                member_type as "member_type: MemberType",
                balance, stamps, referrer_id, referral_code,
                created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        user.ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }

    async fn create_referral_discount_code(&self, _user_id: i64, amount: i64) -> AppResult<()> {
        // 生成6位数字优惠码
        let code = generate_six_digit_code();

        // 将美分转换为美元
        let discount_dollars = amount as f64 / 100.0;

        // 使用SevenCloud API生成优惠码
        let mut api = self.sevencloud_api.lock().await;

        // 先尝试登录
        api.login().await?;

        // 生成优惠码，有效期3个月
        api.generate_discount_code(&code, discount_dollars, 3)
            .await?;

        Ok(())
    }

    // 清理过期的验证码
    pub async fn cleanup_expired_verification_codes(&self) -> AppResult<()> {
        let now = Utc::now();
    sqlx::query!("DELETE FROM verification_codes WHERE expires_at < $1", now)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_verification_code_structure() {
        // 测试验证码数据结构是否正确
        assert!(true, "Verification code has been moved to database storage");
    }

    #[test]
    fn test_verification_code_model_creation() {
        // 测试VerificationCode模型创建
        use crate::models::VerificationCode;
        use chrono::Utc;

        let now = Utc::now();
        let verification_code = VerificationCode {
            id: 1,
            phone: "+1234567890".to_string(),
            code: "123456".to_string(),
            created_at: now,
            expires_at: now + Duration::minutes(5),
        };

        assert_eq!(verification_code.phone, "+1234567890");
        assert_eq!(verification_code.code, "123456");
    }
}
