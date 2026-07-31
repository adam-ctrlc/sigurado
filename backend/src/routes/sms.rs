//! Admin surface for SMS alerts.
//!
//! A recipient is a staff account, not a standalone entry: the name and the
//! number come from the user record, so a custodian's number is stored once and
//! a demoted or disabled account stops receiving alerts on its own.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, patch, post},
};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    Set,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    auth::extractor::AdminUser,
    entities::{
        prelude::*,
        sea_orm_active_enums::{Role, SmsStatus},
        sms_messages, sms_recipients, users,
    },
    error::{AppError, AppResult},
    pagination::{Page, page_of, per_page_of},
    services::sms_service,
    state::AppState,
    util,
};

#[derive(Debug, Serialize)]
pub struct RecipientResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub display_name: String,
    pub username: String,
    pub role: Role,
    /// From the user record; None means nothing can be sent to them yet.
    pub phone_number: Option<String>,
    pub user_is_active: bool,
    pub notify_cabinet_opened: bool,
    pub notify_access_denied: bool,
    pub notify_door_opened: bool,
    pub is_active: bool,
    pub created_at: DateTimeWithTimeZone,
}

fn to_recipient(recipient: sms_recipients::Model, user: &users::Model) -> RecipientResponse {
    RecipientResponse {
        id: recipient.id,
        user_id: user.id,
        display_name: user.display_name(),
        username: user.username.clone(),
        role: user.role,
        phone_number: user.phone_number.clone(),
        user_is_active: user.is_active,
        notify_cabinet_opened: recipient.notify_cabinet_opened,
        notify_access_denied: recipient.notify_access_denied,
        notify_door_opened: recipient.notify_door_opened,
        is_active: recipient.is_active,
        created_at: recipient.created_at,
    }
}

/// A staff account that could be subscribed, for the picker.
#[derive(Debug, Serialize)]
pub struct CandidateResponse {
    pub id: Uuid,
    pub display_name: String,
    pub username: String,
    pub role: Role,
    pub phone_number: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub recipient_id: Option<Uuid>,
    pub recipient_label: Option<String>,
    pub phone_number: String,
    pub body: String,
    pub status: SmsStatus,
    pub attempts: i32,
    pub error: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub sent_at: Option<DateTimeWithTimeZone>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRecipientRequest {
    pub user_id: Uuid,
    /// Optional: sets the number on the user in the same step, so an admin does
    /// not have to visit Users first.
    #[serde(default)]
    pub phone_number: Option<String>,
    #[serde(default = "yes")]
    pub notify_cabinet_opened: bool,
    #[serde(default = "yes")]
    pub notify_access_denied: bool,
    #[serde(default)]
    pub notify_door_opened: bool,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct UpdateRecipientRequest {
    /// Writes through to the user record.
    pub phone_number: Option<String>,
    pub notify_cabinet_opened: Option<bool>,
    pub notify_access_denied: Option<bool>,
    pub notify_door_opened: Option<bool>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct MessageQuery {
    pub status: Option<SmsStatus>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/sms/recipients",
            get(list_recipients).post(create_recipient),
        )
        .route(
            "/sms/recipients/{id}",
            patch(update_recipient).delete(delete_recipient),
        )
        .route("/sms/recipients/{id}/test", post(send_test))
        .route("/sms/candidates", get(list_candidates))
        .route("/sms/messages", get(list_messages))
}

/// Loads the recipient together with the account behind it.
async fn recipient_with_user(
    state: &AppState,
    id: Uuid,
) -> AppResult<(sms_recipients::Model, users::Model)> {
    let recipient = SmsRecipients::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("recipient"))?;
    let user = Users::find_by_id(recipient.user_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("user"))?;
    Ok((recipient, user))
}

async fn list_recipients(
    State(state): State<AppState>,
    _admin: AdminUser,
) -> AppResult<Json<Vec<RecipientResponse>>> {
    let pairs: Vec<(sms_recipients::Model, Option<users::Model>)> = SmsRecipients::find()
        .order_by_asc(sms_recipients::Column::CreatedAt)
        .find_also_related(Users)
        .all(&state.db)
        .await?;

    Ok(Json(
        pairs
            .into_iter()
            .filter_map(|(r, u)| u.map(|user| to_recipient(r, &user)))
            .collect(),
    ))
}

/// Staff who are not subscribed yet, so the picker only offers real options.
async fn list_candidates(
    State(state): State<AppState>,
    _admin: AdminUser,
) -> AppResult<Json<Vec<CandidateResponse>>> {
    let taken: Vec<Uuid> = SmsRecipients::find()
        .all(&state.db)
        .await?
        .into_iter()
        .map(|r| r.user_id)
        .collect();

    let mut query = Users::find()
        .filter(users::Column::IsActive.eq(true))
        .filter(
            Condition::any()
                .add(users::Column::Role.eq(Role::Admin))
                .add(users::Column::Role.eq(Role::Faculty)),
        );
    if !taken.is_empty() {
        query = query.filter(users::Column::Id.is_not_in(taken));
    }

    let staff = query
        .order_by_asc(users::Column::FirstName)
        .all(&state.db)
        .await?;

    Ok(Json(
        staff
            .into_iter()
            .map(|u| CandidateResponse {
                display_name: u.display_name(),
                id: u.id,
                username: u.username,
                role: u.role,
                phone_number: u.phone_number,
            })
            .collect(),
    ))
}

async fn create_recipient(
    State(state): State<AppState>,
    _admin: AdminUser,
    Json(body): Json<CreateRecipientRequest>,
) -> AppResult<(StatusCode, Json<RecipientResponse>)> {
    let user = Users::find_by_id(body.user_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("user"))?;

    if !sms_service::may_receive_alerts(user.role) {
        return Err(AppError::Validation(
            "only faculty and admin accounts can receive alerts".to_owned(),
        ));
    }

    let existing = SmsRecipients::find()
        .filter(sms_recipients::Column::UserId.eq(user.id))
        .one(&state.db)
        .await?;
    if existing.is_some() {
        return Err(AppError::Conflict(
            "that person is already on the list".to_owned(),
        ));
    }

    // Setting the number here writes it onto the account, which is the one place
    // it is stored.
    let user = match body
        .phone_number
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        Some(raw) => {
            let normalized = sms_service::normalize_number(raw)?;
            let mut active: users::ActiveModel = user.into();
            active.phone_number = Set(Some(normalized));
            active.updated_at = Set(util::now());
            active.update(&state.db).await?
        }
        None => {
            if user.phone_number.is_none() {
                return Err(AppError::Validation(
                    "that account has no mobile number yet, so add one here".to_owned(),
                ));
            }
            user
        }
    };

