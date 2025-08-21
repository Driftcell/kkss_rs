use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum StripeTransactions {
    Table,
    Id,
    UserId,
    Category,
    PaymentIntentId,
    ChargeId,
    RefundId,
    SubscriptionId,
    InvoiceId,
    Amount,
    Currency,
    Status,
    Description,
    RawEvent,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // enum for category
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("stripe_transaction_category"))
                    .values(vec![
                        Alias::new("recharge"),
                        Alias::new("membership"),
                        Alias::new("monthly_card"),
                    ])
                    .to_owned(),
            )
            .await?;

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
                        ColumnDef::new(StripeTransactions::Category)
                            .custom(Alias::new("stripe_transaction_category"))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StripeTransactions::PaymentIntentId)
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StripeTransactions::ChargeId)
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StripeTransactions::RefundId)
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StripeTransactions::SubscriptionId)
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StripeTransactions::InvoiceId)
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StripeTransactions::Amount)
                            .big_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StripeTransactions::Currency)
                            .string_len(10)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StripeTransactions::Status)
                            .string_len(50)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StripeTransactions::Description)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StripeTransactions::RawEvent)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StripeTransactions::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::cust("NOW()"))
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StripeTransactions::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::cust("NOW()"))
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // indexes
        for (name, col) in [
            (
                "idx_stx_payment_intent",
                StripeTransactions::PaymentIntentId,
            ),
            ("idx_stx_charge", StripeTransactions::ChargeId),
            ("idx_stx_refund", StripeTransactions::RefundId),
            ("idx_stx_subscription", StripeTransactions::SubscriptionId),
            ("idx_stx_invoice", StripeTransactions::InvoiceId),
            ("idx_stx_user", StripeTransactions::UserId),
        ] {
            manager
                .create_index(
                    Index::create()
                        .if_not_exists()
                        .name(name)
                        .table(StripeTransactions::Table)
                        .col(col)
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .if_exists()
                    .table(StripeTransactions::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("stripe_transaction_category"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
