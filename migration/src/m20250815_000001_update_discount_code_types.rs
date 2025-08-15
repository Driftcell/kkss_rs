use sea_orm::Statement;
use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum DiscountCodes {
    Table,
    CodeType,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create new enum type code_type_new
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("code_type_new"))
                    .values(vec![
                        Alias::new("shareholder_reward"),
                        Alias::new("super_shareholder_reward"),
                        Alias::new("sweets_credits_reward"),
                    ])
                    .to_owned(),
            )
            .await?;

        // Temporarily convert to TEXT to remap values
        manager
            .alter_table(
                Table::alter()
                    .table(DiscountCodes::Table)
                    .modify_column(ColumnDef::new(DiscountCodes::CodeType).string().not_null())
                    .to_owned(),
            )
            .await?;

        // Remap data with UPDATE CASE using Statement
        let stmt = Statement::from_string(
            manager.get_database_backend(),
            r#"UPDATE discount_codes SET code_type = CASE code_type
                WHEN 'welcome' THEN 'shareholder_reward'
                WHEN 'referral' THEN 'sweets_credits_reward'
                WHEN 'purchase_reward' THEN 'shareholder_reward'
                WHEN 'redeemed' THEN 'sweets_credits_reward'
                ELSE 'sweets_credits_reward'
            END"#
                .to_owned(),
        );
        manager.get_connection().execute(stmt).await?;

        // Drop old type and recreate code_type with new variants
        manager
            .drop_type(Type::drop().name(Alias::new("code_type")).to_owned())
            .await
            .ok();
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("code_type"))
                    .values(vec![
                        Alias::new("shareholder_reward"),
                        Alias::new("super_shareholder_reward"),
                        Alias::new("sweets_credits_reward"),
                    ])
                    .to_owned(),
            )
            .await?;

        // Convert column to new enum type
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

        // Finally, drop the temp type if exists
        manager
            .drop_type(Type::drop().name(Alias::new("code_type_new")).to_owned())
            .await
            .ok();
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
