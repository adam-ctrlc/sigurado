//! The sheet: the same rows a spreadsheet would hold, served from our own data.
//!
//! Nothing external is involved. The readers post to this server, the server
//! shapes the rows, and the website renders them as a grid. The Google mirror is
//! an optional extra on top, and its status is reported here too.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, header},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{
    auth::extractor::AdminUser,
    error::{AppError, AppResult},
    pagination::{page_of, per_page_of},
    services::{
        sheet_rows::{self, SheetPage, Tab},
        sheets_service::{self, SyncReport, TabStatus},
    },
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct SheetQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    /// Free text over every column, matched in the database.
    pub q: Option<String>,
}

/// One entry per tab, for the strip along the bottom of the grid.
#[derive(Debug, Serialize)]
pub struct TabInfo {
    pub tab: Tab,
    pub title: String,
    pub summary: String,
    pub columns: usize,
    /// True for a growing log, false for a derived summary.
    pub history: bool,
}

#[derive(Debug, Serialize)]
pub struct SheetIndex {
    pub tabs: Vec<TabInfo>,
    /// Present only when the optional Google mirror is switched on.
    pub google: Option<GoogleStatus>,
}

#[derive(Debug, Serialize)]
pub struct GoogleStatus {
    pub spreadsheet_id: String,
    pub every_secs: u64,
    pub tabs: Vec<TabStatus>,
    pub events_waiting: u64,
    pub checkouts_waiting: u64,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/sheet", get(index))
        .route("/admin/sheet/{tab}", get(rows))
        .route("/admin/sheet/{tab}/csv", get(csv))
        .route("/admin/sheets/sync", post(sync_now))
}

fn tab_or_404(raw: &str) -> Result<Tab, AppError> {
    Tab::parse(raw).ok_or(AppError::NotFound("sheet tab"))
}

async fn index(State(state): State<AppState>, _who: AdminUser) -> AppResult<Json<SheetIndex>> {
    let tabs = Tab::all()
        .into_iter()
        .map(|tab| TabInfo {
            tab,
            title: tab.title().to_owned(),
            summary: tab.summary().to_owned(),
            columns: tab.headers().len(),
            history: tab.is_history(),
        })
        .collect();

    let google = if state.sheets.enabled() {
        let (events_waiting, checkouts_waiting) = sheets_service::backlog(&state.db).await?;
        Some(GoogleStatus {
            spreadsheet_id: state.sheets.spreadsheet_id().to_owned(),
            every_secs: state.cfg.sheets.every.as_secs(),
            tabs: sheets_service::status(&state.db).await?,
            events_waiting,
            checkouts_waiting,
        })
    } else {
        None
    };

    Ok(Json(SheetIndex { tabs, google }))
}

async fn rows(
    State(state): State<AppState>,
    _who: AdminUser,
    Path(tab): Path<String>,
    Query(params): Query<SheetQuery>,
) -> AppResult<Json<SheetPage>> {
    let tab = tab_or_404(&tab)?;
    let page = sheet_rows::page(
        &state.db,
        tab,
        page_of(params.page),
        per_page_of(params.per_page),
        params.q,
    )
    .await?;
    Ok(Json(page))
}

/// The whole tab as a CSV file, for handing to somebody who wants a spreadsheet
/// of their own. Capped so one click cannot ask for an unbounded export.
async fn csv(
    State(state): State<AppState>,
    _who: AdminUser,
    Path(tab): Path<String>,
    Query(params): Query<SheetQuery>,
) -> AppResult<(HeaderMap, String)> {
    const MAX_ROWS: u64 = 10_000;

    let tab = tab_or_404(&tab)?;
    let page = sheet_rows::page(&state.db, tab, 1, MAX_ROWS, params.q).await?;

    let mut out = String::new();
    push_csv_row(&mut out, &page.headers);
    for row in &page.rows.items {
        push_csv_row(&mut out, row);
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    let filename = format!("sigurado-{}.csv", tab.title().to_lowercase());
    if let Ok(value) = HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")) {
        headers.insert(header::CONTENT_DISPOSITION, value);
    }
    Ok((headers, out))
}

/// Quotes a cell only when it has to be, and doubles any quote inside it.
fn push_csv_row(out: &mut String, row: &[String]) {
    for (i, cell) in row.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        if cell.contains([',', '"', '\n', '\r']) {
            out.push('"');
            out.push_str(&cell.replace('"', "\"\""));
            out.push('"');
        } else {
            out.push_str(cell);
        }
    }
    out.push_str("\r\n");
}

/// Pushes the optional Google mirror now. Only useful when it is configured.
async fn sync_now(State(state): State<AppState>, _who: AdminUser) -> AppResult<Json<SyncReport>> {
    if !state.sheets.enabled() {
        return Err(AppError::Validation(
            "the Google mirror is not configured; the sheet on this page needs nothing set up"
                .to_owned(),
        ));
    }

    match state.sheets.sync(&state.db).await {
        Ok(report) => Ok(Json(report)),
        Err(err) => {
            let _ = sheets_service::record_failure(&state.db, &err.to_string()).await;
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_quotes_only_what_needs_it() {
        let mut out = String::new();
        push_csv_row(
            &mut out,
            &[
                "plain".to_owned(),
                "has, comma".to_owned(),
                "has \"quotes\"".to_owned(),
                "two\nlines".to_owned(),
            ],
        );
        assert_eq!(
            out,
            "plain,\"has, comma\",\"has \"\"quotes\"\"\",\"two\nlines\"\r\n"
        );
    }

    #[test]
    fn unknown_tabs_are_not_found() {
        assert!(tab_or_404("Sheet1").is_err());
        assert!(tab_or_404("events").is_ok());
    }
}
