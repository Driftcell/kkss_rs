use sea_orm_migration::prelude::*;
#[derive(DeriveIden)]
enum VerificationCodes {
    Table,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if manager.has_table("verification_codes").await? {
            manager
                .drop_table(
                    Table::drop()
                        .table(VerificationCodes::Table)
                        .if_exists()
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
