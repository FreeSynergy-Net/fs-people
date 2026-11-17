//! `fs-people-view`: the module "look at / edit a card" of the people program.
//!
//! A card as a form (`draft.rs`), a form written back onto a card (`apply.rs`), a card as a
//! reading sheet (`sheet.rs`), the labels of what is shown (`field.rs`) and the core's errors as
//! the workbench's reasons (`reason.rs`). The rules hold for every frontend, not only for the
//! window. Sorted out of `fs-people-app` (conversation 3, answer 12).
//!
//! Pattern: Facade — this file only declares modules and re-exports the public API.
//!
//! Sorted in, not compiled (workflow geruest-heben).

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

mod apply;
mod draft;
pub mod field;
mod reason;
mod sheet;

pub use apply::apply_draft;
pub use draft::{blank_draft, draft_of};
pub use reason::{reason, reason_text};
pub use sheet::{first_email, sensitive_mark, sheet_of};
