//! `fs-people-directory`: the light consumer side of People (ADR 0087, plan O2).
//!
//! Pattern: Facade (one crate door for the locked trait, a fake, a composite and the conformance tests).
//!
//! No Blinc, no app, no store: calendar, tasks and groups depend on this crate, never on People itself (ADR 0085
//! rule 3). The bridge to the real `PeopleRepo` is a later step (P0b) and lives elsewhere.

// Locked trait, hash dbbc4b8d8c1c93bc6eb343eb87c121d99ad9eba3857eed24764d9a842219947a: the folder is copied byte for
// byte and never edited; any change is a new version `person_directory_v2`.
pub mod person_directory_v1;

pub mod composite;
pub mod conformance;
pub mod doubles;
pub mod fake;

pub use composite::CompositeDirectory;
pub use doubles::{FailingDirectory, Gate, SlowDirectory};
pub use fake::{Entry, FakeDirectory, FakeFixture, Status, sample, sample_ref};
pub use person_directory_v1::*;
