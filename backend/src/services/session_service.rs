use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    dto::UserBrief,
    entities::{
        access_sessions, devices, fingerprints,
        prelude::*,
        sea_orm_active_enums::{AccessEventType, DeviceKind, EventDecision, SessionStatus},
        users,
    },
    error::AppError,
    services::{
        checkout_code_service,
        event_service::{self, NewEvent},
    },
    state::AppState,
    util,
};

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanAction {
    Unlock,
    Deny,
}

/// The response a device (or the simulator) acts on after a fingerprint scan.
#[derive(Debug, Serialize)]
pub struct DeviceScanResponse {
    pub action: ScanAction,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserBrief>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_expires_at: Option<DateTimeWithTimeZone>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub enroll_hint: bool,
    /// Shown on the cabinet's LCD. The person types it on the website to record
    /// what they took, which is how a written record gets tied to this opening.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkout_code: Option<String>,
}

impl DeviceScanResponse {
    fn deny(reason: &str) -> Self {
        Self {
            action: ScanAction::Deny,
            reason: reason.to_owned(),
            user: None,
            session_expires_at: None,
            enroll_hint: false,
            checkout_code: None,
        }
    }

    fn deny_unregistered() -> Self {
        Self {
            action: ScanAction::Deny,
            reason: "unregistered".to_owned(),
            user: None,
            session_expires_at: None,
            enroll_hint: true,
            checkout_code: None,
        }
    }
}

/// Resolve a scanned finger token to its owning user, if registered.
async fn resolve_finger(
    state: &AppState,
    finger_token: &str,
) -> Result<Option<(fingerprints::Model, users::Model)>, AppError> {
    let Some(fp) = Fingerprints::find()
        .filter(fingerprints::Column::FingerToken.eq(finger_token))
        .one(&state.db)
        .await?
    else {
        return Ok(None);
    };
    let Some(user) = Users::find_by_id(fp.user_id).one(&state.db).await? else {
        return Ok(None);
    };
    Ok(Some((fp, user)))
}

/// Door scan: on a valid registered finger, open a timed access session and
/// unlock the relay. This session is the gate the box scan later checks.
pub async fn door_scan(
    state: &AppState,
    device: &devices::Model,
    finger_token: &str,
) -> Result<DeviceScanResponse, AppError> {
    let Some((_fp, user)) = resolve_finger(state, finger_token).await? else {
        event_service::emit(
            state,
            NewEvent {
                event_type: AccessEventType::DoorDeniedUnregistered,
                decision: EventDecision::Denied,
                device_id: Some(device.id),
                user_id: None,
                finger_token: Some(finger_token.to_owned()),
                access_session_id: None,
                enrollment_session_id: None,
                message: Some("unregistered finger at door".to_owned()),
            },
        )
        .await?;
        return Ok(DeviceScanResponse::deny_unregistered());
    };

    if !user.is_active {
        event_service::emit(
            state,
            NewEvent {
                event_type: AccessEventType::DoorDeniedInactive,
                decision: EventDecision::Denied,
                device_id: Some(device.id),
                user_id: Some(user.id),
                finger_token: Some(finger_token.to_owned()),
                access_session_id: None,
                enrollment_session_id: None,
                message: Some("deactivated account".to_owned()),
            },
        )
        .await?;
        return Ok(DeviceScanResponse::deny("account_inactive"));
    }

    let now = util::now();
    let expires_at: DateTimeWithTimeZone = (chrono::Utc::now()
        + chrono::Duration::from_std(state.cfg.session_ttl).unwrap_or_default())
    .into();

    // Refresh an existing active window rather than stacking sessions.
    let existing = AccessSessions::find()
        .filter(access_sessions::Column::UserId.eq(user.id))
        .filter(access_sessions::Column::Status.eq(SessionStatus::Active))
        .filter(access_sessions::Column::ExpiresAt.gt(now))
        .one(&state.db)
        .await?;

    let session = match existing {
        Some(session) => {
            let id = session.id;
            let mut active: access_sessions::ActiveModel = session.into();
            active.expires_at = Set(expires_at);
            active.door_device_id = Set(device.id);
            active.update(&state.db).await?;
            id
        }
        None => {
            let id = Uuid::new_v4();
            access_sessions::ActiveModel {
                id: Set(id),
                user_id: Set(user.id),
                door_device_id: Set(device.id),
                opened_at: Set(now),
                expires_at: Set(expires_at),
                status: Set(SessionStatus::Active),
                consumed_by_box_device_id: Set(None),
                closed_at: Set(None),
            }
            .insert(&state.db)
            .await?;
            id
        }
    };

    event_service::emit(
        state,
        NewEvent {
            event_type: AccessEventType::DoorGranted,
            decision: EventDecision::Granted,
            device_id: Some(device.id),
            user_id: Some(user.id),
            finger_token: Some(finger_token.to_owned()),
            access_session_id: Some(session),
            enrollment_session_id: None,
            message: None,
        },
    )
    .await?;

    Ok(DeviceScanResponse {
        action: ScanAction::Unlock,
        reason: "door_granted".to_owned(),
        user: Some(UserBrief::from(&user)),
        session_expires_at: Some(expires_at),
        enroll_hint: false,
        // Only the cabinet issues a checkout code: nothing has left a shelf yet.
        checkout_code: None,
    })
}

