use std::collections::HashMap;

use axum::{
    Json, Router,
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::extractor::AuthUser,
    entities::{
        checkouts,
        prelude::*,
        sea_orm_active_enums::{AccessEventType, EventDecision, Role},
    },
    error::{AppError, AppResult},
    pagination::{Page, page_of, per_page_of, search_term},
    services::{
        checkout_code_service,
        checkout_service::{self, NewCheckout, PhotoUpload},
        event_service::{self, NewEvent},
        purge_service,
    },
    state::AppState,
};

#[derive(Debug, Serialize)]
pub struct CheckoutResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_name: Option<String>,
    pub note: String,
    pub photo_path: Option<String>,
    pub photo_mime: Option<String>,
    pub access_session_id: Option<Uuid>,
    pub box_device_id: Option<Uuid>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Debug, Deserialize)]
pub struct CheckoutQuery {
    /// Free text over the note, and the person's name for staff.
    pub q: Option<String>,
    /// Staff only: narrow to one person.
    pub user_id: Option<Uuid>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct PendingCodeResponse {
    /// True when the cabinet has issued a code that is still good.
    pub waiting: bool,
    pub expires_at: Option<DateTimeWithTimeZone>,
    pub issued_at: Option<DateTimeWithTimeZone>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/checkouts", post(create_checkout).get(list_checkouts))
        .route("/checkouts/pending-code", get(pending_code))
        .route("/checkouts/{id}", get(get_checkout))
}

async fn create_checkout(
    State(state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> AppResult<(StatusCode, Json<CheckoutResponse>)> {
    let mut note: Option<String> = None;
    let mut code: Option<String> = None;
    let mut access_session_id: Option<Uuid> = None;
    let mut box_device_id: Option<Uuid> = None;
    let mut photo: Option<PhotoUpload> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("invalid multipart form: {e}")))?
    {
        match field.name() {
            Some("note") => {
                note = Some(read_text(field).await?);
            }
            Some("code") => {
                code = Some(read_text(field).await?);
            }
            Some("access_session_id") => {
                access_session_id = parse_optional_uuid(read_text(field).await?)?;
            }
            Some("box_device_id") => {
                box_device_id = parse_optional_uuid(read_text(field).await?)?;
            }
            Some("photo") => {
                let mime = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_owned();
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("could not read photo: {e}")))?;
                if !bytes.is_empty() {
                    photo = Some(PhotoUpload {
                        bytes: bytes.to_vec(),
                        mime,
                    });
                }
            }
            _ => {}
        }
    }

    let note = note.ok_or_else(|| AppError::Validation("a note is required".to_owned()))?;

    // The cabinet showed a code when it opened. Demanding it back is what ties
    // this record to a real opening, so nobody can log materials from a couch.
    let typed = code.as_deref().unwrap_or_default().to_owned();
    let claimed = match checkout_code_service::claim(&state.db, auth.user_id, &typed).await {
        Ok(code) => code,
        Err(err) => {
            // A refused code is worth recording: repeated attempts are someone
            // guessing, and the flags page looks for exactly that.
            let attempted: String = typed.chars().take(12).collect();
            let _ = event_service::emit(
                &state,
                NewEvent {
                    event_type: AccessEventType::CheckoutCodeRejected,
                    decision: EventDecision::Denied,
                    device_id: None,
                    user_id: Some(auth.user_id),
                    finger_token: None,
                    access_session_id: None,
                    enrollment_session_id: None,
                    message: Some(format!("checkout code refused: {attempted}")),
                },
            )
            .await;
            return Err(err);
        }
    };

    let checkout = checkout_service::create(
        &state,
        NewCheckout {
            user_id: auth.user_id,
            note,
            // Prefer what the code knows: it came from the reader itself.
            access_session_id: claimed.access_session_id.or(access_session_id),
            box_device_id: claimed.device_id.or(box_device_id),
            photo,
        },
    )
    .await?;

    // Spent only once the row exists, so a failed write leaves the code usable.
    checkout_code_service::spend(&state.db, claimed, checkout.id, checkout.created_at).await?;

    let user_name = Users::find_by_id(auth.user_id)
        .one(&state.db)
        .await?
        .map(|u| u.display_name());

    Ok((StatusCode::CREATED, Json(to_response(checkout, user_name))))
}

