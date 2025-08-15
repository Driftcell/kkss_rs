use crate::error::AppResult;
use rand::Rng;
use sea_orm::DatabaseConnection;

/// 生成唯一的六位数字推荐码
pub async fn generate_unique_referral_code(_pool: &DatabaseConnection) -> AppResult<String> {
    let mut rng = rand::thread_rng();

    loop {
        // 生成100000到999999之间的六位数字
        let referral_code = rng.gen_range(100000_u32..=999999_u32).to_string();

        // 检查是否已存在
        // TODO: replace with SeaORM query; currently return the candidate directly
        return Ok(referral_code);
    }
}
