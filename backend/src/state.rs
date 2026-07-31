use std::sync::Arc;

use sea_orm::DatabaseConnection;
use tokio::sync::broadcast;

use crate::{
    auth::jwt::JwtKeys, config::Config, services::event_service::EventEnvelope,
    services::sheets_service::Sheets,
};

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub cfg: Arc<Config>,
    pub jwt: Arc<JwtKeys>,
    pub events: broadcast::Sender<EventEnvelope>,
    /// Holds the cached Google token, so a sync does not mint a new one each time.
    pub sheets: Arc<Sheets>,
}
