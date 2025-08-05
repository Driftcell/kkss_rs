use crate::error::{AppError, AppResult};
use regex::Regex;

/// 验证美国手机号格式
pub fn validate_us_phone(phone: &str) -> AppResult<()> {
    let phone_regex = Regex::new(r"^\+1\d{10}$").unwrap();

    if !phone_regex.is_match(phone) {
        return Err(AppError::ValidationError(
            "Invalid US phone number format, must be (+1xxxxxxxxxx)".to_string(),
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

/// 从美国手机号中提取十位数字作为member_code
pub fn extract_member_code_from_phone(phone: &str) -> AppResult<String> {
    let digits: String = phone.chars().filter(|c| c.is_digit(10)).collect();

    if digits.len() == 11 && digits.starts_with('1') {
        // +1234567890 -> 2345678901
        Ok(digits[1..].to_string())
    } else if digits.len() == 10 {
        // 2345678901 -> 2345678901
        Ok(digits)
    } else {
        Err(AppError::ValidationError(
            "Failed to extract member code from phone number".to_string(),
        ))
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

    #[test]
    fn test_extract_member_code_from_phone() {
        assert_eq!(
            extract_member_code_from_phone("+12345678901").unwrap(),
            "2345678901"
        );
        assert_eq!(
            extract_member_code_from_phone("12345678901").unwrap(),
            "2345678901"
        );
        assert_eq!(
            extract_member_code_from_phone("2345678901").unwrap(),
            "2345678901"
        );
        assert_eq!(
            extract_member_code_from_phone("+1(234) 567-8901").unwrap(),
            "2345678901"
        );
        assert!(extract_member_code_from_phone("+12345").is_err());
        assert!(extract_member_code_from_phone("12345").is_err());
    }
}
