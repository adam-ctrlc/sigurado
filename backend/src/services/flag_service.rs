//! Anomaly detection over the audit trail.
//!
//! The readers record facts; this module reads those facts back and asks the
//! awkward questions. Nothing here blocks anybody: a flag is a prompt for a
//! human to look, which is the right posture for a lab where the most common
//! cause of a flag is forgetfulness rather than theft.
//!
//! Every check is derived at read time from tables we already keep, so turning
//! detection on cannot corrupt anything and a flag disappears the moment the
//! underlying record is resolved.

use std::collections::HashMap;

use chrono::Timelike;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    entities::{
        access_events, access_sessions, checkout_codes, devices, login_events,
        prelude::*,
        sea_orm_active_enums::{AccessEventType, SessionStatus},
        users,
    },
    error::AppError,
    services::purge_service,
    util,
};

/// How loud a flag is. Only `High` deserves interrupting someone's day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    High,
    Medium,
    Low,
}

impl Severity {
    /// Read a severity out of a query string. Unknown words mean "no filter"
    /// rather than an error, so a stale bookmark still shows something.
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "high" => Some(Self::High),
            "medium" => Some(Self::Medium),
            "low" => Some(Self::Low),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Flag {
    /// Stable machine name, so the UI can group and explain.
    pub kind: &'static str,
    pub severity: Severity,
    /// One line stating what happened.
    pub title: String,
    /// Why it matters and what to do about it.
    pub detail: String,
    /// When the thing being flagged happened.
    pub at: DateTimeWithTimeZone,
    pub person: Option<String>,
    pub device: Option<String>,
    /// How many occurrences this flag summarises.
    pub count: u32,
}

/// Lab hours. Anything at the cabinet outside this window is worth a look, and
/// this is deliberately generous so ordinary late work does not cry wolf.
const DAY_START_HOUR: u32 = 6;
const DAY_END_HOUR: u32 = 20;

/// A window long enough to catch a pattern, short enough to stay relevant.
pub const DEFAULT_DAYS: i64 = 14;

/// Repeated refusals only count as probing above these thresholds.
const PROBE_THRESHOLD: u32 = 3;
const SIGN_IN_THRESHOLD: u32 = 5;
const OPENINGS_PER_SESSION: u32 = 3;

struct Context {
    users: HashMap<Uuid, String>,
    devices: HashMap<Uuid, String>,
}

impl Context {
    fn person(&self, id: Option<Uuid>) -> Option<String> {
        id.and_then(|id| self.users.get(&id).cloned())
    }

