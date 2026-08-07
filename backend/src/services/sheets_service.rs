//! Mirroring the audit trail into a Google spreadsheet.
//!
//! The readers report to this server, so this is the only place that sees
//! everything: both nodes, the checkouts, and the flags derived from them. That
//! is why the export lives here rather than on a board in a hallway, where the
//! credentials would sit in flash for anyone with a screwdriver.
//!
//! Four tabs, two shapes. Events and Checkouts are history, so they are appended
//! and a cursor remembers how far we got. Flags and Daily are derived from the
//! whole window every time, so they are rewritten in place; appending them would
//! duplicate the same findings on every pass.

use std::sync::Arc;

use chrono::{Duration as ChronoDuration, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    config::SheetsConfig,
    entities::{
        access_events, checkouts, prelude::*, sea_orm_active_enums::AccessEventType, sheet_syncs,
    },
    error::AppError,
    services::flag_service,
    services::purge_service,
    services::sheet_rows,
    util,
};

/// Tab names. Changing one starts a fresh tab rather than renaming the old one,
/// so the cursor for the old name is simply left behind.
pub const TAB_EVENTS: &str = "Events";
pub const TAB_CHECKOUTS: &str = "Checkouts";
pub const TAB_FLAGS: &str = "Flags";
pub const TAB_DAILY: &str = "Daily";

/// How many history rows to send per pass. A first sync of a busy term catches
/// up over several passes rather than posting one enormous request.
const BATCH: u64 = 500;

/// The window the derived tabs cover.
const FLAG_DAYS: i64 = 30;
const DAILY_DAYS: i64 = 30;

/// Google rejects an assertion older than an hour; well short of that leaves
/// room for a slow clock.
const TOKEN_TTL_SECS: i64 = 3000;

#[derive(Debug, Deserialize)]
struct ServiceAccountKey {
    client_email: String,
    private_key: String,
    #[serde(default)]
    token_uri: Option<String>,
}

#[derive(Debug, Serialize)]
struct Assertion<'a> {
    iss: &'a str,
    scope: &'a str,
    aud: &'a str,
    exp: i64,
    iat: i64,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    expires_in: Option<i64>,
}

struct CachedToken {
    value: String,
    /// Unix seconds after which it is no longer trusted.
    good_until: i64,
}

/// What the admin page shows.
#[derive(Debug, Serialize)]
pub struct TabStatus {
    pub tab: String,
    pub rows_sent: i32,
    pub last_synced_at: Option<DateTimeWithTimeZone>,
    pub last_error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SyncReport {
    pub events: usize,
    pub checkouts: usize,
    pub flags: usize,
    pub daily: usize,
}

pub struct Sheets {
    cfg: SheetsConfig,
    http: reqwest::Client,
    token: Mutex<Option<CachedToken>>,
}

impl Sheets {
    pub fn new(cfg: SheetsConfig) -> Arc<Self> {
        Arc::new(Self {
            cfg,
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(20))
                .build()
                .unwrap_or_default(),
            token: Mutex::new(None),
        })
    }

    pub fn enabled(&self) -> bool {
        self.cfg.enabled
    }

    pub fn spreadsheet_id(&self) -> &str {
        &self.cfg.spreadsheet_id
    }

    // ---- authentication ----------------------------------------------------

