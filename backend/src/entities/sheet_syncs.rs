//! `SeaORM` Entity for how far the spreadsheet export has got.
//!
//! One row per tab. The cursor is what makes the export safe to restart: rows
//! already sent are never sent again.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "sheet_syncs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub tab: String,
    /// Opaque "timestamp|id" of the last row sent. Null before the first sync.
    pub last_key: Option<String>,
    pub rows_sent: i32,
    pub last_synced_at: Option<DateTimeWithTimeZone>,
    pub last_error: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
