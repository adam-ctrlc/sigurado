use std::{net::SocketAddr, time::Duration};

use axum::{
    Json, Router,
    extract::{ConnectInfo, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{
        extractor::AuthUser,
        password::{hash_password, verify_password},
        policy::validate_password,
    },
    entities::{login_events, prelude::*, users},
    error::{AppError, AppResult},
    pagination::{Page, page_of, per_page_of},
    routes::users::{LoginEventResponse, UserResponse},
    services::sms_service,
    state::AppState,
    util,
};

const AUTH_TOKEN_TTL: Duration = Duration::from_secs(60 * 60 * 12);
/// Long enough for any real browser string, short enough that nobody can stuff
/// the audit table with a megabyte of header.
const MAX_USER_AGENT: usize = 512;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub identifier: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMeRequest {
    pub email: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    /// An empty string clears it; omitting the field leaves it alone.
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub suffix: Option<String>,
    /// An empty string clears it.
    pub phone_number: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/me", get(me).patch(update_me))
        .route("/auth/password", post(change_password))
        .route("/auth/my-logins", get(my_logins))
}

/// Trims an optional field, treating whitespace-only as absent.
fn normalize(value: Option<String>) -> Option<String> {
    value.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
}

/// The caller's address, preferring the proxy header when one is present.
fn client_ip(headers: &HeaderMap, peer: Option<SocketAddr>) -> Option<String> {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_owned)
        .or_else(|| peer.map(|addr| addr.ip().to_string()))
}

fn user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.chars().take(MAX_USER_AGENT).collect::<String>())
        .filter(|v| !v.is_empty())
}

async fn record_login(
    state: &AppState,
    user_id: Option<Uuid>,
    identifier: &str,
    success: bool,
    reason: Option<&str>,
    ip: Option<String>,
    agent: Option<String>,
) {
    let entry = login_events::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        identifier: Set(identifier.to_owned()),
        success: Set(success),
        reason: Set(reason.map(str::to_owned)),
        ip_address: Set(ip),
        user_agent: Set(agent),
        created_at: Set(util::now()),
    };
    // An audit write must never be the reason a sign-in fails.
    if let Err(err) = entry.insert(&state.db).await {
        tracing::warn!("could not record the sign-in attempt: {err}");
    }
}

async fn login(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    let ip = client_ip(&headers, Some(peer));
    let agent = user_agent(&headers);
    let identifier = body.identifier.trim();

    let found = Users::find()
        .filter(
            Condition::any()
                .add(users::Column::Email.eq(identifier))
                .add(users::Column::Username.eq(identifier)),
        )
        .one(&state.db)
        .await?;

    let Some(user) = found else {
        record_login(
            &state,
            None,
            identifier,
            false,
            Some("no such account"),
            ip,
            agent,
        )
        .await;
        return Err(AppError::Unauthorized);
    };

    let refusal = if !user.is_active {
        Some("account disabled")
    } else if !verify_password(&body.password, &user.password_hash) {
        Some("wrong password")
    } else {
        None
    };

    if let Some(reason) = refusal {
        record_login(
            &state,
            Some(user.id),
            identifier,
            false,
            Some(reason),
            ip,
            agent,
        )
        .await;
        return Err(AppError::Unauthorized);
    }

    let token = state
        .jwt
        .issue(user.id, user.role, AUTH_TOKEN_TTL)
        .map_err(AppError::Internal)?;

    record_login(&state, Some(user.id), identifier, true, None, ip, agent).await;

    let now = util::now();
    let user_id = user.id;
    let mut active: users::ActiveModel = user.into();
    active.last_login_at = Set(Some(now));
    active.updated_at = Set(now);
    let user = match active.update(&state.db).await {
        Ok(updated) => updated,
        Err(err) => {
            // Stamping the time is bookkeeping; a failure here must not block sign-in.
            tracing::warn!("could not stamp the last sign-in time: {err}");
            Users::find_by_id(user_id)
                .one(&state.db)
                .await?
                .ok_or(AppError::Unauthorized)?
        }
    };

    Ok(Json(LoginResponse {
        token,
        user: user.into(),
    }))
}