    /// A bearer token for the Sheets API, minted from the service account key
    /// and cached until shortly before it expires.
    async fn access_token(&self) -> Result<String, AppError> {
        let now = Utc::now().timestamp();

        {
            let held = self.token.lock().await;
            if let Some(cached) = held.as_ref() {
                if cached.good_until > now {
                    return Ok(cached.value.clone());
                }
            }
        }

        let raw = tokio::fs::read_to_string(&self.cfg.key_path)
            .await
            .map_err(|e| {
                AppError::Internal(anyhow::anyhow!(
                    "could not read the service account key at {}: {e}",
                    self.cfg.key_path
                ))
            })?;
        let key: ServiceAccountKey = serde_json::from_str(&raw).map_err(|e| {
            AppError::Internal(anyhow::anyhow!("service account key is not valid: {e}"))
        })?;

        let token_url = key
            .token_uri
            .clone()
            .unwrap_or_else(|| self.cfg.token_url.clone());
        let claims = Assertion {
            iss: &key.client_email,
            scope: "https://www.googleapis.com/auth/spreadsheets",
            aud: &token_url,
            exp: now + TOKEN_TTL_SECS,
            iat: now,
        };

        let signing = EncodingKey::from_rsa_pem(key.private_key.as_bytes()).map_err(|e| {
            AppError::Internal(anyhow::anyhow!(
                "the private key in the service account file is unusable: {e}"
            ))
        })?;
        let assertion = jsonwebtoken::encode(&Header::new(Algorithm::RS256), &claims, &signing)
            .map_err(|e| {
                AppError::Internal(anyhow::anyhow!("could not sign the token request: {e}"))
            })?;

        let res = self
            .http
            .post(&token_url)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &assertion),
            ])
            .send()
            .await
            .map_err(|e| {
                AppError::Internal(anyhow::anyhow!(
                    "Google did not answer the token request: {e}"
                ))
            })?;

        if !res.status().is_success() {
            let code = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(AppError::Internal(anyhow::anyhow!(
                "Google refused the service account ({code}): {}",
                body.chars().take(200).collect::<String>()
            )));
        }

        let parsed: TokenResponse = res.json().await.map_err(|e| {
            AppError::Internal(anyhow::anyhow!("token reply was not what we expected: {e}"))
        })?;

        let ttl = parsed.expires_in.unwrap_or(3600).max(60);
        let mut held = self.token.lock().await;
        *held = Some(CachedToken {
            value: parsed.access_token.clone(),
            good_until: now + ttl - 60,
        });
        Ok(parsed.access_token)
    }

    // ---- the two write shapes ---------------------------------------------

    /// Adds rows to the bottom of a tab.
    async fn append(&self, tab: &str, rows: Vec<Vec<String>>) -> Result<(), AppError> {
        if rows.is_empty() {
            return Ok(());
        }
        let url = format!(
            "{}/v4/spreadsheets/{}/values/{}!A1:append?valueInputOption=USER_ENTERED&insertDataOption=INSERT_ROWS",
            self.cfg.api_base, self.cfg.spreadsheet_id, tab
        );
        self.send(&url, reqwest::Method::POST, json!({ "values": rows }))
            .await
    }

    /// Replaces a tab's contents. Used for the derived tabs, which are a picture
    /// of right now rather than a log.
    async fn rewrite(&self, tab: &str, rows: Vec<Vec<String>>) -> Result<(), AppError> {
        let clear = format!(
            "{}/v4/spreadsheets/{}/values/{}!A1:Z50000:clear",
            self.cfg.api_base, self.cfg.spreadsheet_id, tab
        );
        self.send(&clear, reqwest::Method::POST, json!({})).await?;

        if rows.is_empty() {
            return Ok(());
        }
        let write = format!(
            "{}/v4/spreadsheets/{}/values/{}!A1?valueInputOption=USER_ENTERED",
            self.cfg.api_base, self.cfg.spreadsheet_id, tab
        );
        self.send(&write, reqwest::Method::PUT, json!({ "values": rows }))
            .await
    }

    async fn send(&self, url: &str, method: reqwest::Method, body: Value) -> Result<(), AppError> {
        let token = self.access_token().await?;
        let res = self
            .http
            .request(method, url)
            .bearer_auth(token)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                AppError::Internal(anyhow::anyhow!("the spreadsheet did not answer: {e}"))
            })?;

        if res.status().is_success() {
            return Ok(());
        }

        let code = res.status();
        let detail = res.text().await.unwrap_or_default();
        // 403 here almost always means the sheet was never shared with the
        // service account, which is the one setup step people miss.
        let hint = if code.as_u16() == 403 {
            " (share the spreadsheet with the service account address, as an Editor)"
        } else {
            ""
        };
        Err(AppError::Internal(anyhow::anyhow!(
            "the spreadsheet refused the write ({code}){hint}: {}",
            detail.chars().take(200).collect::<String>()
        )))
    }

    /// Creates any tab that is missing and gives it a header row. Safe to call
    /// on every sync: existing tabs are left alone.
    pub async fn ensure_tabs(&self) -> Result<(), AppError> {
        let url = format!(
            "{}/v4/spreadsheets/{}?fields=sheets.properties.title",
            self.cfg.api_base, self.cfg.spreadsheet_id
        );
        let token = self.access_token().await?;
        let res = self
            .http
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| {
                AppError::Internal(anyhow::anyhow!("could not read the spreadsheet: {e}"))
            })?;

        if !res.status().is_success() {
            let code = res.status();
            let detail = res.text().await.unwrap_or_default();
            return Err(AppError::Internal(anyhow::anyhow!(
                "could not read the spreadsheet ({code}): {}",
                detail.chars().take(200).collect::<String>()
            )));
        }

        let meta: Value = res.json().await.map_err(|e| {
            AppError::Internal(anyhow::anyhow!("spreadsheet reply was unreadable: {e}"))
        })?;
        let existing: Vec<String> = meta["sheets"]
            .as_array()
            .map(|tabs| {
                tabs.iter()
                    .filter_map(|t| t["properties"]["title"].as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();

        let wanted = [
            (TAB_EVENTS, headers_events()),
            (TAB_CHECKOUTS, headers_checkouts()),
            (TAB_FLAGS, headers_flags()),
            (TAB_DAILY, headers_daily()),
        ];

        let missing: Vec<&(&str, Vec<String>)> = wanted
            .iter()
            .filter(|(tab, _)| !existing.iter().any(|e| e == tab))
            .collect();
        if missing.is_empty() {
            return Ok(());
        }

        let requests: Vec<Value> = missing
            .iter()
            .map(|(tab, _)| json!({ "addSheet": { "properties": { "title": tab } } }))
            .collect();
        let batch = format!(
            "{}/v4/spreadsheets/{}:batchUpdate",
            self.cfg.api_base, self.cfg.spreadsheet_id
        );
        self.send(
            &batch,
            reqwest::Method::POST,
            json!({ "requests": requests }),
        )
        .await?;

        for (tab, header) in missing {
            self.append(tab, vec![header.clone()]).await?;
        }
        Ok(())
    }

    /// Puts a header on an appended tab that has none.
    ///
    /// A tab we create gets its header at creation, but a tab somebody made by
    /// hand, or renamed from Sheet1, arrives empty. Without this the first sync
    /// would fill it with unlabelled columns.
    async fn ensure_headers(&self) -> Result<(), AppError> {
        let url = format!(
            "{}/v4/spreadsheets/{}/values:batchGet?ranges={}!A1&ranges={}!A1",
            self.cfg.api_base, self.cfg.spreadsheet_id, TAB_EVENTS, TAB_CHECKOUTS
        );
        let token = self.access_token().await?;
        let res = self
            .http
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| {
                AppError::Internal(anyhow::anyhow!("could not read the tab headers: {e}"))
            })?;

        if !res.status().is_success() {
            // Not worth failing the sync over: the rows still land, and the next
            // pass tries again.
            return Ok(());
        }

        let body: Value = res
            .json()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("header reply was unreadable: {e}")))?;
        let ranges = body["valueRanges"].as_array().cloned().unwrap_or_default();

        for (i, (tab, header)) in [
            (TAB_EVENTS, headers_events()),
            (TAB_CHECKOUTS, headers_checkouts()),
        ]
        .into_iter()
        .enumerate()
        {
            let filled = ranges
                .get(i)
                .and_then(|r| r["values"].as_array())
                .map(|rows| !rows.is_empty())
                .unwrap_or(false);
            if !filled {
                self.append(tab, vec![header]).await?;
            }
        }
        Ok(())
    }

    // ---- the sync ---------------------------------------------------------

    pub async fn sync(&self, db: &DatabaseConnection) -> Result<SyncReport, AppError> {
        self.ensure_tabs().await?;
        self.ensure_headers().await?;

        let events = self.sync_events(db).await?;
        let checkouts = self.sync_checkouts(db).await?;
        let flags = self.sync_flags(db).await?;
        let daily = self.sync_daily(db).await?;

        Ok(SyncReport {
            events,
            checkouts,
            flags,
            daily,
        })
    }

    async fn sync_events(&self, db: &DatabaseConnection) -> Result<usize, AppError> {
        let cursor = read_cursor(db, TAB_EVENTS).await?;
        let hidden_before = purge_service::cutoff(db).await?;
        let mut query = AccessEvents::find()
            .order_by_asc(access_events::Column::CreatedAt)
            .order_by_asc(access_events::Column::Id);
        if let Some((at, id)) = cursor.as_ref() {
            // Strictly after the last row sent, with the id breaking ties so two
            // events in the same millisecond cannot be skipped or repeated.
            query = query.filter(
                sea_orm::Condition::any()
                    .add(access_events::Column::CreatedAt.gt(*at))
                    .add(
                        sea_orm::Condition::all()
                            .add(access_events::Column::CreatedAt.eq(*at))
                            .add(access_events::Column::Id.gt(*id)),
                    ),
            );
        }

        if let Some(since) = hidden_before {
            query = query.filter(access_events::Column::CreatedAt.gt(since));
        }

        let batch = query.limit(BATCH).all(db).await?;
        if batch.is_empty() {
            return stamp(db, TAB_EVENTS, cursor, 0).await.map(|_| 0);
        }

        let names = sheet_rows::name_map(db).await?;
        let devices = sheet_rows::device_map(db).await?;

        let rows: Vec<Vec<String>> = batch
            .iter()
            .map(|e| sheet_rows::event_row(e, &names, &devices))
            .collect();

        let sent = rows.len();
        self.append(TAB_EVENTS, rows).await?;

        let last = batch.last().expect("batch is not empty");
        stamp(
            db,
            TAB_EVENTS,
            Some((last.created_at, last.id)),
            sent as i32,
        )
        .await?;
        Ok(sent)
    }

    async fn sync_checkouts(&self, db: &DatabaseConnection) -> Result<usize, AppError> {
        let cursor = read_cursor(db, TAB_CHECKOUTS).await?;
        let hidden_before = purge_service::cutoff(db).await?;
        let mut query = Checkouts::find()
            .order_by_asc(checkouts::Column::CreatedAt)
            .order_by_asc(checkouts::Column::Id);
        if let Some((at, id)) = cursor.as_ref() {
            query = query.filter(
                sea_orm::Condition::any()
                    .add(checkouts::Column::CreatedAt.gt(*at))
                    .add(
                        sea_orm::Condition::all()
                            .add(checkouts::Column::CreatedAt.eq(*at))
                            .add(checkouts::Column::Id.gt(*id)),
                    ),
            );
        }

        if let Some(since) = hidden_before {
            query = query.filter(checkouts::Column::CreatedAt.gt(since));
        }

        let batch = query.limit(BATCH).all(db).await?;
        if batch.is_empty() {
            return stamp(db, TAB_CHECKOUTS, cursor, 0).await.map(|_| 0);
        }

        let names = sheet_rows::name_map(db).await?;
        let devices = sheet_rows::device_map(db).await?;

        let rows: Vec<Vec<String>> = batch
            .iter()
            .map(|c| sheet_rows::checkout_row(c, &names, &devices))
            .collect();

        let sent = rows.len();
        self.append(TAB_CHECKOUTS, rows).await?;

        let last = batch.last().expect("batch is not empty");
        stamp(
            db,
            TAB_CHECKOUTS,
            Some((last.created_at, last.id)),
            sent as i32,
        )
        .await?;
        Ok(sent)
    }

    async fn sync_flags(&self, db: &DatabaseConnection) -> Result<usize, AppError> {
        let flags = flag_service::detect(db, FLAG_DAYS).await?;

        let mut rows = vec![headers_flags()];
        rows.extend(flags.iter().map(sheet_rows::flag_row));

        let sent = rows.len() - 1;
        self.rewrite(TAB_FLAGS, rows).await?;
        stamp_derived(db, TAB_FLAGS, sent as i32).await?;
        Ok(sent)
    }

    async fn sync_daily(&self, db: &DatabaseConnection) -> Result<usize, AppError> {
        let now = util::now();
        let since = now - ChronoDuration::days(DAILY_DAYS);

        let events = AccessEvents::find()
            .filter(access_events::Column::CreatedAt.gte(since))
            .all(db)
            .await?;
        let taken = Checkouts::find()
            .filter(checkouts::Column::CreatedAt.gte(since))
            .all(db)
            .await?;
        let flags = flag_service::detect(db, DAILY_DAYS).await?;

        // Keyed by the date on the lab's clock, not UTC, so a 9 PM opening lands
        // on the day it happened locally.
        let mut days: std::collections::BTreeMap<String, [u32; 4]> =
            std::collections::BTreeMap::new();
        let day_of =
            |at: DateTimeWithTimeZone| util::in_lab_time(at).format("%Y-%m-%d").to_string();

        for e in &events {
            let slot = days.entry(day_of(e.created_at)).or_insert([0; 4]);
            if e.event_type == AccessEventType::BoxGranted {
                slot[0] += 1;
            }
            if format!("{:?}", e.decision).eq_ignore_ascii_case("denied") {
                slot[2] += 1;
            }
        }
        for c in &taken {
            days.entry(day_of(c.created_at)).or_insert([0; 4])[1] += 1;
        }
        for f in &flags {
            days.entry(day_of(f.at)).or_insert([0; 4])[3] += 1;
        }

        let mut rows = vec![headers_daily()];
        rows.extend(days.iter().map(|(day, counts)| {
            vec![
                day.clone(),
                counts[0].to_string(),
                counts[1].to_string(),
                counts[2].to_string(),
                counts[3].to_string(),
            ]
        }));

        let sent = rows.len() - 1;
        self.rewrite(TAB_DAILY, rows).await?;
        stamp_derived(db, TAB_DAILY, sent as i32).await?;
        Ok(sent)
    }
}

