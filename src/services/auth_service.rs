use crate::entities::user_entity as users;
use crate::error::{AppError, AppResult};
use crate::external::*;
use crate::models::*;
use crate::utils::*;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, Set,
};

#[derive(Clone)]
pub struct AuthService {
    pool: DatabaseConnection,
    jwt_service: JwtService,
    twilio_service: TwilioService,
}

impl AuthService {
    pub fn new(
        pool: DatabaseConnection,
        jwt_service: JwtService,
        twilio_service: TwilioService,
    ) -> Self {
        Self {
            pool,
            jwt_service,
            twilio_service,
        }
    }

    /// 发送验证码到指定手机号
    ///
    /// # 参数
    ///
    /// * `phone`: 手机号
    ///
    /// # 返回值
    ///
    /// 返回一个包含验证码有效期的响应
    pub async fn send_verification_code(&self, phone: &str) -> AppResult<SendCodeResponse> {
        // 验证手机号格式
        validate_us_phone(phone)?;

        // 依赖 Twilio Verify 自身的速率限制与风控，这里不再读写本地库
        self.twilio_service.start_verification_sms(phone).await?;

        // Twilio Verify 默认验证码有效期 10 分钟
        Ok(SendCodeResponse { expires_in: 600 })
    }

    /// 用户注册
    ///
    /// # 参数
    ///
    /// * `request`: 注册请求
    ///
    /// # 返回值
    ///
    /// 返回一个包含用户信息的响应
    pub async fn register(&self, request: CreateUserRequest) -> AppResult<AuthResponse> {
        // 验证输入参数
        validate_us_phone(&request.phone)?;
        validate_password(&request.password)?;

        // 验证验证码（通过 Twilio Verify）
        let approved = self
            .twilio_service
            .check_verification_code(&request.phone, &request.verification_code)
            .await?;
        if !approved {
            return Err(AppError::ValidationError(
                "The verification code is incorrect or expired".to_string(),
            ));
        }

        // 检查手机号是否已注册
        let existing_user = users::Entity::find()
            .filter(users::Column::Phone.eq(request.phone.clone()))
            .one(&self.pool)
            .await?;
        if existing_user.is_some() {
            return Err(AppError::ValidationError(
                "The mobile phone number is registered".to_string(),
            ));
        }

        // 解析生日
        let birthday = chrono::NaiveDate::parse_from_str(&request.birthday, "%Y-%m-%d")
            .map_err(|_| AppError::ValidationError("Invalid birthday format".to_string()))?;

        // 从手机号生成会员号（去掉+1前缀的十位数字）
        let member_code = extract_member_code_from_phone(&request.phone)?;

        // 检查会员号是否已存在（防止重复注册）
        let existing_member = users::Entity::find()
            .filter(users::Column::MemberCode.eq(member_code.clone()))
            .one(&self.pool)
            .await?;
        if existing_member.is_some() {
            return Err(AppError::ValidationError(
                "The member code corresponding to this phone number already exists".to_string(),
            ));
        }

        // 密码哈希
        let password_hash = hash_password(&request.password)?;

        // 处理推荐人，要求推荐人不是 fan
        let (referrer_id, member_type) = if let Some(referrer_code) = &request.referrer_code {
            let ref_row = users::Entity::find()
                .filter(users::Column::MemberCode.eq(referrer_code.clone()))
                .one(&self.pool)
                .await?;

            if let Some(row) = ref_row {
                let rid = row.id;
                let rtype = row.member_type;
                if rtype == MemberType::Fan {
                    return Err(AppError::ValidationError(
                        "The referrer is not eligible (fan)".to_string(),
                    ));
                }
                (Some(rid), MemberType::Fan)
            } else {
                return Err(AppError::ValidationError(
                    "The referrer does not exist".to_string(),
                ));
            }
        } else {
            (None, MemberType::Fan)
        };

        // 生成推荐码
        let referral_code = generate_unique_referral_code(&self.pool).await?;

        // 插入用户
        let new_user = users::ActiveModel {
            member_code: Set(member_code.clone()),
            phone: Set(request.phone.clone()),
            username: Set(request.username.clone()),
            password_hash: Set(password_hash),
            birthday: Set(birthday),
            member_type: Set(member_type),
            membership_expires_at: sea_orm::ActiveValue::NotSet,
            balance: sea_orm::ActiveValue::NotSet,
            stamps: sea_orm::ActiveValue::NotSet,
            referrer_id: Set(referrer_id),
            referral_code: Set(Some(referral_code.clone())),
            ..Default::default()
        }
        .insert(&self.pool)
        .await?;
        let user_id = new_user.id;

        // 生成JWT令牌
        let access_token = self
            .jwt_service
            .generate_access_token(user_id, &member_code)?;
        let refresh_token = self
            .jwt_service
            .generate_refresh_token(user_id, &member_code)?;

        // 获取完整用户信息（包含推荐人数）
        let user_response = self.get_user_with_referrals(user_id).await?;

        Ok(AuthResponse {
            user: user_response,
            access_token,
            refresh_token,
            expires_in: self.jwt_service.get_access_token_expires_in(),
        })
    }

