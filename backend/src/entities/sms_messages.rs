//! `SeaORM` Entity for the outbound SMS queue.
//!
//! The server never talks to the network itself: it writes a row here and the
//! GSM-equipped reader node drains the queue and reports back.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "sms_messages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub recipient_id: Option<Uuid>,
    /// Snapshot of the number at queue time, so editing a recipient later does
    /// not rewrite what was actually sent.
    pub phone_number: String,
    pub body: String,
    pub status: super::sea_orm_active_enums::SmsStatus,
    pub attempts: i32,
    pub error: Option<String>,
    pub access_event_id: Option<Uuid>,
    pub sent_by_device_id: Option<Uuid>,
    pub created_at: DateTimeWithTimeZone,
    pub sent_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::sms_recipients::Entity",
        from = "Column::RecipientId",
        to = "super::sms_recipients::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    SmsRecipients,
    #[sea_orm(
        belongs_to = "super::access_events::Entity",
        from = "Column::AccessEventId",
        to = "super::access_events::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    AccessEvents,
}

impl Related<super::sms_recipients::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SmsRecipients.def()
    }
}

impl Related<super::access_events::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AccessEvents.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