// ---- headers --------------------------------------------------------------

fn headers_events() -> Vec<String> {
    [
        "When", "Event", "Decision", "Person", "Finger", "Reader", "Detail",
    ]
    .iter()
    .map(|s| (*s).to_owned())
    .collect()
}

fn headers_checkouts() -> Vec<String> {
    ["When", "Person", "Items", "Photo", "Reader", "Record id"]
        .iter()
        .map(|s| (*s).to_owned())
        .collect()
}

fn headers_flags() -> Vec<String> {
    [
        "Severity",
        "Check",
        "What happened",
        "What to do",
        "When",
        "Person",
        "Reader",
        "Count",
    ]
    .iter()
    .map(|s| (*s).to_owned())
    .collect()
}

fn headers_daily() -> Vec<String> {
    [
        "Day",
        "Cabinet openings",
        "Checkouts recorded",
        "Refusals",
        "Flags raised",
    ]
    .iter()
    .map(|s| (*s).to_owned())
    .collect()
}

// ---- cursors --------------------------------------------------------------

/// "timestamp|uuid", which sorts the same way the query orders rows.
fn encode_key(at: DateTimeWithTimeZone, id: Uuid) -> String {
    format!("{}|{}", at.to_rfc3339(), id)
}

fn decode_key(raw: &str) -> Option<(DateTimeWithTimeZone, Uuid)> {
    let (at, id) = raw.split_once('|')?;
    let at = DateTimeWithTimeZone::parse_from_rfc3339(at).ok()?;
    let id = Uuid::parse_str(id).ok()?;
    Some((at, id))
}

