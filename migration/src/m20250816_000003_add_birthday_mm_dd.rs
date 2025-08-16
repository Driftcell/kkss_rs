use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum Users {
    Table,
    BirthdayMonth,
    BirthdayDay,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1) add nullable columns first
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(ColumnDef::new(Users::BirthdayMonth).small_integer().null())
                    .add_column(ColumnDef::new(Users::BirthdayDay).small_integer().null())
                    .to_owned(),
            )
            .await?;

        // 2) backfill from birthday
        use sea_orm::sea_query::{Expr, Query};
        let backfill = Query::update()
            .table(Users::Table)
            .values([
                (Users::BirthdayMonth, Expr::cust("EXTRACT(MONTH FROM \"birthday\")::smallint")),
                (Users::BirthdayDay, Expr::cust("EXTRACT(DAY FROM \"birthday\")::smallint")),
            ])
            .to_owned();
        manager.exec_stmt(backfill).await?;

        // 3) set NOT NULL
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .modify_column(ColumnDef::new(Users::BirthdayMonth).small_integer().not_null())
                    .modify_column(ColumnDef::new(Users::BirthdayDay).small_integer().not_null())
                    .to_owned(),
            )
            .await?;

        // 4) indexes (optional)
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_users_birthday_mm")
                    .table(Users::Table)
                    .col(Users::BirthdayMonth)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_users_birthday_dd")
                    .table(Users::Table)
                    .col(Users::BirthdayDay)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::BirthdayMonth)
                    .drop_column(Users::BirthdayDay)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