/// Whether the signed-in person currently holds an unspent cabinet code. The
/// code itself is never returned: it has to be read off the LCD.
async fn pending_code(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<PendingCodeResponse>> {
    let live = checkout_code_service::live_for_user(&state.db, auth.user_id).await?;
    Ok(Json(PendingCodeResponse {
        waiting: live.is_some(),
        expires_at: live.as_ref().map(|c| c.expires_at),
        issued_at: live.as_ref().map(|c| c.created_at),
    }))
}

async fn list_checkouts(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<CheckoutQuery>,
) -> AppResult<Json<Page<CheckoutResponse>>> {
    let page = page_of(params.page);
    let per_page = per_page_of(params.per_page);

    let hidden_before = purge_service::cutoff(&state.db).await?;

    let mut query = Checkouts::find();
    if let Some(since) = hidden_before {
        query = query.filter(checkouts::Column::CreatedAt.gt(since));
    }

    // Students only see their own checkouts; faculty and admin see everything.
    if auth.role == Role::Student {
        query = query.filter(checkouts::Column::UserId.eq(auth.user_id));
    } else if let Some(user_id) = params.user_id {
        query = query.filter(checkouts::Column::UserId.eq(user_id));
    }

    if let Some(term) = search_term(params.q) {
        let mut any = Condition::any().add(checkouts::Column::Note.contains(&term));
        // Staff can search by who took it, which lives on the user row.
        if auth.role != Role::Student {
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
            if !user_hits.is_empty() {
                any = any.add(checkouts::Column::UserId.is_in(user_hits));
            }
        }
        query = query.filter(any);
    }

    let paginator = query
        .order_by_desc(checkouts::Column::CreatedAt)
        .paginate(&state.db, per_page);
    let total = paginator.num_items().await?;
    let items = paginator.fetch_page(page - 1).await?;

    let user_ids: Vec<Uuid> = items.iter().map(|c| c.user_id).collect();
    let names: HashMap<Uuid, String> = Users::find()
        .filter(crate::entities::users::Column::Id.is_in(user_ids))
        .all(&state.db)
        .await?
        .into_iter()
        .map(|u| (u.id, u.display_name()))
        .collect();

    let responses = items
        .into_iter()
        .map(|c| {
            let name = names.get(&c.user_id).cloned();
            to_response(c, name)
        })
        .collect();

    Ok(Json(Page::new(responses, total, page, per_page)))
}

async fn get_checkout(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<CheckoutResponse>> {
    // A record hidden by a clearing reads as gone, the same as it does in the
    // list, rather than staying reachable by guessing its id.
    let mut one = Checkouts::find_by_id(id);
    if let Some(since) = purge_service::cutoff(&state.db).await? {
        one = one.filter(checkouts::Column::CreatedAt.gt(since));
    }
    let checkout = one
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("checkout"))?;

    if auth.role == Role::Student && checkout.user_id != auth.user_id {
        return Err(AppError::Forbidden);
    }

    let user_name = Users::find_by_id(checkout.user_id)
        .one(&state.db)
        .await?
        .map(|u| u.display_name());

    Ok(Json(to_response(checkout, user_name)))
}

fn to_response(c: checkouts::Model, user_name: Option<String>) -> CheckoutResponse {
    CheckoutResponse {
        id: c.id,
        user_id: c.user_id,
        user_name,
        note: c.note,
        photo_path: c.photo_path,
        photo_mime: c.photo_mime,
        access_session_id: c.access_session_id,
        box_device_id: c.box_device_id,
        created_at: c.created_at,
    }
}

async fn read_text(field: axum::extract::multipart::Field<'_>) -> AppResult<String> {
    field
        .text()
        .await
        .map_err(|e| AppError::BadRequest(format!("invalid form field: {e}")))
}

fn parse_optional_uuid(value: String) -> AppResult<Option<Uuid>> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    Uuid::parse_str(trimmed)
        .map(Some)
        .map_err(|_| AppError::BadRequest("invalid id in form".to_owned()))
}