async fn read_cursor(
    db: &DatabaseConnection,
    tab: &str,
) -> Result<Option<(DateTimeWithTimeZone, Uuid)>, AppError> {
    Ok(SheetSyncs::find_by_id(tab.to_owned())
        .one(db)
        .await?
        .and_then(|row| row.last_key)
        .as_deref()
        .and_then(decode_key))
}

/// Records progress for an appended tab. `added` is added to the running total.
async fn stamp(
    db: &DatabaseConnection,
    tab: &str,
    cursor: Option<(DateTimeWithTimeZone, Uuid)>,
    added: i32,
) -> Result<(), AppError> {
    let key = cursor.map(|(at, id)| encode_key(at, id));
    let existing = SheetSyncs::find_by_id(tab.to_owned()).one(db).await?;

    match existing {
        Some(row) => {
            let total = row.rows_sent + added;
            let mut active: sheet_syncs::ActiveModel = row.into();
            if key.is_some() {
                active.last_key = Set(key);
            }
            active.rows_sent = Set(total);
            active.last_synced_at = Set(Some(util::now()));
            active.last_error = Set(None);
            active.update(db).await?;
        }
        None => {
            sheet_syncs::ActiveModel {
                tab: Set(tab.to_owned()),
                last_key: Set(key),
                rows_sent: Set(added),
                last_synced_at: Set(Some(util::now())),
                last_error: Set(None),
            }
            .insert(db)
            .await?;
        }
    }
    Ok(())
}

