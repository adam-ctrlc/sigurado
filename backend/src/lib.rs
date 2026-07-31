pub mod auth;
pub mod config;
pub mod dto;
pub mod entities;
pub mod error;
pub mod pagination;
pub mod routes;
pub mod services;
pub mod state;
pub mod util;

use std::sync::Arc;

use anyhow::Context;
use sea_orm::Database;
use tokio::sync::broadcast;

use crate::{
    auth::jwt::JwtKeys, config::Config, services::sheets_service::Sheets, state::AppState,
};

/// Build shared application state from configuration and a live DB connection.
pub async fn build_state(cfg: Config) -> anyhow::Result<AppState> {
    let db = Database::connect(&cfg.database_url)
        .await
        .context("connecting to the database")?;
    let (events, _rx) = broadcast::channel(256);
    let jwt = Arc::new(JwtKeys::new(&cfg.jwt_secret));
    let sheets = Sheets::new(cfg.sheets.clone());
    Ok(AppState {
        db,
        cfg: Arc::new(cfg),
        jwt,
        events,
        sheets,
    })
}

/// Mirrors the trail into the spreadsheet on a timer. A failure is recorded and
/// the loop carries on: the export falling over must never take the readers with
/// it, and the next pass picks up where the cursor left off.
fn spawn_sheets_sync(state: AppState) {
    tokio::spawn(async move {
        let every = state.cfg.sheets.every;
        // A short delay first, so the server is answering requests before it
        // starts talking to Google.
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        loop {
            match state.sheets.sync(&state.db).await {
                Ok(report) => {
                    if report.events > 0 || report.checkouts > 0 {
                        tracing::info!(
                            "sheets: {} events, {} checkouts, {} flags, {} days",
                            report.events,
                            report.checkouts,
                            report.flags,
                            report.daily
                        );
                    }
                }
                Err(err) => {
                    tracing::warn!("sheets sync failed: {err}");
                    let _ =
                        services::sheets_service::record_failure(&state.db, &err.to_string()).await;
                }
            }
            tokio::time::sleep(every).await;
        }
    });
}

/// Start the HTTP server and serve until shutdown.
pub async fn run() -> anyhow::Result<()> {
    let cfg = Config::from_env()?;
    let state = build_state(cfg.clone()).await?;
    if state.sheets.enabled() {
        spawn_sheets_sync(state.clone());
    }
    let app = routes::app_router(state, &cfg);

    let listener = tokio::net::TcpListener::bind(&cfg.bind_addr)
        .await
        .with_context(|| format!("binding to {}", cfg.bind_addr))?;
    tracing::info!("Sigurado API listening on http://{}", cfg.bind_addr);

    // with_connect_info so sign-in attempts can be logged against a peer address.
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .context("running the HTTP server")?;
    Ok(())
}
