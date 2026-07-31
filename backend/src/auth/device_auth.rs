use axum::{extract::FromRequestParts, http::request::Parts};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use uuid::Uuid;

use crate::{
    auth::password::verify_password,
    entities::{devices, prelude::Devices},
    error::AppError,
    services::uptime_service,
    state::AppState,
    util,
};

/// A verified physical device (ESP32-S3 node, or the simulator standing in for
/// one), authenticated by the `X-Device-Id` + `X-Device-Secret` headers.
pub struct DeviceCtx {
    pub device: devices::Model,
}

impl FromRequestParts<AppState> for DeviceCtx {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let device_id = parts
            .headers
            .get("x-device-id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or(AppError::Unauthorized)?;
        let secret = parts
            .headers
            .get("x-device-secret")
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let device = Devices::find_by_id(device_id)
            .one(&state.db)
            .await?
            .ok_or(AppError::Unauthorized)?;

        if !device.is_active || !verify_password(secret, &device.secret_hash) {
            return Err(AppError::Unauthorized);
        }

        let mut active: devices::ActiveModel = device.into();
        active.last_seen_at = Set(Some(util::now()));
        let device = active.update(&state.db).await?;

        // Any authenticated call is a sign of life, not just /device/heartbeat.
        uptime_service::record_contact(&state.db, device.id).await?;

        Ok(Self { device })
    }
}
