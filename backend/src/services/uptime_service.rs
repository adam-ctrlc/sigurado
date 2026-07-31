//! Connectivity tracking for the reader nodes.
//!
//! Every authenticated device request (heartbeat, scan, enrollment) is a sign of
//! life. Rather than storing each one, contact is folded into periods: while the
//! beats keep arriving, the open period is extended; once a node has been quiet
//! longer than `OFFLINE_AFTER`, the next beat starts a new period. A gap between
//! two periods is downtime, and no background sweeper is needed to notice it.

use std::time::Duration;

use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use uuid::Uuid;

use crate::{
    entities::{device_connections, prelude::DeviceConnections},
    error::AppError,
    util,
};

/// Two missed 30-second heartbeats. Kept in step with the dashboard's window.
pub const OFFLINE_AFTER: Duration = Duration::from_secs(90);

/// True when the node was heard from recently enough to count as connected.
pub fn is_live(last_seen: Option<DateTimeWithTimeZone>, now: DateTimeWithTimeZone) -> bool {
    match last_seen {
        Some(seen) => (now - seen).num_seconds() < OFFLINE_AFTER.as_secs() as i64,
        None => false,
    }
}

/// Fold one sign of life into the device's connection history.
pub async fn record_contact(db: &DatabaseConnection, device_id: Uuid) -> Result<(), AppError> {
    let now = util::now();

    let open = DeviceConnections::find()
        .filter(device_connections::Column::DeviceId.eq(device_id))
        .order_by_desc(device_connections::Column::LastSeenAt)
        .one(db)
        .await?;

    match open {
        // Still inside the window: the same stretch of uptime continues.
        Some(period) if is_live(Some(period.last_seen_at), now) => {
            let heartbeats = period.heartbeats.saturating_add(1);
            let mut active: device_connections::ActiveModel = period.into();
            active.last_seen_at = Set(now);
            active.heartbeats = Set(heartbeats);
            active.update(db).await?;
        }
        // First contact, or contact after a gap: a new stretch begins.
        _ => {
            device_connections::ActiveModel {
                id: Set(Uuid::new_v4()),
                device_id: Set(device_id),
                connected_at: Set(now),
                last_seen_at: Set(now),
                heartbeats: Set(1),
            }
            .insert(db)
            .await?;
        }
    }

    Ok(())
}

/// When the node's current stretch of uptime began, or None if it is offline.
pub async fn connected_since(
    db: &DatabaseConnection,
    device_id: Uuid,
) -> Result<Option<DateTimeWithTimeZone>, AppError> {
    let now = util::now();
    let latest = DeviceConnections::find()
        .filter(device_connections::Column::DeviceId.eq(device_id))
        .order_by_desc(device_connections::Column::LastSeenAt)
        .one(db)
        .await?;
    Ok(latest
        .filter(|p| is_live(Some(p.last_seen_at), now))
        .map(|p| p.connected_at))
}

/// Most recent connection periods, newest first.
pub async fn history(
    db: &DatabaseConnection,
    device_id: Uuid,
    limit: u64,
) -> Result<Vec<device_connections::Model>, AppError> {
    Ok(DeviceConnections::find()
        .filter(device_connections::Column::DeviceId.eq(device_id))
        .order_by_desc(device_connections::Column::ConnectedAt)
        .limit(limit)
        .all(db)
        .await?)
}
