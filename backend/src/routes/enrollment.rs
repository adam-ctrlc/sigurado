use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::extractor::AuthUser,
    entities::{
        enrollment_sessions, prelude::EnrollmentSessions, sea_orm_active_enums::EnrollmentStatus,
    },
    error::AppResult,
    services::enrollment_service,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct VerifyCodeRequest {
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct EnrollmentResponse {
    pub id: Uuid,
    pub device_id: Uuid,
    pub status: EnrollmentStatus,
    pub created_at: DateTimeWithTimeZone,
    pub code_expires_at: DateTimeWithTimeZone,
    pub bound_at: Option<DateTimeWithTimeZone>,
}

impl From<enrollment_sessions::Model> for EnrollmentResponse {
    fn from(e: enrollment_sessions::Model) -> Self {
        Self {
            id: e.id,
            device_id: e.device_id,
            status: e.status,
            created_at: e.created_at,
            code_expires_at: e.code_expires_at,
            bound_at: e.bound_at,
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/enrollment/verify-code", post(verify_code))
        .route("/enrollment/mine", get(mine))
}

async fn verify_code(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<VerifyCodeRequest>,
) -> AppResult<Json<EnrollmentResponse>> {
    let code = body.code.trim().to_uppercase();
    let session = enrollment_service::verify_code(&state, auth.user_id, &code).await?;
    Ok(Json(session.into()))
}

async fn mine(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<EnrollmentResponse>>> {
    let items = EnrollmentSessions::find()
        .filter(enrollment_sessions::Column::UserId.eq(auth.user_id))
        .order_by_desc(enrollment_sessions::Column::CreatedAt)
        .all(&state.db)
        .await?;
    Ok(Json(
        items.into_iter().map(EnrollmentResponse::from).collect(),
    ))
}
