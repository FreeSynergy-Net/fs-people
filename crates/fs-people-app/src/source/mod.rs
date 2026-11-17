//! From the program's contract to the workbench's source.
//!
//! Pattern: Adapter — `adapter.rs` drives the async contract from the interface thread. What was
//! here besides is sorted out (not compiled): the card as form, sheet and the write-back are in
//! `fs-people-view`, the sensitive marking in `fs-people-core`, the socket client in
//! `fs-people-client` (see `sortierung/people.md`).

mod adapter;

pub use adapter::PeopleAdapter;
pub use fs_people_client::{NoRoles, RemotePeople};
pub use fs_people_view::apply_draft;
