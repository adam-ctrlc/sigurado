//! `SeaORM` Entity for the one-time code the cabinet shows when it opens.
//!
//! Recording a checkout needs one of these, so a written record always
//! corresponds to a real opening rather than to a browser session.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "checkout_codes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub code: String,
    pub user_id: Uuid,
    pub device_id: Option<Uuid>,
    pub access_session_id: Option<Uuid>,
    pub expires_at: DateTimeWithTimeZone,
    /// Set the moment it is spent, so a code cannot cover two checkouts.
    pub used_at: Option<DateTimeWithTimeZone>,
    pub checkout_id: Option<Uuid>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
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

impl ActiveModelBehavior for ActiveModel {}
