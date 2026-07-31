use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use uuid::Uuid;

use crate::{
    entities::{
        devices, enrollment_sessions, fingerprints,
        prelude::*,
        sea_orm_active_enums::{AccessEventType, EnrollmentStatus, EventDecision},
        users,
    },
    error::AppError,
    services::event_service::{self, NewEvent},
    state::AppState,
    util,
};

const CODE_LEN: usize = 6;

fn is_expired(session: &enrollment_sessions::Model) -> bool {
    session.code_expires_at < util::now()
}

async fn cancel_open_sessions(state: &AppState, device_id: Uuid) -> Result<(), AppError> {
    let open = EnrollmentSessions::find()
        .filter(enrollment_sessions::Column::DeviceId.eq(device_id))
        .filter(enrollment_sessions::Column::Status.is_in([
            EnrollmentStatus::PendingCode,
            EnrollmentStatus::AwaitingScan,
        ]))
        .all(&state.db)
        .await?;
    for session in open {
        let mut active: enrollment_sessions::ActiveModel = session.into();
        active.status = Set(EnrollmentStatus::Cancelled);
        active.update(&state.db).await?;
    }
    Ok(())
}

/// Device push-button pressed: issue an enrollment code to show on the LCD.
/// The user is still unknown; they bind by typing this code on the website.
pub async fn request_code(
    state: &AppState,
    device: &devices::Model,
    finger_token: Option<String>,
) -> Result<enrollment_sessions::Model, AppError> {
    cancel_open_sessions(state, device.id).await?;

    let now = util::now();
    let expires_at = (chrono::Utc::now()
        + chrono::Duration::from_std(state.cfg.enroll_code_ttl).unwrap_or_default())
    .into();

    let session = enrollment_sessions::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(None),
        device_id: Set(device.id),
        code: Set(util::random_code(CODE_LEN)),
        status: Set(EnrollmentStatus::PendingCode),
        pending_finger_token: Set(finger_token),
        created_at: Set(now),
        code_expires_at: Set(expires_at),
        bound_at: Set(None),
    }
    .insert(&state.db)
    .await?;

    event_service::emit(
        state,
        NewEvent {
            event_type: AccessEventType::EnrollCodeIssued,
            decision: EventDecision::Info,
            device_id: Some(device.id),
            user_id: None,
            finger_token: session.pending_finger_token.clone(),
            access_session_id: None,
            enrollment_session_id: Some(session.id),
            message: Some("enrollment code issued".to_owned()),
        },
    )
    .await?;

    Ok(session)
}

/// Website step: the logged-in user types the LCD code to claim the pending
/// enrollment, moving it to `awaiting_scan` and binding it to their account.
pub async fn verify_code(
    state: &AppState,
    user_id: Uuid,
    code: &str,
) -> Result<enrollment_sessions::Model, AppError> {
    let session = EnrollmentSessions::find()
        .filter(enrollment_sessions::Column::Code.eq(code))
        .filter(enrollment_sessions::Column::Status.eq(EnrollmentStatus::PendingCode))
        .order_by_desc(enrollment_sessions::Column::CreatedAt)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("enrollment code"))?;

    if is_expired(&session) {
        let mut active: enrollment_sessions::ActiveModel = session.into();
        active.status = Set(EnrollmentStatus::Expired);
        active.update(&state.db).await?;
        return Err(AppError::Validation(
            "enrollment code has expired".to_owned(),
        ));
    }

    let mut active: enrollment_sessions::ActiveModel = session.into();
    active.status = Set(EnrollmentStatus::AwaitingScan);
    active.user_id = Set(Some(user_id));
    let session = active.update(&state.db).await?;
    Ok(session)
}

pub struct BindOutcome {
    pub bound: bool,
    pub user: Option<users::Model>,
    pub message: String,
}

