use crate::error::AppResult;
use rand::Rng;
use sqlx::PgPool;

/// 生成唯一的六位数字推荐码
pub async fn generate_unique_referral_code(pool: &PgPool) -> AppResult<String> {
    let mut rng = rand::thread_rng();

    loop {
        // 生成100000到999999之间的六位数字
        let referral_code = rng.gen_range(100000_u32..=999999_u32).to_string();

        // 检查是否已存在
        let exists = sqlx::query!(
            "SELECT COUNT(*) as count FROM users WHERE referral_code = $1",
            referral_code
        )
        .fetch_one(pool)
        .await?;

        if exists.count.unwrap_or(0) == 0 {
            return Ok(referral_code);
        }
    }
}
