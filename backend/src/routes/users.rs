use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{extractor::AdminUser, password::hash_password, policy::validate_password},
    entities::{fingerprints, login_events, prelude::*, sea_orm_active_enums::Role, users},
    error::{AppError, AppResult},
    pagination::{Page, page_of, per_page_of, search_term},
    services::sms_service,
    state::AppState,
    util,
};

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub suffix: Option<String>,
    /// Convenience for the UI: the name parts joined for display.
    pub display_name: String,
    /// Where SMS alerts go. Staff only in practice, since only they subscribe.
    pub phone_number: Option<String>,
    pub role: Role,
    pub is_active: bool,
    pub last_login_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
}

impl From<users::Model> for UserResponse {
    fn from(u: users::Model) -> Self {
        Self {
            display_name: u.display_name(),
            id: u.id,
            email: u.email,
            username: u.username,
            first_name: u.first_name,
            middle_name: u.middle_name,
            last_name: u.last_name,
            suffix: u.suffix,
            phone_number: u.phone_number,
            role: u.role,
            is_active: u.is_active,
            last_login_at: u.last_login_at,
            created_at: u.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct LoginEventResponse {
    pub id: Uuid,
    pub success: bool,
    pub reason: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

impl From<login_events::Model> for LoginEventResponse {
    fn from(e: login_events::Model) -> Self {
        Self {
            id: e.id,
            success: e.success,
            reason: e.reason,
            ip_address: e.ip_address,
            user_agent: e.user_agent,
            created_at: e.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct FingerprintResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub finger_token: String,
    pub label: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

impl From<fingerprints::Model> for FingerprintResponse {
    fn from(f: fingerprints::Model) -> Self {
        Self {
            id: f.id,
            user_id: f.user_id,
            finger_token: f.finger_token,
            label: f.label,
            created_at: f.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub username: String,
    pub password: String,
    pub first_name: String,
    #[serde(default)]
    pub middle_name: Option<String>,
    pub last_name: String,
    #[serde(default)]
    pub suffix: Option<String>,
    #[serde(default)]
    pub phone_number: Option<String>,
    #[serde(default)]
    pub role: Option<Role>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    /// An empty string clears the middle name; omitting the field leaves it alone.
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub suffix: Option<String>,
    /// An empty string clears it.
    pub phone_number: Option<String>,
    pub role: Option<Role>,
    pub is_active: Option<bool>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UserQuery {
    /// Free text over the name parts, username and email.
    pub q: Option<String>,
    pub role: Option<Role>,
    pub active: Option<bool>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct LoginQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users", post(create_user).get(list_users))
        .route(
            "/users/{id}",
            get(get_user).patch(update_user).delete(delete_user),
        )
        .route("/users/{id}/fingerprints", get(list_fingerprints))
        .route("/users/{id}/logins", get(list_logins))
        .route("/fingerprints/{id}", delete(delete_fingerprint))
}

/// Trims an optional name part, treating whitespace-only as absent.
fn normalize(value: Option<String>) -> Option<String> {
    value.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
}

/// Refuses to leave the system without a way back in: the last active admin
/// cannot be demoted, disabled, or deleted.
async fn assert_not_last_admin(state: &AppState, id: Uuid) -> AppResult<()> {
    let others = Users::find()
        .filter(users::Column::Role.eq(Role::Admin))
        .filter(users::Column::IsActive.eq(true))
        .filter(users::Column::Id.ne(id))
        .count(&state.db)
        .await?;
    if others == 0 {
        return Err(AppError::Validation(
            "this is the last active admin".to_owned(),
        ));
    }
    Ok(())
}

async fn create_user(
    State(state): State<AppState>,
    _admin: AdminUser,
    Json(body): Json<CreateUserRequest>,
) -> AppResult<(StatusCode, Json<UserResponse>)> {
    validate_password(&body.password)?;

    let clash = Users::find()
        .filter(
            Condition::any()
                .add(users::Column::Email.eq(&body.email))
                .add(users::Column::Username.eq(&body.username)),
        )
        .one(&state.db)
        .await?;
    if clash.is_some() {
        return Err(AppError::Conflict(
            "email or username already in use".to_owned(),
        ));
    }

    // Normalized up front so every number in the database has one shape.
    let phone = match normalize(body.phone_number) {
        Some(raw) => Some(sms_service::normalize_number(&raw)?),
        None => None,
    };

    let now = util::now();
    let user = users::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(body.email),
        username: Set(body.username),
        password_hash: Set(hash_password(&body.password)?),
        first_name: Set(body.first_name.trim().to_owned()),
        middle_name: Set(normalize(body.middle_name)),
        last_name: Set(body.last_name.trim().to_owned()),
        suffix: Set(normalize(body.suffix)),
        phone_number: Set(phone),
        last_login_at: Set(None),
        role: Set(body.role.unwrap_or(Role::Student)),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(user.into())))
}

async fn list_users(
    State(state): State<AppState>,
    _admin: AdminUser,
    Query(params): Query<UserQuery>,
) -> AppResult<Json<Page<UserResponse>>> {
    let page = page_of(params.page);
    let per_page = per_page_of(params.per_page);

    let mut query = Users::find();
    if let Some(role) = params.role {
        query = query.filter(users::Column::Role.eq(role));
    }
    if let Some(active) = params.active {
        query = query.filter(users::Column::IsActive.eq(active));
    }
    if let Some(term) = search_term(params.q) {
        // SQLite's LIKE is case-insensitive for ASCII, which is all a username
        // or email can hold here.
        query = query.filter(
            Condition::any()
                .add(users::Column::FirstName.contains(&term))
                .add(users::Column::MiddleName.contains(&term))
                .add(users::Column::LastName.contains(&term))
                .add(users::Column::Username.contains(&term))
                .add(users::Column::Email.contains(&term)),
        );
    }

    let paginator = query
        .order_by_desc(users::Column::CreatedAt)
        .paginate(&state.db, per_page);
    let total = paginator.num_items().await?;
    let items = paginator.fetch_page(page - 1).await?;

    Ok(Json(Page::new(
        items.into_iter().map(UserResponse::from).collect(),
        total,
        page,
        per_page,
    )))
}

async fn get_user(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<UserResponse>> {
    let user = Users::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("user"))?;
    Ok(Json(user.into()))
}

async fn update_user(
    State(state): State<AppState>,
    admin: AdminUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    let user = Users::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("user"))?;

    if admin.0.user_id == id {
        if matches!(body.role, Some(ref r) if *r != user.role) {
            return Err(AppError::Validation(
                "you cannot change your own role".to_owned(),
            ));
        }
        if body.is_active == Some(false) {
            return Err(AppError::Validation(
                "you cannot disable your own account".to_owned(),
            ));
        }
    }

    let was_admin = user.role == Role::Admin && user.is_active;
    let losing_admin = was_admin
        && (matches!(body.role, Some(ref r) if *r != Role::Admin) || body.is_active == Some(false));
    if losing_admin {
        assert_not_last_admin(&state, id).await?;
    }

    if body.email.is_some() || body.username.is_some() {
        let email = body.email.clone().unwrap_or_else(|| user.email.clone());
        let username = body
            .username
            .clone()
            .unwrap_or_else(|| user.username.clone());
        let clash = Users::find()
            .filter(users::Column::Id.ne(id))
            .filter(
                Condition::any()
                    .add(users::Column::Email.eq(&email))
                    .add(users::Column::Username.eq(&username)),
            )
            .one(&state.db)
            .await?;
        if clash.is_some() {
            return Err(AppError::Conflict(
                "email or username already in use".to_owned(),
            ));
        }
    }

    let mut active: users::ActiveModel = user.into();
    if let Some(email) = body.email {
        active.email = Set(email);
    }
    if let Some(username) = body.username {
        active.username = Set(username);
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
    if let Some(role) = body.role {
        active.role = Set(role);
    }
    if let Some(is_active) = body.is_active {
        active.is_active = Set(is_active);
    }
    if let Some(password) = body.password {
        validate_password(&password)?;
        active.password_hash = Set(hash_password(&password)?);
    }
    active.updated_at = Set(util::now());

    let user = active.update(&state.db).await?;
    Ok(Json(user.into()))
}

async fn delete_user(
    State(state): State<AppState>,
    admin: AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    if admin.0.user_id == id {
        return Err(AppError::Validation(
            "you cannot delete your own account".to_owned(),
        ));
    }

    let user = Users::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("user"))?;
    if user.role == Role::Admin && user.is_active {
        assert_not_last_admin(&state, id).await?;
    }

    Users::delete_by_id(id).exec(&state.db).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_logins(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<Uuid>,
    Query(params): Query<LoginQuery>,
) -> AppResult<Json<Page<LoginEventResponse>>> {
    let page = page_of(params.page);
    let per_page = per_page_of(params.per_page);

    let paginator = LoginEvents::find()
        .filter(login_events::Column::UserId.eq(id))
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

async fn list_fingerprints(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<FingerprintResponse>>> {
    let items = Fingerprints::find()
        .filter(fingerprints::Column::UserId.eq(id))
        .order_by_desc(fingerprints::Column::CreatedAt)
        .all(&state.db)
        .await?;
    Ok(Json(
        items.into_iter().map(FingerprintResponse::from).collect(),
    ))
}

async fn delete_fingerprint(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let result = Fingerprints::delete_by_id(id).exec(&state.db).await?;
    if result.rows_affected == 0 {
        return Err(AppError::NotFound("fingerprint"));
    }
    Ok(StatusCode::NO_CONTENT)
}
