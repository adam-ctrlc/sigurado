//! `SeaORM` Entity for the people who get an SMS when something happens.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "sms_recipients")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// The staff account that receives these alerts. The name and number come
    /// from that user, so a number is stored in exactly one place.
    #[sea_orm(unique)]
    pub user_id: Uuid,
    pub notify_cabinet_opened: bool,
    pub notify_access_denied: bool,
    pub notify_door_opened: bool,
    pub is_active: bool,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::sms_messages::Entity")]
    SmsMessages,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Users,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::sms_messages::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SmsMessages.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
