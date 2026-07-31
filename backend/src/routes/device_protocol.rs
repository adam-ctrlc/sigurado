use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};

use uuid::Uuid;

use crate::{
    auth::device_auth::DeviceCtx,
    dto::UserBrief,
    error::{AppError, AppResult},
    services::{
        enrollment_service,
        session_service::{self, DeviceScanResponse},
        sms_service,
    },
    state::AppState,
};

/// How many queued messages a node may pull at once. A SIM800L sends one at a
/// time and slowly, so a small batch keeps the round trip short.
const SMS_BATCH: u64 = 5;

#[derive(Debug, Deserialize)]
pub struct ScanRequest {
    pub finger_token: String,
}

#[derive(Debug, Deserialize)]
pub struct EnrollRequestCodeRequest {
    #[serde(default)]
    pub finger_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EnrollCodeResponse {
    pub code: String,
    pub code_expires_at: DateTimeWithTimeZone,
}

#[derive(Debug, Serialize)]
pub struct EnrollBindResponse {
    pub bound: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserBrief>,
}

#[derive(Debug, Serialize)]
pub struct HeartbeatResponse {
    pub ok: bool,
    /// Tells the node whether it should bother polling the SMS queue.
    pub sms_capable: bool,
    pub pending_sms: u64,
}

/// One queued message, trimmed to what the modem needs.
#[derive(Debug, Serialize)]
pub struct OutboundSms {
    pub id: Uuid,
    pub phone_number: String,
    pub body: String,
    pub attempts: i32,
}

#[derive(Debug, Deserialize)]
pub struct SmsResultRequest {
    pub sent: bool,
    #[serde(default)]
    pub error: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/device/scan", post(scan))
        .route("/device/enroll/request-code", post(request_code))
        .route("/device/enroll/bind", post(bind))
        .route("/device/heartbeat", post(heartbeat))
        .route("/device/sms/pending", get(pending_sms))
        .route("/device/sms/{id}/result", post(sms_result))
}

async fn scan(
    State(state): State<AppState>,
    ctx: DeviceCtx,
    Json(body): Json<ScanRequest>,
) -> AppResult<Json<DeviceScanResponse>> {
    let token = body.finger_token.trim();
    if token.is_empty() {
        return Err(AppError::BadRequest("finger_token is required".to_owned()));
    }
    let response = session_service::scan(&state, &ctx.device, token).await?;
    Ok(Json(response))
}

async fn request_code(
    State(state): State<AppState>,
    ctx: DeviceCtx,
    Json(body): Json<EnrollRequestCodeRequest>,
) -> AppResult<Json<EnrollCodeResponse>> {
    let session = enrollment_service::request_code(&state, &ctx.device, body.finger_token).await?;
    Ok(Json(EnrollCodeResponse {
        code: session.code,
        code_expires_at: session.code_expires_at,
    }))
}

async fn bind(
    State(state): State<AppState>,
    ctx: DeviceCtx,
    Json(body): Json<ScanRequest>,
) -> AppResult<Json<EnrollBindResponse>> {
    let token = body.finger_token.trim();
    if token.is_empty() {
        return Err(AppError::BadRequest("finger_token is required".to_owned()));
    }
    let outcome = enrollment_service::bind(&state, &ctx.device, token).await?;
    Ok(Json(EnrollBindResponse {
        bound: outcome.bound,
        message: outcome.message,
        user: outcome.user.map(UserBrief::from),
    }))
}

async fn heartbeat(
    State(state): State<AppState>,
    ctx: DeviceCtx,
) -> AppResult<Json<HeartbeatResponse>> {
    // Only the GSM node needs a queue depth, so skip the count for the other one.
    let pending_sms = if ctx.device.sms_capable {
        sms_service::pending(&state.db, SMS_BATCH).await?.len() as u64
    } else {
        0
    };
    Ok(Json(HeartbeatResponse {
        ok: true,
        sms_capable: ctx.device.sms_capable,
        pending_sms,
    }))
}

/// The queue, for the node holding the SIM800L. Refused elsewhere so a second
/// node cannot send every alert twice.
async fn pending_sms(
    State(state): State<AppState>,
    ctx: DeviceCtx,
) -> AppResult<Json<Vec<OutboundSms>>> {
    if !ctx.device.sms_capable {
        return Err(AppError::Forbidden);
    }
    let items = sms_service::pending(&state.db, SMS_BATCH).await?;
    Ok(Json(
        items
            .into_iter()
            .map(|m| OutboundSms {
                id: m.id,
                phone_number: m.phone_number,
                body: m.body,
                attempts: m.attempts,
            })
            .collect(),
    ))
}

async fn sms_result(
    State(state): State<AppState>,
    ctx: DeviceCtx,
    Path(id): Path<Uuid>,
    Json(body): Json<SmsResultRequest>,
) -> AppResult<Json<OutboundSms>> {
    if !ctx.device.sms_capable {
        return Err(AppError::Forbidden);
    }
    let message =
        sms_service::record_result(&state.db, id, ctx.device.id, body.sent, body.error).await?;
    Ok(Json(OutboundSms {
        id: message.id,
        phone_number: message.phone_number,
        body: message.body,
        attempts: message.attempts,
    }))
}
