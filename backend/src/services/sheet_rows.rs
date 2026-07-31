//! What a row of the sheet is, in one place.
//!
//! The website renders these as a grid and, if anybody turns the Google export
//! on, the same builders fill the spreadsheet. Defining a row once is what keeps
//! the two identical, so what you read on screen is what lands in the sheet.

use std::collections::HashMap;

use chrono::Duration as ChronoDuration;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    entities::{
        access_events, checkouts,
        prelude::*,
        sea_orm_active_enums::{AccessEventType, EventDecision},
        users,
    },
    error::AppError,
    pagination::Page,
    services::flag_service,
    util,
};

/// The four views, and the only names the API accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Tab {
    Events,
    Checkouts,
    Flags,
    Daily,
}

impl Tab {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "events" => Some(Self::Events),
            "checkouts" => Some(Self::Checkouts),
            "flags" => Some(Self::Flags),
            "daily" => Some(Self::Daily),
            _ => None,
        }
    }

    /// The tab name as it appears in a spreadsheet.
    pub fn title(self) -> &'static str {
        match self {
            Self::Events => "Events",
            Self::Checkouts => "Checkouts",
            Self::Flags => "Flags",
            Self::Daily => "Daily",
        }
    }

    pub fn headers(self) -> Vec<String> {
        let cols: &[&str] = match self {
            Self::Events => &[
                "When", "Event", "Decision", "Person", "Finger", "Reader", "Detail",
            ],
            Self::Checkouts => &["When", "Person", "Items", "Photo", "Reader", "Record id"],
            Self::Flags => &[
                "Severity",
                "Check",
                "What happened",
                "What to do",
                "When",
                "Person",
                "Reader",
                "Count",
            ],
            Self::Daily => &[
                "Day",
                "Cabinet openings",
                "Checkouts recorded",
                "Refusals",
                "Flags raised",
            ],
        };
        cols.iter().map(|c| (*c).to_owned()).collect()
    }

    /// One line on what the tab holds, for the page that shows it.
    pub fn summary(self) -> &'static str {
        match self {
            Self::Events => "Every scan, unlock and refusal the readers reported, newest last.",
            Self::Checkouts => "What each person recorded taking, and whether they added a photo.",
            Self::Flags => "Anomalies over the last 30 days, worst first.",
            Self::Daily => "One row per day for the last 30 days.",
        }
    }

    /// True for the tabs that are a growing log rather than a derived summary.
    pub fn is_history(self) -> bool {
        matches!(self, Self::Events | Self::Checkouts)
    }

    pub fn all() -> [Tab; 4] {
        [Tab::Events, Tab::Checkouts, Tab::Flags, Tab::Daily]
    }
}

/// The window the derived tabs cover.
pub const DERIVED_DAYS: i64 = 30;

/// A timestamp a person can read and a spreadsheet still recognises as a date.
/// The raw value carries nanoseconds, which fills the column with noise.
fn stamp(at: DateTimeWithTimeZone) -> String {
    util::in_lab_time(at)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

/// Plain English for an event, because this sheet is read by people. The machine
/// name is still what the search matches, so both ways of looking work.
fn event_label(kind: &AccessEventType) -> &'static str {
    match kind {
        AccessEventType::DoorGranted => "Door opened",
        AccessEventType::DoorDeniedUnregistered => "Door denied: finger not registered",
        AccessEventType::DoorDeniedInactive => "Door denied: account disabled",
        AccessEventType::BoxGranted => "Cabinet unlocked",
        AccessEventType::BoxDeniedNoSession => "Cabinet denied: no door scan first",
        AccessEventType::BoxDeniedUnregistered => "Cabinet denied: finger not registered",
        AccessEventType::BoxDeniedSessionExpired => "Cabinet denied: window expired",
        AccessEventType::EnrollCodeIssued => "Enrollment code issued",
        AccessEventType::EnrollBound => "Fingerprint bound",
        AccessEventType::EnrollFailed => "Enrollment failed",
        AccessEventType::CheckoutCodeRejected => "Checkout code refused",
    }
}

fn decision_label(decision: &EventDecision) -> &'static str {
    match decision {
        EventDecision::Granted => "Granted",
        EventDecision::Denied => "Denied",
        EventDecision::Info => "Info",
    }
}

#[derive(Debug, Serialize)]
pub struct SheetPage {
    pub tab: Tab,
    pub title: String,
    pub summary: String,
    pub headers: Vec<String>,
    pub rows: Page<Vec<String>>,
}

/// A page of rows for one tab. Searching and paging happen here, in the
/// database where they belong, so the browser never holds the whole table.
pub async fn page(
    db: &DatabaseConnection,
    tab: Tab,
    page_no: u64,
    per_page: u64,
    query: Option<String>,
) -> Result<SheetPage, AppError> {
    let term = query.map(|q| q.trim().to_owned()).filter(|q| !q.is_empty());

    let rows = match tab {
        Tab::Events => events_page(db, page_no, per_page, term).await?,
        Tab::Checkouts => checkouts_page(db, page_no, per_page, term).await?,
        Tab::Flags => flags_page(db, page_no, per_page, term).await?,
        Tab::Daily => daily_page(db, page_no, per_page, term).await?,
    };

    Ok(SheetPage {
        tab,
        title: tab.title().to_owned(),
        summary: tab.summary().to_owned(),
        headers: tab.headers(),
        rows,
    })
}

