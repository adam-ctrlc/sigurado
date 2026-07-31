//! Outbound SMS for the SIM800L on the cabinet node.
//!
//! The server has no modem, so it queues a row per recipient and the GSM-capable
//! reader drains that queue over the device API and reports the result. Queueing
//! is therefore never allowed to fail an access decision: the lock has already
//! opened by the time an alert is written.

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use uuid::Uuid;

use crate::{
    entities::{
        prelude::{SmsMessages, SmsRecipients, Users},
        sea_orm_active_enums::{Role, SmsStatus},
        sms_messages, sms_recipients, users,
    },
    error::AppError,
    util,
};

/// SMS is charged per 160 characters, so alerts are kept to one part.
pub const MAX_BODY: usize = 160;
/// How many times the node may retry a number before the row is given up on.
pub const MAX_ATTEMPTS: i32 = 3;

/// What a recipient can subscribe to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertKind {
    CabinetOpened,
    AccessDenied,
    DoorOpened,
}

impl AlertKind {
    fn column(self) -> sms_recipients::Column {
        match self {
            AlertKind::CabinetOpened => sms_recipients::Column::NotifyCabinetOpened,
            AlertKind::AccessDenied => sms_recipients::Column::NotifyAccessDenied,
            AlertKind::DoorOpened => sms_recipients::Column::NotifyDoorOpened,
        }
    }
}

/// Normalizes a Philippine mobile number to E.164, and rejects anything that is
/// clearly not dialable. Stored in one shape so the modem never has to guess.
pub fn normalize_number(raw: &str) -> Result<String, AppError> {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    let e164 = if let Some(rest) = digits.strip_prefix("63") {
        // 63 9XX XXX XXXX
        format!("+63{rest}")
    } else if let Some(rest) = digits.strip_prefix('0') {
        // 09XX XXX XXXX
        format!("+63{rest}")
    } else if digits.len() == 10 && digits.starts_with('9') {
        // 9XX XXX XXXX
        format!("+63{digits}")
    } else if raw.trim().starts_with('+') {
        format!("+{digits}")
    } else {
        return Err(AppError::Validation(
            "enter a mobile number like 09171234567 or +639171234567".to_owned(),
        ));
    };

    let body = &e164[1..];
    if body.len() < 11 || body.len() > 15 {
        return Err(AppError::Validation(
            "that does not look like a mobile number".to_owned(),
        ));
    }
    Ok(e164)
}

/// Trims a body to one SMS part, on a character boundary.
fn clamp_body(body: &str) -> String {
    body.chars().take(MAX_BODY).collect()
}

/// Only staff carry a lab phone; a student's number is never texted.
pub fn may_receive_alerts(role: Role) -> bool {
    matches!(role, Role::Admin | Role::Faculty)
}

/// Queue one message per subscribed, active recipient. Returns how many were
/// written. Errors are logged rather than propagated by the callers on the
/// access path.
pub async fn queue_for_alert(
    db: &DatabaseConnection,
    kind: AlertKind,
    body: &str,
    access_event_id: Option<Uuid>,
) -> Result<usize, AppError> {
    // The number and the eligibility both live on the user, so join rather than
    // trusting a copy: a demoted or disabled account stops receiving alerts
    // without anyone having to remember to unsubscribe it.
    let pairs: Vec<(sms_recipients::Model, Option<users::Model>)> = SmsRecipients::find()
        .filter(sms_recipients::Column::IsActive.eq(true))
        .filter(kind.column().eq(true))
        .find_also_related(Users)
        .all(db)
        .await?;

    let body = clamp_body(body);
    let now = util::now();
    let mut queued = 0;
    for (recipient, user) in pairs {
        let Some(user) = user else { continue };
        if !user.is_active || !may_receive_alerts(user.role) {
            continue;
        }
        let Some(number) = user.phone_number.clone() else {
            continue;
        };
        sms_messages::ActiveModel {
            id: Set(Uuid::new_v4()),
            recipient_id: Set(Some(recipient.id)),
            phone_number: Set(number),
            body: Set(body.clone()),
            status: Set(SmsStatus::Pending),
            attempts: Set(0),
            error: Set(None),
            access_event_id: Set(access_event_id),
            sent_by_device_id: Set(None),
            created_at: Set(now),
            sent_at: Set(None),
        }
        .insert(db)
        .await?;
        queued += 1;
    }
    Ok(queued)
}

/// Queue a single message to one recipient, for the admin's test button.
pub async fn queue_direct(
    db: &DatabaseConnection,
    recipient: &sms_recipients::Model,
    number: &str,
    body: &str,
) -> Result<sms_messages::Model, AppError> {
    let now = util::now();
    Ok(sms_messages::ActiveModel {
        id: Set(Uuid::new_v4()),
        recipient_id: Set(Some(recipient.id)),
        phone_number: Set(number.to_owned()),
        body: Set(clamp_body(body)),
        status: Set(SmsStatus::Pending),
        attempts: Set(0),
        error: Set(None),
        access_event_id: Set(None),
        sent_by_device_id: Set(None),
        created_at: Set(now),
        sent_at: Set(None),
    }
    .insert(db)
    .await?)
}

/// The oldest pending messages, for the GSM node to send.
pub async fn pending(
    db: &DatabaseConnection,
    limit: u64,
) -> Result<Vec<sms_messages::Model>, AppError> {
    Ok(SmsMessages::find()
        .filter(sms_messages::Column::Status.eq(SmsStatus::Pending))
        .order_by_asc(sms_messages::Column::CreatedAt)
        .limit(limit)
        .all(db)
        .await?)
}

/// Record what the modem actually did with a message.
pub async fn record_result(
    db: &DatabaseConnection,
    id: Uuid,
    device_id: Uuid,
    sent: bool,
    error: Option<String>,
) -> Result<sms_messages::Model, AppError> {
    let message = SmsMessages::find_by_id(id)
        .one(db)
        .await?
        .ok_or(AppError::NotFound("sms message"))?;

    let attempts = message.attempts.saturating_add(1);
    let mut active: sms_messages::ActiveModel = message.into();
    active.attempts = Set(attempts);
    active.sent_by_device_id = Set(Some(device_id));

    if sent {
        active.status = Set(SmsStatus::Sent);
        active.sent_at = Set(Some(util::now()));
        active.error = Set(None);
    } else {
        // Leave it pending until the node has had a fair number of tries: a
        // SIM800L fails often enough that one refusal means little.
        active.status = Set(if attempts >= MAX_ATTEMPTS {
            SmsStatus::Failed
        } else {
            SmsStatus::Pending
        });
        active.error = Set(error.map(|e| e.chars().take(200).collect()));
    }

    Ok(active.update(db).await?)
}

#[cfg(test)]
mod tests {
    use super::normalize_number;

    #[test]
    fn normalizes_the_common_shapes() {
        assert_eq!(normalize_number("09171234567").unwrap(), "+639171234567");
        assert_eq!(
            normalize_number("+63 917 123 4567").unwrap(),
            "+639171234567"
        );
        assert_eq!(normalize_number("639171234567").unwrap(), "+639171234567");
        assert_eq!(normalize_number("9171234567").unwrap(), "+639171234567");
    }

    #[test]
    fn rejects_nonsense() {
        assert!(normalize_number("12345").is_err());
        assert!(normalize_number("hello").is_err());
    }
}