/// Device step: the next scan after the website confirmed the code binds the
/// scanned finger to the claimed account.
pub async fn bind(
    state: &AppState,
    device: &devices::Model,
    finger_token: &str,
) -> Result<BindOutcome, AppError> {
    let session = EnrollmentSessions::find()
        .filter(enrollment_sessions::Column::DeviceId.eq(device.id))
        .filter(enrollment_sessions::Column::Status.eq(EnrollmentStatus::AwaitingScan))
        .order_by_desc(enrollment_sessions::Column::CreatedAt)
        .one(&state.db)
        .await?;

    let Some(session) = session else {
        return Ok(BindOutcome {
            bound: false,
            user: None,
            message: "no enrollment awaiting a scan; press the enroll button first".to_owned(),
        });
    };

    if is_expired(&session) {
        let session_id = session.id;
        let mut active: enrollment_sessions::ActiveModel = session.into();
        active.status = Set(EnrollmentStatus::Expired);
        active.update(&state.db).await?;
        emit_failed(
            state,
            device.id,
            Some(session_id),
            finger_token,
            "enrollment expired",
        )
        .await?;
        return Ok(BindOutcome {
            bound: false,
            user: None,
            message: "enrollment expired; start again".to_owned(),
        });
    }

    let Some(user_id) = session.user_id else {
        emit_failed(
            state,
            device.id,
            Some(session.id),
            finger_token,
            "no user on session",
        )
        .await?;
        return Ok(BindOutcome {
            bound: false,
            user: None,
            message: "enrollment is missing its user; start again".to_owned(),
        });
    };

    let Some(user) = Users::find_by_id(user_id).one(&state.db).await? else {
        return Ok(BindOutcome {
            bound: false,
            user: None,
            message: "the enrolling account no longer exists".to_owned(),
        });
    };

    // Reject a finger already registered to someone.
    let already = Fingerprints::find()
        .filter(fingerprints::Column::FingerToken.eq(finger_token))
        .one(&state.db)
        .await?;
    if already.is_some() {
        emit_failed(
            state,
            device.id,
            Some(session.id),
            finger_token,
            "finger already registered",
        )
        .await?;
        return Ok(BindOutcome {
            bound: false,
            user: None,
            message: "this fingerprint is already registered".to_owned(),
        });
    }

    fingerprints::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        finger_token: Set(finger_token.to_owned()),
        label: Set(None),
        enrolled_via_device_id: Set(Some(device.id)),
        created_at: Set(util::now()),
    }
    .insert(&state.db)
    .await?;

    let session_id = session.id;
    let mut active: enrollment_sessions::ActiveModel = session.into();
    active.status = Set(EnrollmentStatus::Bound);
    active.bound_at = Set(Some(util::now()));
    active.update(&state.db).await?;

    event_service::emit(
        state,
        NewEvent {
            event_type: AccessEventType::EnrollBound,
            decision: EventDecision::Info,
            device_id: Some(device.id),
            user_id: Some(user_id),
            finger_token: Some(finger_token.to_owned()),
            access_session_id: None,
            enrollment_session_id: Some(session_id),
            message: Some(format!("fingerprint bound to {}", user.display_name())),
        },
    )
    .await?;

    let message = format!("fingerprint bound to {}", user.display_name());
    Ok(BindOutcome {
        bound: true,
        user: Some(user),
        message,
    })
}

async fn emit_failed(
    state: &AppState,
    device_id: Uuid,
    enrollment_session_id: Option<Uuid>,
    finger_token: &str,
    message: &str,
) -> Result<(), AppError> {
    event_service::emit(
        state,
        NewEvent {
            event_type: AccessEventType::EnrollFailed,
            decision: EventDecision::Denied,
            device_id: Some(device_id),
            user_id: None,
            finger_token: Some(finger_token.to_owned()),
            access_session_id: None,
            enrollment_session_id,
            message: Some(message.to_owned()),
        },
    )
    .await?;
    Ok(())
}