    let recipient = sms_recipients::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user.id),
        notify_cabinet_opened: Set(body.notify_cabinet_opened),
        notify_access_denied: Set(body.notify_access_denied),
        notify_door_opened: Set(body.notify_door_opened),
        is_active: Set(true),
        created_at: Set(util::now()),
    }
    .insert(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(to_recipient(recipient, &user))))
}

async fn update_recipient(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateRecipientRequest>,
) -> AppResult<Json<RecipientResponse>> {
    let (recipient, user) = recipient_with_user(&state, id).await?;

    let user = match body.phone_number {
        Some(raw) => {
            let trimmed = raw.trim();
            let value = if trimmed.is_empty() {
                None
            } else {
                Some(sms_service::normalize_number(trimmed)?)
            };
            let mut active: users::ActiveModel = user.into();
            active.phone_number = Set(value);
            active.updated_at = Set(util::now());
            active.update(&state.db).await?
        }
        None => user,
    };

    let mut active: sms_recipients::ActiveModel = recipient.into();
    if let Some(v) = body.notify_cabinet_opened {
        active.notify_cabinet_opened = Set(v);
    }
    if let Some(v) = body.notify_access_denied {
        active.notify_access_denied = Set(v);
    }
    if let Some(v) = body.notify_door_opened {
        active.notify_door_opened = Set(v);
    }
    if let Some(v) = body.is_active {
        active.is_active = Set(v);
    }

    let recipient = active.update(&state.db).await?;
    Ok(Json(to_recipient(recipient, &user)))
}

async fn delete_recipient(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    // Alerts exist so that somebody hears about a cabinet opening. Emptying the
    // list entirely would leave the feature switched on but silent, so the last
    // subscriber has to be replaced rather than removed.
    let total = SmsRecipients::find().count(&state.db).await?;
    if total <= 1 {
        return Err(AppError::Validation(
            "this is the only person subscribed; add someone else before removing them".to_owned(),
        ));
    }

    let result = SmsRecipients::delete_by_id(id).exec(&state.db).await?;
    if result.rows_affected == 0 {
        return Err(AppError::NotFound("recipient"));
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Queues a test message, so an admin can prove the modem and the number work
/// without waiting for someone to open the cabinet.
async fn send_test(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<(StatusCode, Json<MessageResponse>)> {
    let (recipient, user) = recipient_with_user(&state, id).await?;
    let number = user
        .phone_number
        .clone()
        .ok_or_else(|| AppError::Validation("that account has no mobile number yet".to_owned()))?;

    let queued = sms_service::queue_direct(
        &state.db,
        &recipient,
        &number,
        "Sigurado: test alert. If you got this, the SIM800L and this number both work.",
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(MessageResponse {
            recipient_label: Some(user.display_name()),
            id: queued.id,
            recipient_id: queued.recipient_id,
            phone_number: queued.phone_number,
            body: queued.body,
            status: queued.status,
            attempts: queued.attempts,
            error: queued.error,
            created_at: queued.created_at,
            sent_at: queued.sent_at,
        }),
    ))
}

async fn list_messages(
    State(state): State<AppState>,
    _admin: AdminUser,
    Query(params): Query<MessageQuery>,
) -> AppResult<Json<Page<MessageResponse>>> {
    let page = page_of(params.page);
    let per_page = per_page_of(params.per_page);

    let mut query = SmsMessages::find();
    if let Some(status) = params.status {
        query = query.filter(sms_messages::Column::Status.eq(status));
    }

    let paginator = query
        .order_by_desc(sms_messages::Column::CreatedAt)
        .paginate(&state.db, per_page);
    let total = paginator.num_items().await?;
    let items = paginator.fetch_page(page - 1).await?;

    // Two lookups for the labels rather than a query per row.
    let recipients: HashMap<Uuid, Uuid> = SmsRecipients::find()
        .all(&state.db)
        .await?
        .into_iter()
        .map(|r| (r.id, r.user_id))
        .collect();
    let names: HashMap<Uuid, String> = Users::find()
        .all(&state.db)
        .await?
        .into_iter()
        .map(|u| (u.id, u.display_name()))
        .collect();

    let responses = items
        .into_iter()
        .map(|m| MessageResponse {
            recipient_label: m
                .recipient_id
                .and_then(|id| recipients.get(&id))
                .and_then(|user_id| names.get(user_id).cloned()),
            id: m.id,
            recipient_id: m.recipient_id,
            phone_number: m.phone_number,
            body: m.body,
            status: m.status,
            attempts: m.attempts,
            error: m.error,
            created_at: m.created_at,
            sent_at: m.sent_at,
        })
        .collect();

    Ok(Json(Page::new(responses, total, page, per_page)))
}
