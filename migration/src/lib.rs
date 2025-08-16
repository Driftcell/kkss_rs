pub use sea_orm_migration::prelude::*;

mod m20241223_000001_initial;
mod m20250810_000002_convert_text_enums;
mod m20250812_000001_add_membership_purchases;
mod m20250814_000001_drop_verification_codes_table;
mod m20250815_000001_update_discount_code_types;
mod m20250815_000002_add_membership_expires_at;
mod m20250816_000001_add_free_topping_code_type;
mod m20250816_000002_add_birthday_rewards;
mod m20250816_000003_add_birthday_mm_dd;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241223_000001_initial::Migration),
            Box::new(m20250810_000002_convert_text_enums::Migration),
            Box::new(m20250812_000001_add_membership_purchases::Migration),
            Box::new(m20250814_000001_drop_verification_codes_table::Migration),
            Box::new(m20250815_000001_update_discount_code_types::Migration),
            Box::new(m20250815_000002_add_membership_expires_at::Migration),
            Box::new(m20250816_000001_add_free_topping_code_type::Migration),
            Box::new(m20250816_000002_add_birthday_rewards::Migration),
            Box::new(m20250816_000003_add_birthday_mm_dd::Migration),
        ]
    }
}