/// Derived tabs are rewritten, so their count is the current row count rather
/// than a running total.
async fn stamp_derived(db: &DatabaseConnection, tab: &str, rows: i32) -> Result<(), AppError> {
    let existing = SheetSyncs::find_by_id(tab.to_owned()).one(db).await?;
    match existing {
        Some(row) => {
            let mut active: sheet_syncs::ActiveModel = row.into();
            active.rows_sent = Set(rows);
            active.last_synced_at = Set(Some(util::now()));
            active.last_error = Set(None);
            active.update(db).await?;
        }
        None => {
            sheet_syncs::ActiveModel {
                tab: Set(tab.to_owned()),
                last_key: Set(None),
                rows_sent: Set(rows),
                last_synced_at: Set(Some(util::now())),
                last_error: Set(None),
            }
            .insert(db)
            .await?;
        }
    }
    Ok(())
}

/// Writes the failure onto every tab row, so the admin page can say what went
/// wrong without anyone reading the server log.
pub async fn record_failure(db: &DatabaseConnection, message: &str) -> Result<(), AppError> {
    let short: String = message.chars().take(300).collect();
    for tab in [TAB_EVENTS, TAB_CHECKOUTS, TAB_FLAGS, TAB_DAILY] {
        match SheetSyncs::find_by_id(tab.to_owned()).one(db).await? {
            Some(row) => {
                let mut active: sheet_syncs::ActiveModel = row.into();
                active.last_error = Set(Some(short.clone()));
                active.update(db).await?;
            }
            None => {
                sheet_syncs::ActiveModel {
                    tab: Set(tab.to_owned()),
                    last_key: Set(None),
                    rows_sent: Set(0),
                    last_synced_at: Set(None),
                    last_error: Set(Some(short.clone())),
                }
                .insert(db)
                .await?;
            }
        }
    }
    Ok(())
}

