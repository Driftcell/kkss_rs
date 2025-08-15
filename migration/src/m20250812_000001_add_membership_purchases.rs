use sea_orm_migration::prelude::*;
use sea_orm_migration::prelude::extension::postgres::Type;

#[derive(DeriveIden)]
enum MembershipPurchases {
    Table,
    Id,
    UserId,
    StripePaymentIntentId,
    TargetMemberType,
    Amount,
    Status,
    StripeStatus,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ensure membership_purchase_status exists
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("membership_purchase_status"))
                    .values(vec![
                        Alias::new("pending"),
                        Alias::new("succeeded"),
                        Alias::new("failed"),
                        Alias::new("canceled"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(MembershipPurchases::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MembershipPurchases::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(MembershipPurchases::UserId).big_integer().not_null())
                    .col(
                        ColumnDef::new(MembershipPurchases::StripePaymentIntentId)
                            .string_len(255)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(MembershipPurchases::TargetMemberType)
                            .custom(Alias::new("member_type"))
                            .not_null(),
                    )
                    .col(ColumnDef::new(MembershipPurchases::Amount).big_integer().not_null())
                    .col(
                        ColumnDef::new(MembershipPurchases::Status)
                            .custom(Alias::new("membership_purchase_status"))
                            .not_null()
                            .default("'pending'"),
                    )
                    .col(ColumnDef::new(MembershipPurchases::StripeStatus).string_len(50).null())
                    .col(
                        ColumnDef::new(MembershipPurchases::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::cust("NOW()"))
                            .null(),
                    )
                    .col(
                        ColumnDef::new(MembershipPurchases::UpdatedAt)
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
                    .name("idx_membership_purchases_user_id")
                    .table(MembershipPurchases::Table)
                    .col(MembershipPurchases::UserId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_membership_purchases_payment_intent")
                    .table(MembershipPurchases::Table)
                    .col(MembershipPurchases::StripePaymentIntentId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
