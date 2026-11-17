//! `fs-people-vcard`: the vCard exchange module of the people program (RFC 6350, RFC 9553).
//!
//! vCard files in and out, by a path; the codec itself (`VCardFormat`, `People::import_vcard`,
//! `People::export_released`) stays in `fs-people-core`. Sorted out of `fs-people-app` as its own
//! module (conversation 3, answer 12).
//!
//! Pattern: Facade — this file only declares modules and re-exports the public API.
//!
//! Sorted in, not compiled (workflow geruest-heben).

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

mod transfer;

pub use transfer::{ExportReport, ImportSummary, export_all, import_file};
