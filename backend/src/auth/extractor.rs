use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use uuid::Uuid;

use crate::{entities::sea_orm_active_enums::Role, error::AppError, state::AppState};

/// An authenticated human user, resolved from a `Bearer` JWT.
pub struct AuthUser {
    pub user_id: Uuid,
    pub role: Role,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| AppError::Unauthorized)?;
        let claims = state
            .jwt
            .verify(bearer.token())
            .map_err(|_| AppError::Unauthorized)?;
        Ok(Self {
            user_id: claims.sub,
            role: claims.role,
        })
    }
}

/// Requires the caller to be an admin.
pub struct AdminUser(pub AuthUser);

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;
        match user.role {
            Role::Admin => Ok(Self(user)),
            _ => Err(AppError::Forbidden),
        }
    }
}

/// Requires the caller to be faculty or admin (the custodian audience).
pub struct FacultyOrAdmin(pub AuthUser);

impl FromRequestParts<AppState> for FacultyOrAdmin {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;
        match user.role {
            Role::Faculty | Role::Admin => Ok(Self(user)),
            Role::Student => Err(AppError::Forbidden),
        }
    }
}
