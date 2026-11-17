//! The core's errors as the workbench's reasons.
//! Herkunft: fs-people@52438a65:crates/fs-people-app/src/source/adapter.rs (reason, reason_text)
//!
//! Pattern: Adapter — what the core said, in the workbench's terms.

use fs_components_workbenches::SourceError;
use fs_people_core::Error;

/// What the core said, in the workbench's terms. A store refusing a sensitive card is said plainly.
pub fn reason(error: &Error) -> SourceError {
    match error {
        Error::Store(fs_objects::Error::EncryptionRequired(_)) => SourceError::failed(format!(
            "this card is marked sensitive and this store is not encrypted, so it was refused ({error})"
        )),
        Error::Changed(_) => {
            SourceError::conflict("Someone else changed this person since you started.")
        }
        Error::NotFound(_) | Error::ListNotFound(_) => SourceError::not_found(error.to_string()),
        Error::Unsupported(_) => SourceError::unsupported(error.to_string()),
        _ => SourceError::failed(error.to_string()),
    }
}

/// The same reason as plain text.
pub fn reason_text(error: &Error) -> String {
    reason(error).message
}
