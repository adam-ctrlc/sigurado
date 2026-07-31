use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    entities::{
        access_events,
        prelude::{Devices, Users},
        sea_orm_active_enums::{AccessEventType, EventDecision},
    },
    error::AppError,
    services::sms_service::{self, AlertKind},
    state::AppState,
    util,
};

/// Input for recording an audit event. Names are resolved by the service.
pub struct NewEvent {
    pub event_type: AccessEventType,
    pub decision: EventDecision,
    pub device_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub finger_token: Option<String>,
    pub access_session_id: Option<Uuid>,
    pub enrollment_session_id: Option<Uuid>,
    pub message: Option<String>,
}

/// The shape sent to the dashboard, both on the REST backfill and the live SSE
/// stream. Enriched with resolved device/user display names.
#[derive(Debug, Clone, Serialize)]
pub struct EventEnvelope {
    pub id: Uuid,
    pub event_type: AccessEventType,
    pub decision: EventDecision,
    pub device_id: Option<Uuid>,
    pub device_name: Option<String>,
    pub user_id: Option<Uuid>,
    pub user_name: Option<String>,
    pub finger_token: Option<String>,
    /// Who that finger turned out to belong to. An event recorded before the
    /// finger was bound has no `user_id`, but the token is enough to name the
    /// person once enrollment finishes. Derived at read time: the stored event
    /// is never rewritten.
    pub finger_owner_id: Option<Uuid>,
    pub finger_owner_name: Option<String>,
    pub access_session_id: Option<Uuid>,
    pub message: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

/// Persist an audit event, then broadcast it to any live dashboard subscribers.
pub async fn emit(state: &AppState, event: NewEvent) -> Result<EventEnvelope, AppError> {
    let id = Uuid::new_v4();
    let created_at = util::now();

    access_events::ActiveModel {
        id: Set(id),
        event_type: Set(event.event_type),
        decision: Set(event.decision),
        device_id: Set(event.device_id),
        user_id: Set(event.user_id),
        finger_token: Set(event.finger_token.clone()),
        access_session_id: Set(event.access_session_id),
        enrollment_session_id: Set(event.enrollment_session_id),
        message: Set(event.message.clone()),
        created_at: Set(created_at),
    }
    .insert(&state.db)
    .await?;

    let device_name = match event.device_id {
        Some(id) => Devices::find_by_id(id)
            .one(&state.db)
            .await?
            .map(|d| d.name),
        None => None,
    };
    let user_name = match event.user_id {
        Some(id) => Users::find_by_id(id)
            .one(&state.db)
            .await?
            .map(|u| u.display_name()),
        None => None,
    };

    let envelope = EventEnvelope {
        id,
        event_type: event.event_type,
        decision: event.decision,
        device_id: event.device_id,
        device_name,
        user_id: event.user_id,
        user_name,
        finger_token: event.finger_token,
        // Nothing to resolve yet at emit time; a later read fills this in.
        finger_owner_id: None,
        finger_owner_name: None,
        access_session_id: event.access_session_id,
        message: event.message,
        created_at,
    };

    // A send error only means no dashboard is currently subscribed; not fatal.
    let _ = state.events.send(envelope.clone());

    queue_alert(state, &envelope).await;

    Ok(envelope)
}

/// Which events are worth an SMS, and what it should say. Every audit event
/// passes through here, so subscribing a number is the only thing an admin has
/// to do to start receiving alerts.
fn alert_for(envelope: &EventEnvelope) -> Option<(AlertKind, String)> {
    let who = envelope.user_name.as_deref().unwrap_or("an unknown finger");
    let where_ = envelope.device_name.as_deref().unwrap_or("a reader");
    let at = envelope.created_at.format("%b %-d, %-I:%M %p");

    match envelope.event_type {
        AccessEventType::BoxGranted => Some((
            AlertKind::CabinetOpened,
            format!("Sigurado: cabinet opened by {who} at {where_}, {at}."),
        )),
        AccessEventType::DoorGranted => Some((
            AlertKind::DoorOpened,
            format!("Sigurado: door opened by {who} at {where_}, {at}."),
        )),
        AccessEventType::BoxDeniedNoSession => Some((
            AlertKind::AccessDenied,
            format!("Sigurado: cabinet REFUSED for {who} at {where_} (no door scan), {at}."),
        )),
        _ if envelope.decision == EventDecision::Denied => Some((
            AlertKind::AccessDenied,
            format!("Sigurado: access refused for {who} at {where_}, {at}."),
        )),
        _ => None,
    }
}

/// Queueing an alert must never fail the scan that caused it: the lock has
/// already decided by this point.
async fn queue_alert(state: &AppState, envelope: &EventEnvelope) {
    let Some((kind, body)) = alert_for(envelope) else {
        return;
    };
    match sms_service::queue_for_alert(&state.db, kind, &body, Some(envelope.id)).await {
        Ok(0) => {}
        Ok(n) => tracing::debug!("queued {n} SMS alert(s) for {:?}", envelope.event_type),
        Err(err) => tracing::warn!("could not queue an SMS alert: {err}"),
    }
}
