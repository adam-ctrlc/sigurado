use std::collections::HashMap;
use std::convert::Infallible;

use axum::{
    Json, Router,
    extract::{Query, State},
    response::sse::{Event, KeepAlive, Sse},
    routing::get,
};
use sea_orm::{ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde::Deserialize;
use tokio_stream::{Stream, StreamExt, wrappers::BroadcastStream};
use uuid::Uuid;

use crate::{
    auth::extractor::FacultyOrAdmin,
    entities::{
        access_events,
        prelude::*,
        sea_orm_active_enums::{AccessEventType, EventDecision},
    },
    error::AppResult,
    pagination::{Page, page_of, per_page_of, search_term},
    services::event_service::EventEnvelope,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct LogQuery {
    pub event_type: Option<AccessEventType>,
    pub decision: Option<EventDecision>,
    pub device_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    /// Free text over the message, the finger token, and the names of the person
    /// and reader involved.
    pub q: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/logs", get(list_logs))
        .route("/logs/stream", get(stream_logs))
}

async fn list_logs(
    State(state): State<AppState>,
    _who: FacultyOrAdmin,
    Query(params): Query<LogQuery>,
) -> AppResult<Json<Page<EventEnvelope>>> {
    let page = page_of(params.page);
    let per_page = per_page_of(params.per_page);

    let mut query = AccessEvents::find();
    if let Some(event_type) = params.event_type {
        query = query.filter(access_events::Column::EventType.eq(event_type));
    }
    if let Some(decision) = params.decision {
        query = query.filter(access_events::Column::Decision.eq(decision));
    }
    if let Some(device_id) = params.device_id {
        query = query.filter(access_events::Column::DeviceId.eq(device_id));
    }
    if let Some(user_id) = params.user_id {
        query = query.filter(access_events::Column::UserId.eq(user_id));
    }

    if let Some(term) = search_term(params.q) {
        // A name lives on the user or device row, not on the event, so resolve
        // the matching ids first and match the event against those.
        let user_hits: Vec<Uuid> = Users::find()
            .filter(
                Condition::any()
                    .add(crate::entities::users::Column::FirstName.contains(&term))
                    .add(crate::entities::users::Column::MiddleName.contains(&term))
                    .add(crate::entities::users::Column::LastName.contains(&term))
                    .add(crate::entities::users::Column::Username.contains(&term)),
            )
            .all(&state.db)
            .await?
            .into_iter()
            .map(|u| u.id)
            .collect();
        let device_hits: Vec<Uuid> = Devices::find()
            .filter(crate::entities::devices::Column::Name.contains(&term))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|d| d.id)
            .collect();
        // A finger the search names may since have been claimed, so include
        // events that only carry the token.
        let finger_hits: Vec<String> = Fingerprints::find()
            .filter(crate::entities::fingerprints::Column::UserId.is_in(user_hits.clone()))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|f| f.finger_token)
            .collect();

        let mut any = Condition::any()
            .add(access_events::Column::Message.contains(&term))
            .add(access_events::Column::FingerToken.contains(&term));
        if !user_hits.is_empty() {
            any = any.add(access_events::Column::UserId.is_in(user_hits));
        }
        if !device_hits.is_empty() {
            any = any.add(access_events::Column::DeviceId.is_in(device_hits));
        }
        if !finger_hits.is_empty() {
            any = any.add(access_events::Column::FingerToken.is_in(finger_hits));
        }
        query = query.filter(any);
    }

    let paginator = query
        .order_by_desc(access_events::Column::CreatedAt)
        .paginate(&state.db, per_page);
    let total = paginator.num_items().await?;
    let events = paginator.fetch_page(page - 1).await?;

    let device_ids: Vec<Uuid> = events.iter().filter_map(|e| e.device_id).collect();

    // An event recorded before its finger was bound carries no user_id, but the
    // token still points at whoever later claimed it. Resolve those owners so
    // the trail can name the person retroactively, without rewriting history.
    let orphan_tokens: Vec<String> = events
        .iter()
        .filter(|e| e.user_id.is_none())
        .filter_map(|e| e.finger_token.clone())
        .collect();
    let finger_owners: HashMap<String, Uuid> = if orphan_tokens.is_empty() {
        HashMap::new()
    } else {
        Fingerprints::find()
            .filter(crate::entities::fingerprints::Column::FingerToken.is_in(orphan_tokens))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|f| (f.finger_token, f.user_id))
            .collect()
    };

    let mut user_ids: Vec<Uuid> = events.iter().filter_map(|e| e.user_id).collect();
    user_ids.extend(finger_owners.values().copied());

    let device_names: HashMap<Uuid, String> = Devices::find()
        .filter(crate::entities::devices::Column::Id.is_in(device_ids))
        .all(&state.db)
        .await?
        .into_iter()
        .map(|d| (d.id, d.name))
        .collect();
    let user_names: HashMap<Uuid, String> = Users::find()
        .filter(crate::entities::users::Column::Id.is_in(user_ids))
        .all(&state.db)
        .await?
        .into_iter()
        .map(|u| (u.id, u.display_name()))
        .collect();

    let envelopes: Vec<EventEnvelope> = events
        .into_iter()
        .map(|e| {
            let owner_id = match (e.user_id, e.finger_token.as_ref()) {
                (None, Some(token)) => finger_owners.get(token).copied(),
                _ => None,
            };
            EventEnvelope {
                device_name: e.device_id.and_then(|id| device_names.get(&id).cloned()),
                user_name: e.user_id.and_then(|id| user_names.get(&id).cloned()),
                finger_owner_name: owner_id.and_then(|id| user_names.get(&id).cloned()),
                finger_owner_id: owner_id,
                id: e.id,
                event_type: e.event_type,
                decision: e.decision,
                device_id: e.device_id,
                user_id: e.user_id,
                finger_token: e.finger_token,
                access_session_id: e.access_session_id,
                message: e.message,
                created_at: e.created_at,
            }
        })
        .collect();

    Ok(Json(Page::new(envelopes, total, page, per_page)))
}

async fn stream_logs(
    State(state): State<AppState>,
    _who: FacultyOrAdmin,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = BroadcastStream::new(state.events.subscribe()).filter_map(|item| match item {
        Ok(envelope) => Some(Ok(Event::default()
            .event("access_event")
            .json_data(envelope)
            .unwrap_or_else(|_| Event::default().comment("serialization error")))),
        Err(_) => None,
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
