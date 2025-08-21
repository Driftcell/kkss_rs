use sea_orm_migration::prelude::*;
use sea_orm_migration::prelude::extension::postgres::Type;

#[derive(DeriveIden)]
enum StripeTransactions {
    Table,
    Id,
    UserId,
    StripePaymentIntentId,
    TransactionType,
    Amount,
    Status,
    Metadata,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create enum type for transaction types
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("stripe_transaction_type"))
                    .values(vec![
                        Alias::new("recharge"),
                        Alias::new("membership"),
                        Alias::new("month_card"),
                    ])
                    .to_owned(),
            )
            .await?;

        // Create enum type for stripe transaction status
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("stripe_transaction_status"))
                    .values(vec![
                        Alias::new("pending"),
                        Alias::new("succeeded"),
                        Alias::new("failed"),
                        Alias::new("canceled"),
                        Alias::new("refunded"),
                    ])
                    .to_owned(),
            )
            .await?;

        // Create the stripe_transactions table
        manager
            .create_table(
                Table::create()
                    .table(StripeTransactions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(StripeTransactions::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(StripeTransactions::UserId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StripeTransactions::StripePaymentIntentId)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(StripeTransactions::TransactionType)
                            .custom(Alias::new("stripe_transaction_type"))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StripeTransactions::Amount)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StripeTransactions::Status)
                            .custom(Alias::new("stripe_transaction_status"))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StripeTransactions::Metadata)
                            .json()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StripeTransactions::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(StripeTransactions::UpdatedAt)
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
                    .name("idx_stripe_transactions_user_id")
                    .table(StripeTransactions::Table)
                    .col(StripeTransactions::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_stripe_transactions_type")
                    .table(StripeTransactions::Table)
                    .col(StripeTransactions::TransactionType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_stripe_transactions_status")
                    .table(StripeTransactions::Table)
                    .col(StripeTransactions::Status)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(StripeTransactions::Table).to_owned())
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("stripe_transaction_type"))
                    .to_owned(),
            )
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("stripe_transaction_status"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}