async fn me(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<UserResponse>> {
    let user = Users::find_by_id(auth.user_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;
    Ok(Json(user.into()))
}

/// A user editing their own record. Deliberately cannot touch role or the
/// active flag: those stay an administrator's decision.
async fn update_me(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<UpdateMeRequest>,
) -> AppResult<Json<UserResponse>> {
    let user = Users::find_by_id(auth.user_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if let Some(ref email) = body.email {
        let email = email.trim();
        if email.is_empty() {
            return Err(AppError::Validation("email cannot be blank".to_owned()));
        }
        let clash = Users::find()
            .filter(users::Column::Id.ne(user.id))
            .filter(users::Column::Email.eq(email))
            .one(&state.db)
            .await?;
        if clash.is_some() {
            return Err(AppError::Conflict("email already in use".to_owned()));
        }
    }

    if let Some(ref username) = body.username {
        let username = username.trim();
        if username.len() < 3 {
            return Err(AppError::Validation(
                "username must be at least 3 characters".to_owned(),
            ));
        }
        if !username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
        {
            return Err(AppError::Validation(
                "username may only use letters, digits, dots, underscores and hyphens".to_owned(),
            ));
        }
        let clash = Users::find()
            .filter(users::Column::Id.ne(user.id))
            .filter(users::Column::Username.eq(username))
            .one(&state.db)
            .await?;
        if clash.is_some() {
            return Err(AppError::Conflict("username already taken".to_owned()));
        }
    }

    if let Some(ref name) = body.first_name {
        if name.trim().is_empty() {
            return Err(AppError::Validation(
                "first name cannot be blank".to_owned(),
            ));
        }
    }
    if let Some(ref name) = body.last_name {
        if name.trim().is_empty() {
            return Err(AppError::Validation("last name cannot be blank".to_owned()));
        }
    }

    let mut active: users::ActiveModel = user.into();
    if let Some(email) = body.email {
        active.email = Set(email.trim().to_owned());
    }
    if let Some(username) = body.username {
        active.username = Set(username.trim().to_owned());
    }
    if let Some(name) = body.first_name {
        active.first_name = Set(name.trim().to_owned());
    }
    if let Some(name) = body.middle_name {
        active.middle_name = Set(normalize(Some(name)));
    }
    if let Some(name) = body.last_name {
        active.last_name = Set(name.trim().to_owned());
    }
    if let Some(value) = body.suffix {
        active.suffix = Set(normalize(Some(value)));
    }
    if let Some(value) = body.phone_number {
        active.phone_number = Set(match normalize(Some(value)) {
            Some(raw) => Some(sms_service::normalize_number(&raw)?),
            None => None,
        });
    }
    active.updated_at = Set(util::now());

    let user = active.update(&state.db).await?;
    Ok(Json(user.into()))
}

/// A password change always proves the current password first, so a borrowed
/// session cannot lock the owner out of their own account.
async fn change_password(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<ChangePasswordRequest>,
) -> AppResult<StatusCode> {
    validate_password(&body.new_password)?;

    let user = Users::find_by_id(auth.user_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if !verify_password(&body.current_password, &user.password_hash) {
        return Err(AppError::Validation(
            "the current password is not correct".to_owned(),
        ));
    }

    let mut active: users::ActiveModel = user.into();
    active.password_hash = Set(hash_password(&body.new_password)?);
    active.updated_at = Set(util::now());
    active.update(&state.db).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Sign-in history for the caller, so anyone can spot a session they did not start.
async fn my_logins(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<PageQuery>,
) -> AppResult<Json<Page<LoginEventResponse>>> {
    let page = page_of(params.page);
    let per_page = per_page_of(params.per_page);

    let paginator = LoginEvents::find()
        .filter(login_events::Column::UserId.eq(auth.user_id))
        .order_by_desc(login_events::Column::CreatedAt)
        .paginate(&state.db, per_page);
    let total = paginator.num_items().await?;
    let items = paginator.fetch_page(page - 1).await?;

    Ok(Json(Page::new(
        items.into_iter().map(LoginEventResponse::from).collect(),
        total,
        page,
        per_page,
    )))
}
