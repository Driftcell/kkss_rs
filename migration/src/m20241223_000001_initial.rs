use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    MemberCode,
    Phone,
    Username,
    PasswordHash,
    Birthday,
    MemberType,
    Balance,
    Stamps,
    ReferrerId,
    ReferralCode,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    Id,
    UserId,
    MemberCode,
    Price,
    ProductName,
    ProductNo,
    OrderStatus,
    PayType,
    StampsEarned,
    ExternalCreatedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum DiscountCodes {
    Table,
    Id,
    UserId,
    Code,
    DiscountAmount,
    CodeType,
    IsUsed,
    UsedAt,
    ExpiresAt,
    ExternalId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum RechargeRecords {
    Table,
    Id,
    UserId,
    StripePaymentIntentId,
    Amount,
    BonusAmount,
    TotalAmount,
    Status,
    StripeStatus,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum SweetCashTransactions {
    Table,
    Id,
    UserId,
    TransactionType,
    Amount,
    BalanceAfter,
    RelatedOrderId,
    RelatedDiscountCodeId,
    Description,
    CreatedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // users
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Users::MemberCode).string().not_null().unique_key())
                    .col(ColumnDef::new(Users::Phone).string().not_null().unique_key())
                    .col(ColumnDef::new(Users::Username).string().not_null())
                    .col(ColumnDef::new(Users::PasswordHash).string().not_null())
                    .col(ColumnDef::new(Users::Birthday).date().not_null())
                    .col(ColumnDef::new(Users::MemberType).string().not_null())
                    .col(ColumnDef::new(Users::Balance).big_integer().not_null().default(0))
                    .col(ColumnDef::new(Users::Stamps).big_integer().not_null().default(0))
                    .col(ColumnDef::new(Users::ReferrerId).big_integer().null())
                    .col(ColumnDef::new(Users::ReferralCode).string().unique_key().null())
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::cust("NOW()"))
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::cust("NOW()"))
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // FK referrer_id -> users.id
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_users_referrer")
                    .from(Users::Table, Users::ReferrerId)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::NoAction)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        // orders
        manager
            .create_table(
                Table::create()
                    .table(Orders::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Orders::Id)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Orders::UserId).big_integer().not_null())
                    .col(ColumnDef::new(Orders::MemberCode).string().null())
                    .col(ColumnDef::new(Orders::Price).big_integer().not_null())
                    .col(ColumnDef::new(Orders::ProductName).string().not_null())
                    .col(ColumnDef::new(Orders::ProductNo).string().null())
                    .col(ColumnDef::new(Orders::OrderStatus).integer().not_null())
                    .col(ColumnDef::new(Orders::PayType).integer().null())
                    .col(ColumnDef::new(Orders::StampsEarned).big_integer().not_null().default(0))
                    .col(
                        ColumnDef::new(Orders::ExternalCreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Orders::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::cust("NOW()"))
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Orders::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::cust("NOW()"))
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_orders_user")
                    .from(Orders::Table, Orders::UserId)
                    .to(Users::Table, Users::Id)
                    .to_owned(),
            )
            .await?;

        // discount_codes
        manager
            .create_table(
                Table::create()
                    .table(DiscountCodes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DiscountCodes::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(DiscountCodes::UserId).big_integer().not_null())
                    .col(ColumnDef::new(DiscountCodes::Code).string().not_null().unique_key())
                    .col(ColumnDef::new(DiscountCodes::DiscountAmount).big_integer().not_null())
                    .col(ColumnDef::new(DiscountCodes::CodeType).string().not_null())
                    .col(ColumnDef::new(DiscountCodes::IsUsed).boolean().not_null().default(false))
                    .col(ColumnDef::new(DiscountCodes::UsedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(DiscountCodes::ExpiresAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(DiscountCodes::ExternalId).big_integer().null())
                    .col(
                        ColumnDef::new(DiscountCodes::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::cust("NOW()"))
                            .null(),
                    )
                    .col(
                        ColumnDef::new(DiscountCodes::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::cust("NOW()"))
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_discount_codes_user")
                    .from(DiscountCodes::Table, DiscountCodes::UserId)
                    .to(Users::Table, Users::Id)
                    .to_owned(),
            )
            .await?;

        // recharge_records
        manager
            .create_table(
                Table::create()
                    .table(RechargeRecords::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RechargeRecords::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(RechargeRecords::UserId).big_integer().not_null())
                    .col(
                        ColumnDef::new(RechargeRecords::StripePaymentIntentId)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(RechargeRecords::Amount).big_integer().not_null())
                    .col(ColumnDef::new(RechargeRecords::BonusAmount).big_integer().not_null())
                    .col(ColumnDef::new(RechargeRecords::TotalAmount).big_integer().not_null())
                    .col(ColumnDef::new(RechargeRecords::Status).string().not_null())
                    .col(ColumnDef::new(RechargeRecords::StripeStatus).string().null())
                    .col(
                        ColumnDef::new(RechargeRecords::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::cust("NOW()"))
                            .null(),
                    )
                    .col(
                        ColumnDef::new(RechargeRecords::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::cust("NOW()"))
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_recharge_records_user")
                    .from(RechargeRecords::Table, RechargeRecords::UserId)
                    .to(Users::Table, Users::Id)
                    .to_owned(),
            )
            .await?;

        // sweet_cash_transactions
        manager
            .create_table(
                Table::create()
                    .table(SweetCashTransactions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SweetCashTransactions::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SweetCashTransactions::UserId).big_integer().not_null())
                    .col(ColumnDef::new(SweetCashTransactions::TransactionType).string().not_null())
                    .col(ColumnDef::new(SweetCashTransactions::Amount).big_integer().not_null())
                    .col(ColumnDef::new(SweetCashTransactions::BalanceAfter).big_integer().not_null())
                    .col(ColumnDef::new(SweetCashTransactions::RelatedOrderId).big_integer().null())
                    .col(ColumnDef::new(SweetCashTransactions::RelatedDiscountCodeId).big_integer().null())
                    .col(ColumnDef::new(SweetCashTransactions::Description).string().null())
                    .col(
                        ColumnDef::new(SweetCashTransactions::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::cust("NOW()"))
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_sct_user")
                    .from(SweetCashTransactions::Table, SweetCashTransactions::UserId)
                    .to(Users::Table, Users::Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_sct_order")
                    .from(
                        SweetCashTransactions::Table,
                        SweetCashTransactions::RelatedOrderId,
                    )
                    .to(Orders::Table, Orders::Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_sct_discount_code")
                    .from(
                        SweetCashTransactions::Table,
                        SweetCashTransactions::RelatedDiscountCodeId,
                    )
                    .to(DiscountCodes::Table, DiscountCodes::Id)
                    .to_owned(),
            )
            .await?;

        // indexes
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_users_member_code")
                    .table(Users::Table)
                    .col(Users::MemberCode)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_users_phone")
                    .table(Users::Table)
                    .col(Users::Phone)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_orders_user_id")
                    .table(Orders::Table)
                    .col(Orders::UserId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_orders_member_code")
                    .table(Orders::Table)
                    .col(Orders::MemberCode)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_orders_external_created_at")
                    .table(Orders::Table)
                    .col(Orders::ExternalCreatedAt)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_discount_codes_user_id")
                    .table(DiscountCodes::Table)
                    .col(DiscountCodes::UserId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_discount_codes_code")
                    .table(DiscountCodes::Table)
                    .col(DiscountCodes::Code)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_discount_codes_expires_at")
                    .table(DiscountCodes::Table)
                    .col(DiscountCodes::ExpiresAt)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_recharge_records_user_id")
                    .table(RechargeRecords::Table)
                    .col(RechargeRecords::UserId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_recharge_records_stripe_payment_intent_id")
                    .table(RechargeRecords::Table)
                    .col(RechargeRecords::StripePaymentIntentId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_sweet_cash_transactions_user_id")
                    .table(SweetCashTransactions::Table)
                    .col(SweetCashTransactions::UserId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_sweet_cash_transactions_transaction_type")
                    .table(SweetCashTransactions::Table)
                    .col(SweetCashTransactions::TransactionType)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop in reverse order
        manager
            .drop_table(Table::drop().if_exists().table(SweetCashTransactions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().if_exists().table(RechargeRecords::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().if_exists().table(DiscountCodes::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().if_exists().table(Orders::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().if_exists().table(Users::Table).to_owned())
            .await?;
        Ok(())
    }
}
