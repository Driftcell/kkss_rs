use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum MonthlyCards {
    Table,
    Id,
    UserId,
    PlanType,
    Status,
    StripeSubscriptionId,
    StartsAt,
    EndsAt,
    LastCouponGrantedOn,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // enums
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("monthly_card_plan_type"))
                    .values(vec![Alias::new("one_time"), Alias::new("subscription")])
                    .to_owned(),
            )
            .await?;
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("monthly_card_status"))
                    .values(vec![
                        Alias::new("pending"),
                        Alias::new("active"),
                        Alias::new("canceled"),
                        Alias::new("expired"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(MonthlyCards::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MonthlyCards::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(MonthlyCards::UserId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MonthlyCards::PlanType)
                            .custom(Alias::new("monthly_card_plan_type"))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MonthlyCards::Status)
                            .custom(Alias::new("monthly_card_status"))
                            .not_null()
                            .default(Expr::cust("'pending'::monthly_card_status")),
                    )
                    .col(
                        ColumnDef::new(MonthlyCards::StripeSubscriptionId)
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(MonthlyCards::StartsAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(MonthlyCards::EndsAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(MonthlyCards::LastCouponGrantedOn)
                            .date()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(MonthlyCards::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::cust("NOW()"))
                            .null(),
                    )
                    .col(
                        ColumnDef::new(MonthlyCards::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::cust("NOW()"))
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_monthly_cards_user")
                    .table(MonthlyCards::Table)
                    .col(MonthlyCards::UserId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .if_exists()
                    .table(MonthlyCards::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("monthly_card_status"))
                    .to_owned(),
            )
            .await?;
        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("monthly_card_plan_type"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
