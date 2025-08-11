use crate::error::AppResult;
use rand::Rng;
use sqlx::PgPool;

/// 生成唯一的10位会员号
pub async fn generate_unique_member_code(pool: &PgPool) -> AppResult<String> {
    let mut rng = rand::thread_rng();

    loop {
        // 生成1000000001到9999999999之间的数字
        let member_code = rng.gen_range(1000000001_u64..=9999999999_u64).to_string();

        // 检查是否已存在
        let exists = sqlx::query!(
            "SELECT COUNT(*) as count FROM users WHERE member_code = $1",
            member_code
        )
        .fetch_one(pool)
        .await?;

        if exists.count.unwrap_or(0) == 0 {
            return Ok(member_code);
        }
    }
}

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

/// 生成唯一的优惠码
pub async fn generate_unique_discount_code(pool: &PgPool) -> AppResult<String> {
    let mut rng = rand::thread_rng();

    loop {
        // 生成8位字母数字组合的优惠码
        let code: String = (0..8)
            .map(|_| {
                let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
                chars[rng.gen_range(0..chars.len())] as char
            })
            .collect();

        // 检查是否已存在
        let exists = sqlx::query!(
            "SELECT COUNT(*) as count FROM discount_codes WHERE code = $1",
            code
        )
        .fetch_one(pool)
        .await?;

        if exists.count.unwrap_or(0) == 0 {
            return Ok(code);
        }
    }
}

// TODO: 添加 Postgres 集成测试 (testcontainers 或 docker)