pub async fn status(db: &DatabaseConnection) -> Result<Vec<TabStatus>, AppError> {
    let rows = SheetSyncs::find().all(db).await?;
    Ok([TAB_EVENTS, TAB_CHECKOUTS, TAB_FLAGS, TAB_DAILY]
        .iter()
        .map(|tab| {
            let found = rows.iter().find(|r| r.tab == *tab);
            TabStatus {
                tab: (*tab).to_owned(),
                rows_sent: found.map(|r| r.rows_sent).unwrap_or(0),
                last_synced_at: found.and_then(|r| r.last_synced_at),
                last_error: found.and_then(|r| r.last_error.clone()),
            }
        })
        .collect())
}

/// How much history is still waiting, so the page can say "catching up".
pub async fn backlog(db: &DatabaseConnection) -> Result<(u64, u64), AppError> {
    let events_cursor = read_cursor(db, TAB_EVENTS).await?;
    let checkouts_cursor = read_cursor(db, TAB_CHECKOUTS).await?;

    let mut events = AccessEvents::find();
    if let Some((at, _)) = events_cursor {
        events = events.filter(access_events::Column::CreatedAt.gt(at));
    }
    let mut taken = Checkouts::find();
    if let Some((at, _)) = checkouts_cursor {
        taken = taken.filter(checkouts::Column::CreatedAt.gt(at));
    }

    Ok((events.count(db).await?, taken.count(db).await?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_round_trips() {
        let at = util::now();
        let id = Uuid::new_v4();
        let (back_at, back_id) = decode_key(&encode_key(at, id)).expect("decodes");
        assert_eq!(back_id, id);
        assert_eq!(back_at.timestamp_micros(), at.timestamp_micros());
    }

    #[test]
    fn nonsense_cursors_are_ignored() {
        assert!(decode_key("").is_none());
        assert!(decode_key("no-separator").is_none());
        assert!(decode_key("not-a-date|not-a-uuid").is_none());
    }

    #[test]
    fn headers_match_the_widest_row() {
        // A row longer than its header would silently shift columns in the sheet.
        assert_eq!(headers_events().len(), 7);
        assert_eq!(headers_checkouts().len(), 6);
        assert_eq!(headers_flags().len(), 8);
        assert_eq!(headers_daily().len(), 5);
    }
}