// ---- rows -----------------------------------------------------------------

pub fn event_row(
    e: &access_events::Model,
    names: &HashMap<Uuid, String>,
    devices: &HashMap<Uuid, String>,
) -> Vec<String> {
    vec![
        stamp(e.created_at),
        event_label(&e.event_type).to_owned(),
        decision_label(&e.decision).to_owned(),
        e.user_id
            .and_then(|id| names.get(&id).cloned())
            .unwrap_or_default(),
        e.finger_token.clone().unwrap_or_default(),
        e.device_id
            .and_then(|id| devices.get(&id).cloned())
            .unwrap_or_default(),
        e.message.clone().unwrap_or_default(),
    ]
}

pub fn checkout_row(
    c: &checkouts::Model,
    names: &HashMap<Uuid, String>,
    devices: &HashMap<Uuid, String>,
) -> Vec<String> {
    vec![
        stamp(c.created_at),
        names.get(&c.user_id).cloned().unwrap_or_default(),
        c.note.clone(),
        if c.photo_path.is_some() { "yes" } else { "no" }.to_owned(),
        c.box_device_id
            .and_then(|id| devices.get(&id).cloned())
            .unwrap_or_default(),
        c.id.to_string(),
    ]
}

pub fn flag_row(f: &flag_service::Flag) -> Vec<String> {
    vec![
        format!("{:?}", f.severity),
        f.kind.to_owned(),
        f.title.clone(),
        f.detail.clone(),
        stamp(f.at),
        f.person.clone().unwrap_or_default(),
        f.device.clone().unwrap_or_default(),
        f.count.to_string(),
    ]
}

/// Every day in the window, oldest first, counted on the lab's clock so a 9 PM
/// opening lands on the day it happened locally.
pub async fn daily_rows(db: &DatabaseConnection, days: i64) -> Result<Vec<Vec<String>>, AppError> {
    let since = util::now() - ChronoDuration::days(days);

    let events = AccessEvents::find()
        .filter(access_events::Column::CreatedAt.gte(since))
        .all(db)
        .await?;
    let taken = Checkouts::find()
        .filter(checkouts::Column::CreatedAt.gte(since))
        .all(db)
        .await?;
    let flags = flag_service::detect(db, days).await?;

    let day_of = |at: DateTimeWithTimeZone| util::in_lab_time(at).format("%Y-%m-%d").to_string();
    let mut days_map: std::collections::BTreeMap<String, [u32; 4]> =
        std::collections::BTreeMap::new();

    for e in &events {
        let slot = days_map.entry(day_of(e.created_at)).or_insert([0; 4]);
        if e.event_type == AccessEventType::BoxGranted {
            slot[0] += 1;
        }
        if e.decision == EventDecision::Denied {
            slot[2] += 1;
        }
    }
    for c in &taken {
        days_map.entry(day_of(c.created_at)).or_insert([0; 4])[1] += 1;
    }
    for f in &flags {
        days_map.entry(day_of(f.at)).or_insert([0; 4])[3] += 1;
    }

    Ok(days_map
        .into_iter()
        .map(|(day, counts)| {
            vec![
                day,
                counts[0].to_string(),
                counts[1].to_string(),
                counts[2].to_string(),
                counts[3].to_string(),
            ]
        })
        .collect())
}

// ---- per-tab paging -------------------------------------------------------

async fn events_page(
    db: &DatabaseConnection,
    page_no: u64,
    per_page: u64,
    term: Option<String>,
) -> Result<Page<Vec<String>>, AppError> {
    let mut query = AccessEvents::find();

    if let Some(term) = term.as_ref() {
        // Searching a person means searching the users table first, since the
        // event only carries their id.
        let people: Vec<Uuid> = Users::find()
            .filter(
                Condition::any()
                    .add(users::Column::FirstName.contains(term))
                    .add(users::Column::MiddleName.contains(term))
                    .add(users::Column::LastName.contains(term))
                    .add(users::Column::Username.contains(term)),
            )
            .all(db)
            .await?
            .into_iter()
            .map(|u| u.id)
            .collect();

        let mut any = Condition::any()
            .add(access_events::Column::FingerToken.contains(term))
            .add(access_events::Column::Message.contains(term))
            .add(access_events::Column::EventType.contains(term));
        if !people.is_empty() {
            any = any.add(access_events::Column::UserId.is_in(people));
        }
        query = query.filter(any);
    }

    let total = query.clone().count(db).await?;
    let batch = query
        .order_by_asc(access_events::Column::CreatedAt)
        .order_by_asc(access_events::Column::Id)
        .offset((page_no - 1) * per_page)
        .limit(per_page)
        .all(db)
        .await?;

    let names = name_map(db).await?;
    let devices = device_map(db).await?;
    let rows = batch
        .iter()
        .map(|e| event_row(e, &names, &devices))
        .collect();
    Ok(Page::new(rows, total, page_no, per_page))
}

