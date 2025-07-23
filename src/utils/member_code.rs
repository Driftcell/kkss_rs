use rand::Rng;
use sqlx::SqlitePool;
use crate::error::AppResult;

/// 生成唯一的10位会员号
pub async fn generate_unique_member_code(pool: &SqlitePool) -> AppResult<String> {
    let mut rng = rand::thread_rng();
    
    loop {
        // 生成1000000001到9999999999之间的数字
        let member_code = rng.gen_range(1000000001_u64..=9999999999_u64).to_string();
        
        // 检查是否已存在
        let exists = sqlx::query!(
            "SELECT COUNT(*) as count FROM users WHERE member_code = ?",
            member_code
        )
        .fetch_one(pool)
        .await?;
        
        if exists.count == 0 {
            return Ok(member_code);
        }
    }
}

/// 生成推荐码
pub fn generate_referral_code() -> String {
    use rand::distributions::Alphanumeric;
    
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_referral_code() {
        let code = generate_referral_code();
        assert_eq!(code.len(), 32);
        assert!(code.chars().all(|c| c.is_alphanumeric()));
    }
}
