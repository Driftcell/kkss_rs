use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;
use sea_orm_migration::prelude::extension::postgres::Type;

#[derive(DeriveIden)]
enum Users {
    Table,
    MemberType,
}

#[derive(DeriveIden)]
enum DiscountCodes {
    Table,
    CodeType,
}

#[derive(DeriveIden)]
enum RechargeRecords {
    Table,
    Status,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create Postgres ENUM types if not exists
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("member_type"))
                    .values(vec![
                        Alias::new("fan"),
                        Alias::new("sweet_shareholder"),
                        Alias::new("super_shareholder"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("code_type"))
                    .values(vec![
                        Alias::new("welcome"),
                        Alias::new("referral"),
                        Alias::new("purchase_reward"),
                        Alias::new("redeemed"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("recharge_status"))
                    .values(vec![
                        Alias::new("pending"),
                        Alias::new("succeeded"),
                        Alias::new("failed"),
                        Alias::new("canceled"),
                    ])
                    .to_owned(),
            )
            .await?;

        // Alter columns to use enum types
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .modify_column(
                        ColumnDef::new(Users::MemberType)
                            .custom(Alias::new("member_type"))
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(DiscountCodes::Table)
                    .modify_column(
                        ColumnDef::new(DiscountCodes::CodeType)
                            .custom(Alias::new("code_type"))
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(RechargeRecords::Table)
                    .modify_column(
                        ColumnDef::new(RechargeRecords::Status)
                            .custom(Alias::new("recharge_status"))
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
