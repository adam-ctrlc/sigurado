//! `SeaORM` Entity for uptime tracking: one row per continuous stretch of
//! contact with a reader node.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "device_connections")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub device_id: Uuid,
    pub connected_at: DateTimeWithTimeZone,
    pub last_seen_at: DateTimeWithTimeZone,
    pub heartbeats: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::devices::Entity",
        from = "Column::DeviceId",
        to = "super::devices::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Devices,
}

impl Related<super::devices::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Devices.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