async fn checkouts_page(
    db: &DatabaseConnection,
    page_no: u64,
    per_page: u64,
    term: Option<String>,
) -> Result<Page<Vec<String>>, AppError> {
    let mut query = Checkouts::find();

    if let Some(term) = term.as_ref() {
        let people: Vec<Uuid> = Users::find()
            .filter(
                Condition::any()
                    .add(users::Column::FirstName.contains(term))
                    .add(users::Column::MiddleName.contains(term))
                    .add(users::Column::LastName.contains(term))
                    .add(users::Column::Username.contains(term)),
            )
            .all(db)
            .await?
            .into_iter()
            .map(|u| u.id)
            .collect();

        let mut any = Condition::any().add(checkouts::Column::Note.contains(term));
        if !people.is_empty() {
            any = any.add(checkouts::Column::UserId.is_in(people));
        }
        query = query.filter(any);
    }

    let total = query.clone().count(db).await?;
    let batch = query
        .order_by_asc(checkouts::Column::CreatedAt)
        .order_by_asc(checkouts::Column::Id)
        .offset((page_no - 1) * per_page)
        .limit(per_page)
        .all(db)
        .await?;

    let names = name_map(db).await?;
    let devices = device_map(db).await?;
    let rows = batch
        .iter()
        .map(|c| checkout_row(c, &names, &devices))
        .collect();
    Ok(Page::new(rows, total, page_no, per_page))
}

/// Flags are derived, so the whole window is built and then cut. It is bounded
/// by 30 days of activity, and the slicing still happens here rather than in the
/// browser.
async fn flags_page(
    db: &DatabaseConnection,
    page_no: u64,
    per_page: u64,
    term: Option<String>,
) -> Result<Page<Vec<String>>, AppError> {
    let all: Vec<Vec<String>> = flag_service::detect(db, DERIVED_DAYS)
        .await?
        .iter()
        .map(flag_row)
        .collect();
    Ok(slice(all, page_no, per_page, term))
}

async fn daily_page(
    db: &DatabaseConnection,
    page_no: u64,
    per_page: u64,
    term: Option<String>,
) -> Result<Page<Vec<String>>, AppError> {
    let all = daily_rows(db, DERIVED_DAYS).await?;
    Ok(slice(all, page_no, per_page, term))
}

/// Filters and cuts an already-built set of rows.
fn slice(
    rows: Vec<Vec<String>>,
    page_no: u64,
    per_page: u64,
    term: Option<String>,
) -> Page<Vec<String>> {
    let matched: Vec<Vec<String>> = match term {
        Some(term) => {
            let needle = term.to_lowercase();
            rows.into_iter()
                .filter(|row| row.iter().any(|cell| cell.to_lowercase().contains(&needle)))
                .collect()
        }
        None => rows,
    };

    let total = matched.len() as u64;
    let start = ((page_no - 1) * per_page) as usize;
    let page_rows = matched
        .into_iter()
        .skip(start)
        .take(per_page as usize)
        .collect();
    Page::new(page_rows, total, page_no, per_page)
}

// ---- lookups --------------------------------------------------------------

pub async fn name_map(db: &DatabaseConnection) -> Result<HashMap<Uuid, String>, AppError> {
    Ok(Users::find()
        .all(db)
        .await?
        .into_iter()
        .map(|u| (u.id, u.display_name()))
        .collect())
}

pub async fn device_map(db: &DatabaseConnection) -> Result<HashMap<Uuid, String>, AppError> {
    Ok(Devices::find()
        .all(db)
        .await?
        .into_iter()
        .map(|d| (d.id, d.name))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_names_round_trip() {
        for tab in Tab::all() {
            assert_eq!(Tab::parse(tab.title()), Some(tab));
        }
        assert!(Tab::parse("Sheet1").is_none());
    }

    #[test]
    fn every_tab_has_headers() {
        for tab in Tab::all() {
            assert!(!tab.headers().is_empty());
            assert!(!tab.summary().is_empty());
        }
    }

    #[test]
    fn slicing_pages_and_filters() {
        let rows: Vec<Vec<String>> = (1..=25)
            .map(|i| vec![format!("row {i}"), format!("value {}", i % 3)])
            .collect();

        let first = slice(rows.clone(), 1, 10, None);
        assert_eq!(first.items.len(), 10);
        assert_eq!(first.total, 25);
        assert_eq!(first.pages, 3);

        let last = slice(rows.clone(), 3, 10, None);
        assert_eq!(last.items.len(), 5);

        // A page past the end is empty rather than an error.
        assert!(slice(rows.clone(), 9, 10, None).items.is_empty());

        let filtered = slice(rows, 1, 10, Some("value 0".to_owned()));
        assert_eq!(filtered.total, 8);
    }
}
