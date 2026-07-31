//! One-time codes shown on the cabinet's LCD when it opens.
//!
//! The website cannot tell on its own whether someone actually opened the
//! cabinet; only the reader knows. Issuing a code at the moment of unlocking and
//! demanding it back when the checkout is written ties the record to a real
//! opening, so a signed-in student cannot log materials they never took.

use std::time::Duration;

use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

use crate::{
    entities::{checkout_codes, prelude::CheckoutCodes},
    error::AppError,
    util,
};

/// Long enough to walk back to a bench and type it, short enough that a code
/// left on a whiteboard is useless later.
pub const CODE_TTL: Duration = Duration::from_secs(15 * 60);
pub const CODE_LEN: usize = 6;

/// Issue a fresh code for a cabinet opening. Any earlier unused code for the
/// same person is retired, so the LCD and the database never disagree about
/// which code is live.
pub async fn issue(
    db: &DatabaseConnection,
    user_id: Uuid,
    device_id: Option<Uuid>,
    access_session_id: Option<Uuid>,
) -> Result<checkout_codes::Model, AppError> {
    let now = util::now();

    let stale = CheckoutCodes::find()
        .filter(checkout_codes::Column::UserId.eq(user_id))
        .filter(checkout_codes::Column::UsedAt.is_null())
        .filter(checkout_codes::Column::ExpiresAt.gt(now))
        .all(db)
        .await?;
    for code in stale {
        let mut active: checkout_codes::ActiveModel = code.into();
        active.expires_at = Set(now);
        active.update(db).await?;
    }

    Ok(checkout_codes::ActiveModel {
        id: Set(Uuid::new_v4()),
        code: Set(util::random_code(CODE_LEN)),
        user_id: Set(user_id),
        device_id: Set(device_id),
        access_session_id: Set(access_session_id),
        expires_at: Set(now + chrono::Duration::from_std(CODE_TTL).unwrap_or_default()),
        used_at: Set(None),
        checkout_id: Set(None),
        created_at: Set(now),
    }
    .insert(db)
    .await?)
}

/// The code a person can still spend, if any. Used by the website to tell them
/// whether the cabinet has issued one.
pub async fn live_for_user(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<Option<checkout_codes::Model>, AppError> {
    let now = util::now();
    Ok(CheckoutCodes::find()
        .filter(checkout_codes::Column::UserId.eq(user_id))
        .filter(checkout_codes::Column::UsedAt.is_null())
        .filter(checkout_codes::Column::ExpiresAt.gt(now))
        .order_by_desc(checkout_codes::Column::CreatedAt)
        .one(db)
        .await?)
}

/// Checks a typed code and returns it unspent. The caller marks it used once the
/// checkout row exists, so a failed write does not burn the code.
pub async fn claim(
    db: &DatabaseConnection,
    user_id: Uuid,
    typed: &str,
) -> Result<checkout_codes::Model, AppError> {
    let code = typed.trim().to_uppercase();
    if code.is_empty() {
        return Err(AppError::Validation(
            "enter the code the cabinet showed you".to_owned(),
        ));
    }

    let found = CheckoutCodes::find()
        .filter(checkout_codes::Column::Code.eq(&code))
        .one(db)
        .await?
        .ok_or_else(|| {
            AppError::Validation("that code does not match anything the cabinet issued".to_owned())
        })?;

    // Someone else's code is refused with the same wording, so it cannot be used
    // to probe whether a code exists.
    if found.user_id != user_id {
        return Err(AppError::Validation(
            "that code does not match anything the cabinet issued".to_owned(),
        ));
    }
    if found.used_at.is_some() {
        return Err(AppError::Validation(
            "that code was already used for a checkout".to_owned(),
        ));
    }
    if found.expires_at <= util::now() {
        return Err(AppError::Validation(
            "that code has expired. Scan at the cabinet again for a new one".to_owned(),
        ));
    }

    Ok(found)
}

/// Spend the code against the checkout it authorized.
pub async fn spend(
    db: &DatabaseConnection,
    code: checkout_codes::Model,
    checkout_id: Uuid,
    when: DateTimeWithTimeZone,
) -> Result<(), AppError> {
    let mut active: checkout_codes::ActiveModel = code.into();
    active.used_at = Set(Some(when));
    active.checkout_id = Set(Some(checkout_id));
    active.update(db).await?;
    Ok(())
}