/// Box scan: open the solenoid only if this same user holds a live door
/// session. A registered user who never scanned the door (a tailgater) is
/// refused and logged.
pub async fn box_scan(
    state: &AppState,
    device: &devices::Model,
    finger_token: &str,
) -> Result<DeviceScanResponse, AppError> {
    let Some((_fp, user)) = resolve_finger(state, finger_token).await? else {
        event_service::emit(
            state,
            NewEvent {
                event_type: AccessEventType::BoxDeniedUnregistered,
                decision: EventDecision::Denied,
                device_id: Some(device.id),
                user_id: None,
                finger_token: Some(finger_token.to_owned()),
                access_session_id: None,
                enrollment_session_id: None,
                message: Some("unregistered finger at box".to_owned()),
            },
        )
        .await?;
        return Ok(DeviceScanResponse::deny("unregistered"));
    };

    let now = util::now();
    let sessions = AccessSessions::find()
        .filter(access_sessions::Column::UserId.eq(user.id))
        .filter(access_sessions::Column::Status.eq(SessionStatus::Active))
        .order_by_desc(access_sessions::Column::ExpiresAt)
        .all(&state.db)
        .await?;

    if let Some(active) = sessions.iter().find(|s| s.expires_at > now) {
        event_service::emit(
            state,
            NewEvent {
                event_type: AccessEventType::BoxGranted,
                decision: EventDecision::Granted,
                device_id: Some(device.id),
                user_id: Some(user.id),
                finger_token: Some(finger_token.to_owned()),
                access_session_id: Some(active.id),
                enrollment_session_id: None,
                message: None,
            },
        )
        .await?;
        // The code is a convenience, not a lock: failing to mint one must not
        // keep the cabinet shut after a valid scan.
        let checkout_code = match checkout_code_service::issue(
            &state.db,
            user.id,
            Some(device.id),
            Some(active.id),
        )
        .await
        {
            Ok(code) => Some(code.code),
            Err(err) => {
                tracing::warn!("could not issue a checkout code: {err}");
                None
            }
        };

        return Ok(DeviceScanResponse {
            action: ScanAction::Unlock,
            reason: "box_granted".to_owned(),
            user: Some(UserBrief::from(&user)),
            session_expires_at: Some(active.expires_at),
            enroll_hint: false,
            checkout_code,
        });
    }

    if sessions.is_empty() {
        // Tailgater: registered, but no door session was ever opened.
        event_service::emit(
            state,
            NewEvent {
                event_type: AccessEventType::BoxDeniedNoSession,
                decision: EventDecision::Denied,
                device_id: Some(device.id),
                user_id: Some(user.id),
                finger_token: Some(finger_token.to_owned()),
                access_session_id: None,
                enrollment_session_id: None,
                message: Some("no active door session (possible tailgating)".to_owned()),
            },
        )
        .await?;
        return Ok(DeviceScanResponse::deny("no_active_session"));
    }

    // Sessions exist but all have lapsed: lazily expire them and deny.
    for session in sessions {
        let mut active: access_sessions::ActiveModel = session.into();
        active.status = Set(SessionStatus::Expired);
        active.closed_at = Set(Some(now));
        active.update(&state.db).await?;
    }
    event_service::emit(
        state,
        NewEvent {
            event_type: AccessEventType::BoxDeniedSessionExpired,
            decision: EventDecision::Denied,
            device_id: Some(device.id),
            user_id: Some(user.id),
            finger_token: Some(finger_token.to_owned()),
            access_session_id: None,
            enrollment_session_id: None,
            message: Some("door session expired before reaching the box".to_owned()),
        },
    )
    .await?;
    Ok(DeviceScanResponse::deny("session_expired"))
}

/// Dispatch a scan to the door or box handler based on the device kind.
pub async fn scan(
    state: &AppState,
    device: &devices::Model,
    finger_token: &str,
) -> Result<DeviceScanResponse, AppError> {
    match device.kind {
        DeviceKind::Door => door_scan(state, device, finger_token).await,
        DeviceKind::Box => box_scan(state, device, finger_token).await,
    }
}
