pub mod auth;
pub mod checkouts;
pub mod device_protocol;
pub mod devices;
pub mod enrollment;
pub mod flags;
pub mod logs;
pub mod sessions;
pub mod sheets;
pub mod sms;
pub mod users;

use axum::{
    Router,
    http::{HeaderName, Method, header},
};
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

use crate::{config::Config, state::AppState};

/// Compose every feature router under `/api`, serve uploaded photos at
/// `/uploads`, and apply CORS + tracing.
pub fn app_router(state: AppState, cfg: &Config) -> Router {
    let api = Router::new()
        .merge(auth::router())
        .merge(users::router())
        .merge(devices::router())
        .merge(device_protocol::router())
        .merge(enrollment::router())
        .merge(sessions::router())
        .merge(checkouts::router())
        .merge(logs::router())
        .merge(flags::router())
        .merge(sheets::router())
        .merge(sms::router());

    let cors = CorsLayer::new()
        .allow_origin(
            cfg.cors_origin
                .parse::<header::HeaderValue>()
                .expect("CORS_ORIGIN must be a valid origin"),
        )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            HeaderName::from_static("x-device-id"),
            HeaderName::from_static("x-device-secret"),
        ]);

    Router::new()
        .nest("/api", api)
        .nest_service("/uploads", ServeDir::new(cfg.upload_dir.clone()))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
