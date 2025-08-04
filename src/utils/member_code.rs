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

/// 生成唯一的六位数字推荐码
pub async fn generate_unique_referral_code(pool: &SqlitePool) -> AppResult<String> {
    let mut rng = rand::thread_rng();
    
    loop {
        // 生成100000到999999之间的六位数字
        let referral_code = rng.gen_range(100000_u32..=999999_u32).to_string();
        
        // 检查是否已存在
        let exists = sqlx::query!(
            "SELECT COUNT(*) as count FROM users WHERE referral_code = ?",
            referral_code
        )
        .fetch_one(pool)
        .await?;
        
        if exists.count == 0 {
            return Ok(referral_code);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn test_generate_unique_referral_code() {
        // 使用内存数据库进行测试
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        
        // 创建测试用户表
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY,
                referral_code TEXT UNIQUE
            )
            "#
        )
        .execute(&pool)
        .await
        .unwrap();

        let code = generate_unique_referral_code(&pool).await.unwrap();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
        
        // 确保代码在有效范围内
        let code_num: u32 = code.parse().unwrap();
        assert!(code_num >= 100000 && code_num <= 999999);
        
        // 插入一个推荐码到数据库
        sqlx::query!(
            "INSERT INTO users (referral_code) VALUES (?)",
            code
        )
        .execute(&pool)
        .await
        .unwrap();
        
        // 生成另一个推荐码，应该与第一个不同
        let code2 = generate_unique_referral_code(&pool).await.unwrap();
        assert_ne!(code, code2);
    }
}
