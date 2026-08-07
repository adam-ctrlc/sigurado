//! Clearing the data, without destroying any of it.
//!
//! The whole system rests on the promise that a reader event is never edited or
//! deleted. "Delete all data" would break that promise outright, so this hides
//! instead: one row records the moment, and every history view ignores anything
//! recorded at or before it. The rows stay exactly where they were, which means
//! the action is reversible and the promise still holds.
//!
//! The split that matters is between history and live decisions. History is what
//! people read: the audit log, the sheet, the flags, past checkouts. Live
//! decisions are what the readers act on: whether a door window is open, whether
//! a checkout code is good, whether a fingerprint is bound. Clearing the data
//! must never touch the second kind, or the cabinet would start refusing people
//! because somebody tidied up the records.

use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    entities::{
        access_events, access_sessions, checkouts, data_purges, enrollment_sessions, login_events,
        prelude::*, sms_messages,
    },
    error::AppError,
    util,
};

/// Typed exactly, or nothing happens. Checked on the server as well as in the
/// browser: a confirmation the client alone enforces is decoration.
pub const CONFIRMATION: &str = "DELETE ALL DATA";

/// How much history is in view, or how much a clearing would hide.
#[derive(Debug, Default, Clone, Copy, Serialize)]
pub struct Counts {
    pub events: u64,
    pub checkouts: u64,
    pub sessions: u64,
    pub logins: u64,
    pub messages: u64,
    pub enrollments: u64,
}

impl Counts {
    pub fn total(&self) -> u64 {
        self.events
            + self.checkouts
            + self.sessions
            + self.logins
            + self.messages
            + self.enrollments
    }
}

/// The instant everything older than is hidden, if the data has been cleared.
///
/// Read once per request and passed down, rather than queried per table.
pub async fn cutoff(db: &DatabaseConnection) -> Result<Option<DateTimeWithTimeZone>, AppError> {
    Ok(DataPurges::find()
        .filter(data_purges::Column::UndoneAt.is_null())
        .order_by_desc(data_purges::Column::PurgedAt)
        .one(db)
        .await?
        .map(|row| row.purged_at))
}

/// The clearing currently in force, for the page that offers to undo it.
pub async fn active(db: &DatabaseConnection) -> Result<Option<data_purges::Model>, AppError> {
    Ok(DataPurges::find()
        .filter(data_purges::Column::UndoneAt.is_null())
        .order_by_desc(data_purges::Column::PurgedAt)
        .one(db)
        .await?)
}

/// Every clearing ever made, newest first, including undone ones.
pub async fn history(db: &DatabaseConnection) -> Result<Vec<data_purges::Model>, AppError> {
    Ok(DataPurges::find()
        .order_by_desc(data_purges::Column::PurgedAt)
        .all(db)
        .await?)
}

/// What is in view right now, which is what a clearing would hide.
pub async fn visible_counts(db: &DatabaseConnection) -> Result<Counts, AppError> {
    let since = cutoff(db).await?;

    macro_rules! count {
        ($entity:ty, $column:expr) => {{
            let mut query = <$entity>::find();
            if let Some(since) = since {
                query = query.filter($column.gt(since));
            }
            query.count(db).await?
        }};
    }

    Ok(Counts {
        events: count!(AccessEvents, access_events::Column::CreatedAt),
        checkouts: count!(Checkouts, checkouts::Column::CreatedAt),
        sessions: count!(AccessSessions, access_sessions::Column::OpenedAt),
        logins: count!(LoginEvents, login_events::Column::CreatedAt),
        messages: count!(SmsMessages, sms_messages::Column::CreatedAt),
        enrollments: count!(EnrollmentSessions, enrollment_sessions::Column::CreatedAt),
    })
}

/// Hides everything recorded up to now.
///
/// The caller has already checked the confirmation phrase and that the person
/// is an administrator.
pub async fn clear(
    db: &DatabaseConnection,
    actor: Option<Uuid>,
    actor_name: Option<String>,
    note: Option<String>,
) -> Result<data_purges::Model, AppError> {
    let counts = visible_counts(db).await?;
    let at = util::now();

    let row = data_purges::ActiveModel {
        id: Set(Uuid::new_v4()),
        purged_at: Set(at),
        performed_by: Set(actor),
        performed_by_name: Set(actor_name),
        note: Set(note.filter(|n| !n.trim().is_empty())),
        hidden_events: Set(counts.events as i32),
        hidden_checkouts: Set(counts.checkouts as i32),
        hidden_sessions: Set(counts.sessions as i32),
        hidden_logins: Set(counts.logins as i32),
        hidden_messages: Set(counts.messages as i32),
        hidden_enrollments: Set(counts.enrollments as i32),
        undone_at: Set(None),
        undone_by: Set(None),
    }
    .insert(db)
    .await?;

    Ok(row)
}

/// Puts everything back. Only the clearing currently in force can be undone.
pub async fn undo(
    db: &DatabaseConnection,
    actor: Option<Uuid>,
) -> Result<data_purges::Model, AppError> {
    let row = active(db)
        .await?
        .ok_or(AppError::NotFound("cleared data to restore"))?;

    let mut update: data_purges::ActiveModel = row.into();
    update.undone_at = Set(Some(util::now()));
    update.undone_by = Set(actor);
    Ok(update.update(db).await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_phrase_is_exact() {
        assert_eq!(CONFIRMATION, "DELETE ALL DATA");
        // Near misses have to fail, or the confirmation means nothing.
        assert_ne!(CONFIRMATION, "delete all data");
        assert_ne!(CONFIRMATION, "DELETE ALL DATA ");
    }

    #[test]
    fn counts_add_up() {
        let counts = Counts {
            events: 33,
            checkouts: 3,
            sessions: 5,
            logins: 9,
            messages: 5,
            enrollments: 2,
        };
        assert_eq!(counts.total(), 57);
        assert_eq!(Counts::default().total(), 0);
    }
}
