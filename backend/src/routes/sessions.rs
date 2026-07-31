use std::collections::HashMap;

use axum::{Json, Router, extract::State, routing::get};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    auth::extractor::FacultyOrAdmin,
    entities::{access_sessions, prelude::*, sea_orm_active_enums::SessionStatus},
    error::AppResult,
    state::AppState,
    util,
};

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_name: Option<String>,
    pub door_device_id: Uuid,
    pub door_device_name: Option<String>,
    pub opened_at: DateTimeWithTimeZone,
    pub expires_at: DateTimeWithTimeZone,
    pub status: SessionStatus,
    pub is_live: bool,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sessions/active", get(active))
        .route("/sessions", get(history))
}

async fn enrich(
    state: &AppState,
    sessions: Vec<access_sessions::Model>,
) -> AppResult<Vec<SessionResponse>> {
    let now = util::now();

    let user_ids: Vec<Uuid> = sessions.iter().map(|s| s.user_id).collect();
    let device_ids: Vec<Uuid> = sessions.iter().map(|s| s.door_device_id).collect();

    let user_names: HashMap<Uuid, String> = Users::find()
        .filter(crate::entities::users::Column::Id.is_in(user_ids))
        .all(&state.db)
        .await?
        .into_iter()
        .map(|u| (u.id, u.display_name()))
        .collect();
    let device_names: HashMap<Uuid, String> = Devices::find()
        .filter(crate::entities::devices::Column::Id.is_in(device_ids))
        .all(&state.db)
        .await?
        .into_iter()
        .map(|d| (d.id, d.name))
        .collect();

    Ok(sessions
        .into_iter()
        .map(|s| SessionResponse {
            is_live: s.status == SessionStatus::Active && s.expires_at > now,
            user_name: user_names.get(&s.user_id).cloned(),
            door_device_name: device_names.get(&s.door_device_id).cloned(),
            id: s.id,
            user_id: s.user_id,
            door_device_id: s.door_device_id,
            opened_at: s.opened_at,
            expires_at: s.expires_at,
            status: s.status,
        })
        .collect())
}

async fn active(
    State(state): State<AppState>,
    _who: FacultyOrAdmin,
) -> AppResult<Json<Vec<SessionResponse>>> {
    let now = util::now();
    let sessions = AccessSessions::find()
        .filter(access_sessions::Column::Status.eq(SessionStatus::Active))
        .filter(access_sessions::Column::ExpiresAt.gt(now))
        .order_by_desc(access_sessions::Column::OpenedAt)
        .all(&state.db)
        .await?;
    Ok(Json(enrich(&state, sessions).await?))
}

async fn history(
    State(state): State<AppState>,
    _who: FacultyOrAdmin,
) -> AppResult<Json<Vec<SessionResponse>>> {
    let sessions = AccessSessions::find()
        .order_by_desc(access_sessions::Column::OpenedAt)
        .limit(200)
        .all(&state.db)
        .await?;
    Ok(Json(enrich(&state, sessions).await?))
}
