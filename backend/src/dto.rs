//! Small shared response shapes reused across services and routes.

use serde::Serialize;
use uuid::Uuid;

use crate::entities::{sea_orm_active_enums::Role, users};

#[derive(Debug, Clone, Serialize)]
pub struct UserBrief {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub role: Role,
}

impl From<&users::Model> for UserBrief {
    fn from(u: &users::Model) -> Self {
        Self {
            id: u.id,
            username: u.username.clone(),
            display_name: u.display_name(),
            role: u.role,
        }
    }
}

impl From<users::Model> for UserBrief {
    fn from(u: users::Model) -> Self {
        Self::from(&u)
    }
}
