use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{extractor::AdminUser, password::hash_password},
    entities::{devices, prelude::Devices, sea_orm_active_enums::DeviceKind},
    error::{AppError, AppResult},
    services::uptime_service,
    state::AppState,
    util,
};

const SECRET_LEN: usize = 40;

#[derive(Debug, Serialize)]
pub struct DeviceResponse {
    pub id: Uuid,
    pub name: String,
    pub kind: DeviceKind,
    pub last_seen_at: Option<DateTimeWithTimeZone>,
    /// True on the node that carries the SIM800L and drains the SMS queue.
    pub sms_capable: bool,
    /// Start of the node's current unbroken stretch of contact, or None when it
    /// is offline. Lets the UI say "connected for 3 hours" rather than only
    /// "last seen at".
    pub connected_since: Option<DateTimeWithTimeZone>,
    pub is_active: bool,
    pub created_at: DateTimeWithTimeZone,
}

impl From<devices::Model> for DeviceResponse {
    fn from(d: devices::Model) -> Self {
        Self {
            id: d.id,
            name: d.name,
            kind: d.kind,
            last_seen_at: d.last_seen_at,
            sms_capable: d.sms_capable,
            connected_since: None,
            is_active: d.is_active,
            created_at: d.created_at,
        }
    }
}

/// One stretch of contact with a node. `ended_at` is None while it is current.
#[derive(Debug, Serialize)]
pub struct ConnectionResponse {
    pub id: Uuid,
    pub connected_at: DateTimeWithTimeZone,
    pub last_seen_at: DateTimeWithTimeZone,
    pub ended_at: Option<DateTimeWithTimeZone>,
    pub seconds: i64,
    pub heartbeats: i32,
    pub current: bool,
}

/// Includes the plaintext secret; returned only once, on create/rotate.
#[derive(Debug, Serialize)]
pub struct DeviceWithSecretResponse {
    #[serde(flatten)]
    pub device: DeviceResponse,
    pub secret: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateDeviceRequest {
    pub name: String,
    pub kind: DeviceKind,
    /// True for the node that carries the SIM800L.
    #[serde(default)]
    pub sms_capable: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDeviceRequest {
    pub sms_capable: Option<bool>,
    pub is_active: Option<bool>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/devices", post(create_device).get(list_devices))
        .route("/devices/{id}", get(get_device))
        .route("/devices/{id}/rotate-secret", post(rotate_secret))
        .route("/devices/{id}/connections", get(list_connections))
        .route("/devices/{id}", axum::routing::patch(update_device))
}

async fn create_device(
    State(state): State<AppState>,
    _admin: AdminUser,
    Json(body): Json<CreateDeviceRequest>,
) -> AppResult<(StatusCode, Json<DeviceWithSecretResponse>)> {
    let secret = util::random_token(SECRET_LEN);
    let device = devices::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(body.name),
        kind: Set(body.kind),
        secret_hash: Set(hash_password(&secret)?),
        last_seen_at: Set(None),
        is_active: Set(true),
        sms_capable: Set(body.sms_capable),
        created_at: Set(util::now()),
    }
    .insert(&state.db)
    .await
    .map_err(|_| AppError::Conflict("a device with that name already exists".to_owned()))?;

    Ok((
        StatusCode::CREATED,
        Json(DeviceWithSecretResponse {
            device: device.into(),
            secret,
        }),
    ))
}

/// Fills in the live uptime marker, which the plain From impl cannot know.
async fn with_uptime(state: &AppState, device: devices::Model) -> AppResult<DeviceResponse> {
    let connected_since = uptime_service::connected_since(&state.db, device.id).await?;
    let mut response = DeviceResponse::from(device);
    response.connected_since = connected_since;
    Ok(response)
}

async fn list_devices(
    State(state): State<AppState>,
    _admin: AdminUser,
) -> AppResult<Json<Vec<DeviceResponse>>> {
    let devices = Devices::find()
        .order_by_asc(devices::Column::Name)
        .all(&state.db)
        .await?;
    let mut out = Vec::with_capacity(devices.len());
    for device in devices {
        out.push(with_uptime(&state, device).await?);
    }
    Ok(Json(out))
}

/// Connectivity history for one node, newest stretch first.
async fn list_connections(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<ConnectionResponse>>> {
    let now = util::now();
    let periods = uptime_service::history(&state.db, id, 30).await?;
    Ok(Json(
        periods
            .into_iter()
            .map(|p| {
                let current = uptime_service::is_live(Some(p.last_seen_at), now);
                ConnectionResponse {
                    id: p.id,
                    connected_at: p.connected_at,
                    last_seen_at: p.last_seen_at,
                    // While current the stretch has no end; once it has gone
                    // quiet, the last beat is the honest end of it.
                    ended_at: if current { None } else { Some(p.last_seen_at) },
                    seconds: (if current { now } else { p.last_seen_at } - p.connected_at)
                        .num_seconds(),
                    heartbeats: p.heartbeats,
                    current,
                }
            })
            .collect(),
    ))
}

/// Flips the modem flag, so an admin can say which node carries the SIM800L.
async fn update_device(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateDeviceRequest>,
) -> AppResult<Json<DeviceResponse>> {
    let device = Devices::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("device"))?;

    let mut active: devices::ActiveModel = device.into();
    if let Some(v) = body.sms_capable {
        active.sms_capable = Set(v);
    }
    if let Some(v) = body.is_active {
        active.is_active = Set(v);
    }
    let device = active.update(&state.db).await?;
    Ok(Json(with_uptime(&state, device).await?))
}

async fn get_device(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<DeviceResponse>> {
    let device = Devices::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("device"))?;
    Ok(Json(with_uptime(&state, device).await?))
}

async fn rotate_secret(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<DeviceWithSecretResponse>> {
    let device = Devices::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("device"))?;

    let secret = util::random_token(SECRET_LEN);
    let mut active: devices::ActiveModel = device.into();
    active.secret_hash = Set(hash_password(&secret)?);
    let device = active.update(&state.db).await?;

    Ok(Json(DeviceWithSecretResponse {
        device: device.into(),
        secret,
    }))
}
