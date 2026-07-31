use rand::RngExt;
use sea_orm::prelude::DateTimeWithTimeZone;

/// Current instant as the DB timestamp type. Server time is the only clock
/// (simulated NTP): every timestamp in the system flows through here.
pub fn now() -> DateTimeWithTimeZone {
    chrono::Utc::now().into()
}

/// The offset of the room the cabinet lives in. Timestamps are stored in UTC,
/// but a rule like "after hours" is about the clock on the wall, so reading one
/// needs the local offset. Defaults to Philippine time.
pub fn lab_offset() -> chrono::FixedOffset {
    static OFFSET: std::sync::OnceLock<chrono::FixedOffset> = std::sync::OnceLock::new();
    *OFFSET.get_or_init(|| {
        let hours: i32 = std::env::var("LAB_UTC_OFFSET_HOURS")
            .ok()
            .and_then(|raw| raw.trim().parse().ok())
            .filter(|h: &i32| (-14..=14).contains(h))
            .unwrap_or(8);
        chrono::FixedOffset::east_opt(hours * 3600)
            .unwrap_or_else(|| chrono::FixedOffset::east_opt(0).expect("UTC is a valid offset"))
    })
}

/// The same instant, read on the lab's clock.
pub fn in_lab_time(at: DateTimeWithTimeZone) -> DateTimeWithTimeZone {
    at.with_timezone(&lab_offset())
}

/// Human-readable enrollment code shown on the device LCD. Avoids ambiguous
/// characters (0/O, 1/I) so a person can retype it without confusion.
pub fn random_code(len: usize) -> String {
    const CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::rng();
    (0..len)
        .map(|_| CHARS[rng.random_range(0..CHARS.len())] as char)
        .collect()
}

/// Opaque token used for device secrets and simulated fingerprint identities.
pub fn random_token(len: usize) -> String {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::rng();
    (0..len)
        .map(|_| CHARS[rng.random_range(0..CHARS.len())] as char)
        .collect()
}
