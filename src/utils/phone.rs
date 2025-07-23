use regex::Regex;
use crate::error::{AppError, AppResult};

/// 验证美国手机号格式
pub fn validate_us_phone(phone: &str) -> AppResult<()> {
    let phone_regex = Regex::new(r"^\+1\d{10}$").unwrap();
    
    if !phone_regex.is_match(phone) {
        return Err(AppError::ValidationError(
            "手机号格式无效，必须是美国手机号格式 (+1xxxxxxxxxx)".to_string()
        ));
    }
    
    Ok(())
}

/// 格式化手机号，确保以+1开头
pub fn format_us_phone(phone: &str) -> String {
    let digits: String = phone.chars().filter(|c| c.is_digit(10)).collect();
    
    if digits.len() == 11 && digits.starts_with('1') {
        format!("+{}", digits)
    } else if digits.len() == 10 {
        format!("+1{}", digits)
    } else {
        phone.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_us_phone() {
        assert!(validate_us_phone("+12345678901").is_ok());
        assert!(validate_us_phone("+1234567890").is_err());
        assert!(validate_us_phone("12345678901").is_err());
        assert!(validate_us_phone("+22345678901").is_err());
    }

    #[test]
    fn test_format_us_phone() {
        assert_eq!(format_us_phone("2345678901"), "+12345678901");
        assert_eq!(format_us_phone("12345678901"), "+12345678901");
        assert_eq!(format_us_phone("+12345678901"), "+12345678901");
        assert_eq!(format_us_phone("(234) 567-8901"), "+12345678901");
    }
}
