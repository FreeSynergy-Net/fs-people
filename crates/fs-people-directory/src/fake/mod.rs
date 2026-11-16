//! `FakeDirectory`: an in-memory provider of `PersonDirectory` for tests of calendar, tasks and groups.
//!
//! Pattern: Fake (test double) behind the Repository trait; every method of the locked trait is implemented, the
//! mutators (`delete`, `unshare`, `restore`, `merge`) feed the change log like the real provider would.

mod fixture;
mod ids;
mod ops;
mod sample;
mod state;
mod store;

pub use fixture::FakeFixture;
pub use sample::{sample, sample_ref};
pub use state::FakeDirectory;
pub(crate) use state::Inner;
pub use store::{Entry, Status, find_order, same_address, suggest_rank};
