use bcrypt::{hash, verify, DEFAULT_COST};
use crate::error::{AppError, AppResult};

/// 验证密码强度
pub fn validate_password(password: &str) -> AppResult<()> {
    if password.len() < 8 || password.len() > 128 {
        return Err(AppError::ValidationError(
            "密码长度必须在8-128字符之间".to_string()
        ));
    }

    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_digit(10));

    if !has_lowercase || !has_uppercase || !has_digit {
        return Err(AppError::ValidationError(
            "密码必须包含大小写字母和数字".to_string()
        ));
    }

    Ok(())
}

/// 对密码进行哈希
pub fn hash_password(password: &str) -> AppResult<String> {
    hash(password, DEFAULT_COST)
        .map_err(|e| AppError::InternalError(format!("密码哈希失败: {}", e)))
}

/// 验证密码
pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    verify(password, hash)
        .map_err(|e| AppError::InternalError(format!("密码验证失败: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_password() {
        assert!(validate_password("Password123").is_ok());
        assert!(validate_password("password123").is_err()); // 缺少大写
        assert!(validate_password("PASSWORD123").is_err()); // 缺少小写
        assert!(validate_password("Password").is_err()); // 缺少数字
        assert!(validate_password("Pass123").is_err()); // 太短
    }

    #[test]
    fn test_hash_and_verify_password() {
        let password = "Password123";
        let hashed = hash_password(password).unwrap();
        
        assert!(verify_password(password, &hashed).unwrap());
        assert!(!verify_password("WrongPassword", &hashed).unwrap());
    }
}
