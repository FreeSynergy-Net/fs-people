//! The one `Domain` of the People directory and how a store of it is opened.
//!
//! Pattern: Registry entry (prefix, tables, ordered steps; the store applies the missing steps, forward only).

use fs_objects_v5::v5::{Domain, Result, Step, Store, Table};

use super::tables::{BIRTHDAY, CONSENT, SNAPSHOT};

/// The tables of step 1.
const STEP_1: &[Table] = &[CONSENT, SNAPSHOT, BIRTHDAY];

/// The domain of the People directory: prefix `people`, three tables, one step. Steps are only appended: a change
/// to a table is a new step, never an edit of this one (`StepMismatch` finds the edit).
pub const DOMAIN: Domain = Domain::new(
    "people",
    "People directory: consents, frozen snapshots, birthdays",
    STEP_1,
    &[Step::Create(STEP_1)],
);

/// Open the directory store at `path` (`:memory:` is volatile) and register the domain; `now` in milliseconds.
///
/// # Errors
/// The v5 layer's: `Unavailable`, `Schema`, `StepMismatch` (a store made by other steps).
pub async fn open(path: &str, now: i64) -> Result<Store> {
    let store = Store::open(path, now).await?;
    store.register_domain(&DOMAIN, now).await?;
    Ok(store)
}
