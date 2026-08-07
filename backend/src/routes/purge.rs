//! Clearing the records, and putting them back.
//!
//! Administrator only, and the confirmation phrase is checked here as well as in
//! the browser. A confirmation the client alone enforces stops nobody.

use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::extractor::AdminUser,
    entities::{
        prelude::Users,
        sea_orm_active_enums::{AccessEventType, EventDecision},
    },
    error::{AppError, AppResult},
    services::{
        event_service::{self, NewEvent},
        purge_service::{self, Counts},
    },
    state::AppState,
};
use sea_orm::EntityTrait;

#[derive(Debug, Deserialize)]
pub struct ClearRequest {
    /// Must equal the phrase exactly, capitals and all.
    pub confirm: String,
    /// Optional line on why, kept with the record of the clearing.
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PurgeRecord {
    pub id: Uuid,
    pub purged_at: DateTimeWithTimeZone,
    pub performed_by_name: Option<String>,
    pub note: Option<String>,
    pub hidden: Counts,
    pub undone_at: Option<DateTimeWithTimeZone>,
}

#[derive(Debug, Serialize)]
pub struct PurgeStatus {
    /// The phrase the person has to type, so the page and the server agree.
    pub confirmation: String,
    /// What is in view now, which is what clearing would hide.
    pub visible: Counts,
    pub visible_total: u64,
    /// Set while a clearing is in force, and the thing undo would reverse.
    pub active: Option<PurgeRecord>,
    pub history: Vec<PurgeRecord>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/purge", get(status).post(clear))
        .route("/admin/purge/undo", post(undo))
}

fn to_record(row: crate::entities::data_purges::Model) -> PurgeRecord {
    PurgeRecord {
        id: row.id,
        purged_at: row.purged_at,
        performed_by_name: row.performed_by_name,
        note: row.note,
        hidden: Counts {
            events: row.hidden_events.max(0) as u64,
            checkouts: row.hidden_checkouts.max(0) as u64,
            sessions: row.hidden_sessions.max(0) as u64,
            logins: row.hidden_logins.max(0) as u64,
            messages: row.hidden_messages.max(0) as u64,
            enrollments: row.hidden_enrollments.max(0) as u64,
        },
        undone_at: row.undone_at,
    }
}

async fn status(State(state): State<AppState>, _who: AdminUser) -> AppResult<Json<PurgeStatus>> {
    let visible = purge_service::visible_counts(&state.db).await?;
    Ok(Json(PurgeStatus {
        confirmation: purge_service::CONFIRMATION.to_owned(),
        visible,
        visible_total: visible.total(),
        active: purge_service::active(&state.db).await?.map(to_record),
        history: purge_service::history(&state.db)
            .await?
            .into_iter()
            .map(to_record)
            .collect(),
    }))
}

async fn clear(
    State(state): State<AppState>,
    who: AdminUser,
    Json(body): Json<ClearRequest>,
) -> AppResult<Json<PurgeRecord>> {
    if body.confirm != purge_service::CONFIRMATION {
        return Err(AppError::Validation(format!(
            "type {} exactly to confirm",
            purge_service::CONFIRMATION
        )));
    }

    let name = Users::find_by_id(who.0.user_id)
        .one(&state.db)
        .await?
        .map(|u| u.display_name());

    let row = purge_service::clear(&state.db, Some(who.0.user_id), name.clone(), body.note).await?;
    let record = to_record(row);

    // Recorded after the clearing, so it survives it. The trail should always be
    // able to say who emptied it and when, even when it looks empty.
    let hidden = record.hidden.total();
    let _ = event_service::emit(
        &state,
        NewEvent {
            event_type: AccessEventType::DataCleared,
            decision: EventDecision::Info,
            device_id: None,
            user_id: Some(who.0.user_id),
            finger_token: None,
            access_session_id: None,
            enrollment_session_id: None,
            message: Some(format!(
                "{} cleared the records: {hidden} rows hidden, reversible",
                name.unwrap_or_else(|| "an administrator".to_owned())
            )),
        },
    )
    .await;

    Ok(Json(record))
}

async fn undo(State(state): State<AppState>, who: AdminUser) -> AppResult<Json<PurgeRecord>> {
    let row = purge_service::undo(&state.db, Some(who.0.user_id)).await?;

    let name = Users::find_by_id(who.0.user_id)
        .one(&state.db)
        .await?
        .map(|u| u.display_name())
        .unwrap_or_else(|| "an administrator".to_owned());

    let _ = event_service::emit(
        &state,
        NewEvent {
            event_type: AccessEventType::DataRestored,
            decision: EventDecision::Info,
            device_id: None,
            user_id: Some(who.0.user_id),
            finger_token: None,
            access_session_id: None,
            enrollment_session_id: None,
            message: Some(format!("{name} put the cleared records back in view")),
        },
    )
    .await;

    Ok(Json(to_record(row)))
}
