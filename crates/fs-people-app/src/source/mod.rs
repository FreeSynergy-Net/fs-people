//! From the program's contract to the workbench's source.
//!
//! Pattern: Adapter — `adapter.rs` drives the async contract from the interface thread, `sheet.rs`
//! shows a card, `draft.rs` makes a form of a card, `apply.rs` writes a form back onto a card;
//! `groups.rs` is the table of entry kinds both walk, `birthday.rs`, `sensitive.rs` and `role_edit.rs`
//! the fields that are not plain entries.

mod adapter;
mod apply;
mod birthday;
mod draft;
mod groups;
mod marks;
mod remote;
mod role_edit;
mod sensitive;
mod sheet;

pub use adapter::PeopleAdapter;
pub(crate) use adapter::reason_text;
pub use apply::apply_draft;
pub(crate) use marks::is_marked;
pub use remote::{CALL_LIMIT, NoRoles, RemotePeople, START_LIMIT};
