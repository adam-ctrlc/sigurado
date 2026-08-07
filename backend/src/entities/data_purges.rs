//! `SeaORM` Entity for a moment somebody cleared the data.
//!
//! One row per clearing. Nothing is removed from any other table: the history
//! views simply ignore rows older than `purged_at`, which is what makes the
//! action reversible and keeps a reader event from ever being destroyed.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "data_purges")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// Everything recorded at or before this instant is hidden.
    pub purged_at: DateTimeWithTimeZone,
    pub performed_by: Option<Uuid>,
    /// Kept as text so the history still names them if the account is edited.
    pub performed_by_name: Option<String>,
    pub note: Option<String>,
    pub hidden_events: i32,
    pub hidden_checkouts: i32,
    pub hidden_sessions: i32,
    pub hidden_logins: i32,
    pub hidden_messages: i32,
    pub hidden_enrollments: i32,
    /// Set when the clearing is undone, which puts every row back in view.
    pub undone_at: Option<DateTimeWithTimeZone>,
    pub undone_by: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
