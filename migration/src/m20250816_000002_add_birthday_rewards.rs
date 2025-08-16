use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum BirthdayRewards {
    Table,
    Id,
    UserId,
    RewardYear,
    Amount,
    CreatedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BirthdayRewards::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BirthdayRewards::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(BirthdayRewards::UserId).big_integer().not_null())
                    .col(ColumnDef::new(BirthdayRewards::RewardYear).integer().not_null())
                    .col(ColumnDef::new(BirthdayRewards::Amount).big_integer().not_null())
                    .col(
                        ColumnDef::new(BirthdayRewards::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::cust("NOW()"))
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // unique (user_id, reward_year)
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("uq_birthday_rewards_user_year")
                    .table(BirthdayRewards::Table)
                    .col(BirthdayRewards::UserId)
                    .col(BirthdayRewards::RewardYear)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BirthdayRewards::Table).to_owned())
            .await?;
        Ok(())
    }
}
