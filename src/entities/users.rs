use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{DeriveActiveEnum, EnumIter};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, DeriveActiveEnum, EnumIter,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "member_type")]
#[serde(rename_all = "snake_case")]
pub enum MemberType {
    #[sea_orm(string_value = "fan")]
    Fan,
    #[sea_orm(string_value = "sweet_shareholder")]
    SweetShareholder,
    #[sea_orm(string_value = "super_shareholder")]
    SuperShareholder,
}

impl std::fmt::Display for MemberType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemberType::Fan => write!(f, "fan"),
            MemberType::SweetShareholder => write!(f, "sweet_shareholder"),
            MemberType::SuperShareholder => write!(f, "super_shareholder"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub member_code: String,
    pub phone: String,
    pub username: String,
    pub password_hash: String,
    pub birthday: NaiveDate,
    pub member_type: MemberType,
    pub membership_expires_at: Option<DateTime<Utc>>,
    pub balance: Option<i64>,
    pub stamps: Option<i64>,
    pub referrer_id: Option<i64>,
    pub referral_code: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
