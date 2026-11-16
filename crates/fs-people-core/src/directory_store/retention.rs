//! The program setting `consent_retention`: how many years after the end of the year of a withdrawal the proof of
//! a consent is kept (user answer 40, 16.11.: 3 years, changeable). It lives in the store's `setting` table as a
//! whole number, next to `trash_days`; a later change applies to later withdrawals only (rows keep their
//! `retain_until`).
//!
//! Pattern: Repository functions over a unit of work (`Conn`), the pattern of the trash-period setting (ADR 0073
//! addendum). The value is `ConsentRetention` (1 to 10 years, default 3).
//!
//! Not legal advice: the default and the limits are the program's, the operator decides what applies to them.

use fs_objects_v5::v5::codec::{damaged, get_int, int, invalid, text};
use fs_objects_v5::v5::tables::SETTING;
use fs_objects_v5::v5::{Conn, Result};
use fs_people_directory::ConsentRetention;

/// The key in `setting`.
pub const KEY_CONSENT_RETENTION: &str = "consent_retention";

/// The setting: the stored years, or the default (3) when none was set.
///
/// # Errors
/// `Unavailable` for a stored value outside the allowed range (a damaged store).
pub async fn consent_retention(c: &Conn<'_>) -> Result<ConsentRetention> {
    let rows = c
        .query(
            "SELECT int_value FROM setting WHERE key = ?",
            vec![text(KEY_CONSENT_RETENTION)],
        )
        .await?;
    let Some(row) = rows.first() else {
        return Ok(ConsentRetention::default());
    };
    u8::try_from(get_int(row, 0)?)
        .ok()
        .and_then(ConsentRetention::new)
        .ok_or_else(|| damaged("consent_retention out of range"))
}

/// Change the setting (years, 1 to 10).
///
/// # Errors
/// `Invalid` outside the range.
pub async fn set_consent_retention(c: &Conn<'_>, years: u8) -> Result<ConsentRetention> {
    let value = ConsentRetention::new(years)
        .ok_or_else(|| invalid("setting.consent_retention", "between 1 and 10 years"))?;
    c.upsert(
        &SETTING,
        &["key"],
        vec![text(KEY_CONSENT_RETENTION), int(i64::from(value.years()))],
    )
    .await?;
    Ok(value)
}
