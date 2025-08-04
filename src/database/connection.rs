use crate::config::DatabaseConfig;
use crate::error::AppResult;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

pub type DbPool = SqlitePool;

pub async fn create_pool(config: &DatabaseConfig) -> AppResult<DbPool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.url)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &DbPool) -> AppResult<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}