    /// 用户登录
    ///
    /// # 参数
    ///
    /// * `request`: 登录请求
    ///
    /// # 返回值
    /// 返回一个包含用户信息的响应
    pub async fn login(&self, request: LoginRequest) -> AppResult<AuthResponse> {
        // 验证手机号格式
        validate_us_phone(&request.phone)?;
        // 通过手机号获取用户（避免重复查询）
        let user = self.get_user_by_phone(&request.phone).await.map_err(|_| {
            AppError::AuthError("User does not exist or password is incorrect".to_string())
        })?;

        // 验证密码
        let is_valid = verify_password(&request.password, &user.password_hash)?;
        if !is_valid {
            return Err(AppError::AuthError(
                "User does not exist or password is incorrect".to_string(),
            ));
        }

        // 生成JWT令牌
        let access_token = self
            .jwt_service
            .generate_access_token(user.id, &user.member_code)?;
        let refresh_token = self
            .jwt_service
            .generate_refresh_token(user.id, &user.member_code)?;

        // 使用已获取的 user 构建带推荐数的响应，避免再次按 id 查询
        let user_response = self.build_user_response_with_referrals(user).await?;

        Ok(AuthResponse {
            user: user_response,
            access_token,
            refresh_token,
            expires_in: self.jwt_service.get_access_token_expires_in(),
        })
    }

    /// 刷新用户令牌
    ///
    /// # 参数
    ///
    /// * `refresh_token`: 刷新令牌
    ///
    /// # 返回值
    ///
    /// 返回一个包含用户信息的响应
    pub async fn refresh_token(&self, refresh_token: &str) -> AppResult<AuthResponse> {
        // 验证刷新令牌
        let claims = self.jwt_service.verify_refresh_token(refresh_token)?;
        let user_id: i64 = claims
            .sub
            .parse()
            .map_err(|_| AppError::AuthError("Invalid token".to_string()))?;

        // 获取用户信息
        let user_response = self.get_user_with_referrals(user_id).await?;

        // 生成新的访问令牌
        let access_token = self
            .jwt_service
            .generate_access_token(user_response.id, &user_response.member_code)?;

        Ok(AuthResponse {
            user: user_response,
            access_token,
            refresh_token: refresh_token.to_string(),
            expires_in: self.jwt_service.get_access_token_expires_in(),
        })
    }

    /// 根据用户ID获取用户信息
    ///
    /// # 参数
    ///
    /// * `user_id`: 用户ID
    ///
    /// # 返回值
    ///
    /// 返回用户信息
    async fn get_user_by_id(&self, user_id: i64) -> AppResult<User> {
        let u = users::Entity::find_by_id(user_id).one(&self.pool).await?;
        let u = u.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        Ok(map_user_model(u))
    }

    /// 根据手机号获取用户信息
    ///
    /// # 参数
    ///
    /// * `phone`: 用户手机号
    ///
    /// # 返回值
    ///
    /// 返回用户信息
    async fn get_user_by_phone(&self, phone: &str) -> AppResult<User> {
        let u = users::Entity::find()
            .filter(users::Column::Phone.eq(phone.to_string()))
            .one(&self.pool)
            .await?;
        let u = u.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        Ok(map_user_model(u))
    }

    /// 构建用户响应
    ///
    /// # 参数
    ///
    /// * `user`: 用户
    ///
    /// # 返回值
    ///
    /// 返回用户响应
    async fn build_user_response_with_referrals(&self, user: User) -> AppResult<UserResponse> {
        #[derive(Debug, sea_orm::FromQueryResult)]
        struct CountRow {
            count: i64,
        }
        let total_referrals = users::Entity::find()
            .filter(users::Column::ReferrerId.eq(user.id))
            .select_only()
            .column_as(Expr::val(1).count(), "count")
            .into_model::<CountRow>()
            .one(&self.pool)
            .await?
            .map(|r| r.count)
            .unwrap_or(0);

        let mut user_response = UserResponse::from(user);
        user_response.total_referrals = total_referrals;
        Ok(user_response)
    }

    async fn get_user_with_referrals(&self, user_id: i64) -> AppResult<UserResponse> {
        let user = self.get_user_by_id(user_id).await?;
        self.build_user_response_with_referrals(user).await
    }
}

fn map_user_model(m: users::Model) -> User {
    User {
        id: m.id,
        member_code: m.member_code,
        phone: m.phone,
        username: m.username,
        password_hash: m.password_hash,
        birthday: m.birthday,
        member_type: m.member_type,
        membership_expires_at: m.membership_expires_at,
        balance: m.balance,
        stamps: m.stamps,
        referrer_id: m.referrer_id,
        referral_code: m.referral_code,
        created_at: m.created_at,
        updated_at: m.updated_at,
    }
}
