//! The flags view: anomalies the audit trail can be asked about.

use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use serde::{Deserialize, Serialize};

use crate::{
    auth::extractor::FacultyOrAdmin,
    error::AppResult,
    services::flag_service::{self, DEFAULT_DAYS, Flag, FlagSummary, Severity},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct FlagQuery {
    /// How far back to look. Defaults to a fortnight.
    pub days: Option<i64>,
    /// Narrow to one severity: `high`, `medium`, or `low`.
    pub severity: Option<String>,
    /// Narrow to one kind of flag, by its machine name.
    pub kind: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FlagsResponse {
    pub summary: FlagSummary,
    pub flags: Vec<Flag>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/flags", get(list_flags))
}

async fn list_flags(
    State(state): State<AppState>,
    _who: FacultyOrAdmin,
    Query(params): Query<FlagQuery>,
) -> AppResult<Json<FlagsResponse>> {
    let days = flag_service::clamp_days(params.days.unwrap_or(DEFAULT_DAYS));
    let all = flag_service::detect(&state.db, days).await?;

    // The summary counts the whole window even when the list is narrowed, so the
    // severity filter can show what it would reveal before it is clicked.
    let summary = flag_service::summarize(&all, days);

    let wanted = params.severity.as_deref().and_then(Severity::parse);
    let kind = params
        .kind
        .as_deref()
        .map(str::trim)
        .filter(|k| !k.is_empty());
    let flags = all
        .into_iter()
        .filter(|f| wanted.is_none_or(|s| f.severity == s))
        .filter(|f| kind.is_none_or(|k| f.kind == k))
        .collect();

    Ok(Json(FlagsResponse { summary, flags }))
}
