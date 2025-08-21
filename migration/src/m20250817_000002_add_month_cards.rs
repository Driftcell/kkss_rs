use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum MonthCards {
    Table,
    Id,
    UserId,
    SubscriptionId,
    ProductId,
    PriceId,
    IsActive,
    StartDate,
    EndDate,
    CancelAtPeriodEnd,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create the month_cards table
        manager
            .create_table(
                Table::create()
                    .table(MonthCards::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MonthCards::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(MonthCards::UserId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MonthCards::SubscriptionId)
                            .string()
                            .null(), // Null for one-time purchases, filled for subscriptions
                    )
                    .col(
                        ColumnDef::new(MonthCards::ProductId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MonthCards::PriceId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MonthCards::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(MonthCards::StartDate)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MonthCards::EndDate)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MonthCards::CancelAtPeriodEnd)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(MonthCards::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(MonthCards::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_month_cards_user_id")
                    .table(MonthCards::Table)
                    .col(MonthCards::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_month_cards_subscription_id")
                    .table(MonthCards::Table)
                    .col(MonthCards::SubscriptionId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_month_cards_active")
                    .table(MonthCards::Table)
                    .col(MonthCards::IsActive)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MonthCards::Table).to_owned())
            .await
    }
}