    fn device(&self, id: Option<Uuid>) -> Option<String> {
        id.and_then(|id| self.devices.get(&id).cloned())
    }
}

/// The window the caller actually gets. A bookmark asking for ten years should
/// answer with the longest window there is rather than an error.
pub fn clamp_days(days: i64) -> i64 {
    days.clamp(1, 90)
}

/// Everything worth a second look in the last `days` days, worst first.
pub async fn detect(db: &DatabaseConnection, days: i64) -> Result<Vec<Flag>, AppError> {
    let days = clamp_days(days);
    let now = util::now();
    let asked = now - chrono::Duration::days(days);

    // Flags are read off the history, so cleared history cannot raise them. The
    // floor is whichever is later: the window asked for, or the clearing.
    let since = match purge_service::cutoff(db).await? {
        Some(hidden) if hidden > asked => hidden,
        _ => asked,
    };

    let ctx = Context {
        users: Users::find()
            .all(db)
            .await?
            .into_iter()
            .map(|u| (u.id, u.display_name()))
            .collect(),
        devices: Devices::find()
            .all(db)
            .await?
            .into_iter()
            .map(|d| (d.id, d.name))
            .collect(),
    };

    let events = AccessEvents::find()
        .filter(access_events::Column::CreatedAt.gte(since))
        .order_by_desc(access_events::Column::CreatedAt)
        .all(db)
        .await?;

    let mut flags = Vec::new();
    flags.extend(unrecorded_openings(db, &ctx, since).await?);
    flags.extend(tailgating(&ctx, &events));
    flags.extend(code_guessing(&ctx, &events));
    flags.extend(finger_probing(&ctx, &events));
    flags.extend(after_hours(&ctx, &events));
    flags.extend(repeat_openings(&ctx, &events));
    flags.extend(disabled_account_attempts(&ctx, &events));
    flags.extend(sign_in_attempts(db, since).await?);
    flags.extend(sessions_for_disabled_users(db, &ctx).await?);
    flags.extend(silent_readers(db, &ctx, now).await?);
    flags.extend(unclaimed_enrollments(&ctx, &events));

    // Worst first, then newest, so the top of the list is always the thing to
    // read next.
    flags.sort_by(|a, b| {
        let rank = |s: Severity| match s {
            Severity::High => 0,
            Severity::Medium => 1,
            Severity::Low => 2,
        };
        rank(a.severity)
            .cmp(&rank(b.severity))
            .then(b.at.cmp(&a.at))
    });
    Ok(flags)
}

/// The one they asked about: the cabinet opened, and nothing was ever written
/// down. The code issued at the opening was never spent and has now expired, so
/// materials may have left the shelf with no record at all.
async fn unrecorded_openings(
    db: &DatabaseConnection,
    ctx: &Context,
    since: DateTimeWithTimeZone,
) -> Result<Vec<Flag>, AppError> {
    let now = util::now();
    let abandoned = CheckoutCodes::find()
        .filter(checkout_codes::Column::UsedAt.is_null())
        .filter(checkout_codes::Column::ExpiresAt.lt(now))
        .filter(checkout_codes::Column::CreatedAt.gte(since))
        .order_by_desc(checkout_codes::Column::CreatedAt)
        .all(db)
        .await?;

    Ok(abandoned
        .into_iter()
        .map(|code| {
            let who = ctx
                .person(Some(code.user_id))
                .unwrap_or_else(|| "an unknown account".to_owned());
            Flag {
                kind: "unrecorded_opening",
                severity: Severity::High,
                title: format!("{who} opened the cabinet but recorded nothing"),
                detail:
                    "The cabinet issued a code and it was never used, so whatever left the shelf \
                     has no written record. Ask them what they took and add it, or confirm they \
                     took nothing."
                        .to_owned(),
                at: code.created_at,
                person: Some(who),
                device: ctx.device(code.device_id),
                count: 1,
            }
        })
        .collect())
}

/// A registered finger at the cabinet with no door scan behind it. This is the
/// case the whole sequence exists to catch, so every instance is flagged.
fn tailgating(ctx: &Context, events: &[access_events::Model]) -> Vec<Flag> {
    events
        .iter()
        .filter(|e| e.event_type == AccessEventType::BoxDeniedNoSession)
        .map(|e| {
            let who = ctx
                .person(e.user_id)
                .or_else(|| e.finger_token.clone())
                .unwrap_or_else(|| "an unknown finger".to_owned());
            Flag {
                kind: "tailgating",
                severity: Severity::High,
                title: format!("{who} tried the cabinet without scanning the door"),
                detail:
                    "Either they walked in behind someone, or they scanned the door too long ago. \
                     The cabinet refused them, which is the system working, but repeated attempts \
                     by the same person are worth a conversation."
                        .to_owned(),
                at: e.created_at,
                person: Some(who),
                device: ctx.device(e.device_id),
                count: 1,
            }
        })
        .collect()
}

/// Someone typing codes the cabinet never gave them. One is a typo; several is
/// an attempt to invent a record for materials they did not sign out.
fn code_guessing(ctx: &Context, events: &[access_events::Model]) -> Vec<Flag> {
    let mut per_user: HashMap<Option<Uuid>, Vec<&access_events::Model>> = HashMap::new();
    for event in events
        .iter()
        .filter(|e| e.event_type == AccessEventType::CheckoutCodeRejected)
    {
        per_user.entry(event.user_id).or_default().push(event);
    }

    per_user
        .into_iter()
        .filter(|(_, hits)| hits.len() as u32 >= PROBE_THRESHOLD)
        .map(|(user_id, hits)| {
            let who = ctx
                .person(user_id)
                .unwrap_or_else(|| "an unknown account".to_owned());
            let count = hits.len() as u32;
            Flag {
                kind: "code_guessing",
                severity: Severity::High,
                title: format!("{who} was refused a checkout code {count} times"),
                detail:
                    "Codes only come from the cabinet as it opens. Repeated refusals mean someone \
                     is guessing, reusing an old code, or trying to log materials without having \
                     opened anything."
                        .to_owned(),
                at: hits.first().map(|e| e.created_at).unwrap_or_else(util::now),
                person: Some(who),
                device: None,
                count,
            }
        })
        .collect()
}

/// Unknown fingers presented over and over at one reader: someone trying whether
/// any of their fingers happens to be enrolled.
fn finger_probing(ctx: &Context, events: &[access_events::Model]) -> Vec<Flag> {
    let mut per_device: HashMap<Option<Uuid>, Vec<&access_events::Model>> = HashMap::new();
    for event in events.iter().filter(|e| {
        matches!(
            e.event_type,
            AccessEventType::DoorDeniedUnregistered | AccessEventType::BoxDeniedUnregistered
        )
    }) {
        per_device.entry(event.device_id).or_default().push(event);
    }

    per_device
        .into_iter()
        .filter(|(_, hits)| hits.len() as u32 >= PROBE_THRESHOLD)
        .map(|(device_id, hits)| {
            let where_ = ctx
                .device(device_id)
                .unwrap_or_else(|| "a reader".to_owned());
            let fingers: std::collections::BTreeSet<&str> = hits
                .iter()
                .filter_map(|e| e.finger_token.as_deref())
                .collect();
            let count = hits.len() as u32;
            Flag {
                kind: "finger_probing",
                severity: Severity::Medium,
                title: format!("{count} unregistered fingers refused at {where_}"),
                detail: format!(
                    "{} distinct finger slots were presented and refused. This is normal while \
                     people enroll, and suspicious otherwise.",
                    fingers.len().max(1)
                ),
                at: hits.first().map(|e| e.created_at).unwrap_or_else(util::now),
                person: None,
                device: Some(where_),
                count,
            }
        })
        .collect()
}

/// The cabinet opening in the small hours. Legitimate sometimes, but it is
/// exactly when nobody is around to notice.
fn after_hours(ctx: &Context, events: &[access_events::Model]) -> Vec<Flag> {
    events
        .iter()
        .filter(|e| e.event_type == AccessEventType::BoxGranted)
        .filter(|e| {
            let hour = util::in_lab_time(e.created_at).hour();
            hour < DAY_START_HOUR || hour >= DAY_END_HOUR
        })
        .map(|e| {
            let who = ctx
                .person(e.user_id)
                .unwrap_or_else(|| "someone".to_owned());
            let when = e.created_at;
            Flag {
                kind: "after_hours",
                severity: Severity::Medium,
                title: format!(
                    "{who} opened the cabinet at {}",
                    util::in_lab_time(when).format("%-I:%M %p on %b %-d")
                ),
                detail: format!(
                    "Outside {DAY_START_HOUR}:00 to {DAY_END_HOUR}:00, when the room is usually \
                     empty. Worth confirming the visit was expected."
                ),
                at: when,
                person: Some(who),
                device: ctx.device(e.device_id),
                count: 1,
            }
        })
        .collect()
}

/// One door scan, then the cabinet opened several times. Could be a person
/// making trips, or one person holding the window open for others.
fn repeat_openings(ctx: &Context, events: &[access_events::Model]) -> Vec<Flag> {
    let mut per_session: HashMap<Uuid, Vec<&access_events::Model>> = HashMap::new();
    for event in events
        .iter()
        .filter(|e| e.event_type == AccessEventType::BoxGranted)
    {
        if let Some(session) = event.access_session_id {
            per_session.entry(session).or_default().push(event);
        }
    }

    per_session
        .into_iter()
        .filter(|(_, hits)| hits.len() as u32 >= OPENINGS_PER_SESSION)
        .map(|(_, hits)| {
            let count = hits.len() as u32;
            let who = hits
                .first()
                .and_then(|e| ctx.person(e.user_id))
                .unwrap_or_else(|| "someone".to_owned());
            Flag {
                kind: "repeat_openings",
                severity: Severity::Low,
                title: format!("{who} opened the cabinet {count} times on one door scan"),
                detail:
                    "Fine if they were carrying loads back and forth. Less fine if the window was \
                     being used to let other people in."
                        .to_owned(),
                at: hits.first().map(|e| e.created_at).unwrap_or_else(util::now),
                person: Some(who),
                device: hits.first().and_then(|e| ctx.device(e.device_id)),
                count,
            }
        })
        .collect()
}

/// A disabled account whose finger is still being presented at a reader.
fn disabled_account_attempts(ctx: &Context, events: &[access_events::Model]) -> Vec<Flag> {
    events
        .iter()
        .filter(|e| e.event_type == AccessEventType::DoorDeniedInactive)
        .map(|e| {
            let who = ctx
                .person(e.user_id)
                .unwrap_or_else(|| "a disabled account".to_owned());
            Flag {
                kind: "disabled_account",
                severity: Severity::Medium,
                title: format!("{who} tried to get in on a disabled account"),
                detail:
                    "Their finger still matches, but the account is switched off. Either they were \
                     not told, or they should no longer have access at all."
                        .to_owned(),
                at: e.created_at,
                person: Some(who),
                device: ctx.device(e.device_id),
                count: 1,
            }
        })
        .collect()
}

/// Failed sign-ins piling up against one identifier: someone guessing a
/// password on the website rather than at the door.
async fn sign_in_attempts(
    db: &DatabaseConnection,
    since: DateTimeWithTimeZone,
) -> Result<Vec<Flag>, AppError> {
    let failures = LoginEvents::find()
        .filter(login_events::Column::Success.eq(false))
        .filter(login_events::Column::CreatedAt.gte(since))
        .order_by_desc(login_events::Column::CreatedAt)
        .all(db)
        .await?;

    let mut per_identifier: HashMap<String, Vec<login_events::Model>> = HashMap::new();
    for failure in failures {
        per_identifier
            .entry(failure.identifier.to_lowercase())
            .or_default()
            .push(failure);
    }

    Ok(per_identifier
        .into_iter()
        .filter(|(_, hits)| hits.len() as u32 >= SIGN_IN_THRESHOLD)
        .map(|(identifier, hits)| {
            let count = hits.len() as u32;
            let addresses: std::collections::BTreeSet<&str> = hits
                .iter()
                .filter_map(|h| h.ip_address.as_deref())
                .collect();
            Flag {
                kind: "sign_in_attempts",
                severity: Severity::Medium,
                title: format!("{count} failed sign-ins for \"{identifier}\""),
                detail: format!(
                    "From {} address{}. If this is not the account holder forgetting their \
                     password, someone is guessing it.",
                    addresses.len().max(1),
                    if addresses.len() == 1 { "" } else { "es" }
                ),
                at: hits.first().map(|h| h.created_at).unwrap_or_else(util::now),
                person: Some(identifier),
                device: None,
                count,
            }
        })
        .collect())
}

/// An account was switched off while it still held a live door window.
async fn sessions_for_disabled_users(
    db: &DatabaseConnection,
    ctx: &Context,
) -> Result<Vec<Flag>, AppError> {
    let now = util::now();
    let live = AccessSessions::find()
        .filter(access_sessions::Column::Status.eq(SessionStatus::Active))
        .filter(access_sessions::Column::ExpiresAt.gt(now))
        .all(db)
        .await?;
    if live.is_empty() {
        return Ok(Vec::new());
    }

    let disabled: std::collections::HashSet<Uuid> = Users::find()
        .filter(users::Column::IsActive.eq(false))
        .all(db)
        .await?
        .into_iter()
        .map(|u| u.id)
        .collect();

    Ok(live
        .into_iter()
        .filter(|s| disabled.contains(&s.user_id))
        .map(|s| {
            let who = ctx
                .person(Some(s.user_id))
                .unwrap_or_else(|| "a disabled account".to_owned());
            Flag {
                kind: "session_for_disabled_user",
                severity: Severity::High,
                title: format!("{who} was disabled but still holds an open cabinet window"),
                detail:
                    "Their door scan is still valid, so the cabinet would open for them right now. \
                     The window closes on its own, but until then they retain access."
                        .to_owned(),
                at: s.opened_at,
                person: Some(who),
                device: ctx.device(Some(s.door_device_id)),
                count: 1,
            }
        })
        .collect())
}

/// A reader that has stopped reporting. Nobody can be refused by a node that is
/// not running, and nothing it sees gets recorded.
async fn silent_readers(
    db: &DatabaseConnection,
    _ctx: &Context,
    now: DateTimeWithTimeZone,
) -> Result<Vec<Flag>, AppError> {
    let nodes = Devices::find()
        .filter(devices::Column::IsActive.eq(true))
        .all(db)
        .await?;

    Ok(nodes
        .into_iter()
        .filter_map(|d| {
            let quiet_for = match d.last_seen_at {
                Some(seen) => now - seen,
                // Never seen at all: worth saying so once it has been registered.
                None => chrono::Duration::days(999),
            };
            if quiet_for < chrono::Duration::hours(1) {
                return None;
            }
            let human = if d.last_seen_at.is_none() {
                "has never reported in".to_owned()
            } else {
                format!("has been quiet for {} hours", quiet_for.num_hours())
            };
            Some(Flag {
                kind: "silent_reader",
                severity: Severity::Medium,
                title: format!("{} {}", d.name, human),
                detail:
                    "A node that is not reporting cannot grant or refuse anyone, and anything that \
                     happens at it goes unrecorded. Check power and Wi-Fi."
                        .to_owned(),
                at: d.last_seen_at.unwrap_or(d.created_at),
                person: None,
                device: Some(d.name),
                count: 1,
            })
        })
        .collect())
}

/// Enrollment codes issued and never claimed: someone leaning on the ENROLL
/// button, or an enrollment that quietly failed halfway.
fn unclaimed_enrollments(ctx: &Context, events: &[access_events::Model]) -> Vec<Flag> {
    let mut per_device: HashMap<Option<Uuid>, u32> = HashMap::new();
    let mut latest: HashMap<Option<Uuid>, DateTimeWithTimeZone> = HashMap::new();

    let bound: std::collections::HashSet<&str> = events
        .iter()
        .filter(|e| e.event_type == AccessEventType::EnrollBound)
        .filter_map(|e| e.finger_token.as_deref())
        .collect();

    for event in events
        .iter()
        .filter(|e| e.event_type == AccessEventType::EnrollCodeIssued)
    {
        // A code whose finger later bound is a success, not a loose end.
        if event
            .finger_token
            .as_deref()
            .is_some_and(|t| bound.contains(t))
        {
            continue;
        }
        *per_device.entry(event.device_id).or_insert(0) += 1;
        latest
            .entry(event.device_id)
            .and_modify(|at| {
                if event.created_at > *at {
                    *at = event.created_at;
                }
            })
            .or_insert(event.created_at);
    }

    per_device
        .into_iter()
        .filter(|(_, count)| *count >= PROBE_THRESHOLD)
        .map(|(device_id, count)| {
            let where_ = ctx
                .device(device_id)
                .unwrap_or_else(|| "a reader".to_owned());
            Flag {
                kind: "unclaimed_enrollments",
                severity: Severity::Low,
                title: format!("{count} enrollment codes at {where_} were never claimed"),
                detail: "Codes were issued but no finger was ever bound. Usually someone pressing \
                     ENROLL out of curiosity; occasionally an enrollment that keeps failing."
                    .to_owned(),
                at: latest.get(&device_id).copied().unwrap_or_else(util::now),
                person: None,
                device: Some(where_),
                count,
            }
        })
        .collect()
}

/// Convenience for the dashboard: how many flags at each severity.
#[derive(Debug, Serialize)]
pub struct FlagSummary {
    pub high: u32,
    pub medium: u32,
    pub low: u32,
    pub total: u32,
    pub days: i64,
}

pub fn summarize(flags: &[Flag], days: i64) -> FlagSummary {
    let count = |s: Severity| flags.iter().filter(|f| f.severity == s).count() as u32;
    FlagSummary {
        high: count(Severity::High),
        medium: count(Severity::Medium),
        low: count(Severity::Low),
        total: flags.len() as u32,
        days,
    }
}

/// Kept so the module compiles standalone in tests without a database.
#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> Context {
        Context {
            users: HashMap::new(),
            devices: HashMap::new(),
        }
    }

    #[test]
    fn a_quiet_trail_raises_nothing() {
        assert!(tailgating(&ctx(), &[]).is_empty());
        assert!(code_guessing(&ctx(), &[]).is_empty());
        assert!(after_hours(&ctx(), &[]).is_empty());
    }

    #[test]
    fn day_window_is_sane() {
        assert!(DAY_START_HOUR < DAY_END_HOUR);
    }
}
