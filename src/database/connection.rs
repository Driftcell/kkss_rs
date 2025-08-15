use crate::config::DatabaseConfig;
use crate::error::AppResult;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

pub type DbConn = DatabaseConnection;

pub async fn create_pool(config: &DatabaseConfig) -> AppResult<DbConn> {
    let mut opt = ConnectOptions::new(config.url.clone());
    opt.max_connections(config.max_connections)
        .sqlx_logging(true);
    let conn = Database::connect(opt).await?;
    Ok(conn)
}

pub async fn run_migrations(conn: &DbConn) -> AppResult<()> {
    use migration::MigratorTrait;
    // Cast to the migration crate's DatabaseConnection reference to satisfy IntoSchemaManagerConnection
    migration::Migrator::up(conn as &migration::sea_orm::DatabaseConnection, None).await?;
    Ok(())
}
