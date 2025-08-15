use crate::models::MemberType;
use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::entity::prelude::*;

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
