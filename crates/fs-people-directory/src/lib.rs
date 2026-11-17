//! `fs-people-directory`: the light consumer side of People (ADR 0087, plan O2): names, suggestions, excerpts for a
//! purpose and consents.
//!
//! Pattern: Facade (one crate door for the locked trait, a fake, a composite and the conformance tests).
//!
//! No Blinc, no app, no store: calendar, tasks and groups depend on this crate, never on People itself (ADR 0085
//! rule 3). The bridge to the real `PeopleRepo` is a later step (P0b) and lives elsewhere.

// Fixpunkt (working fixpoint, not a freeze), hash 6c885cfe515a3d8c195a0ddcf43f19602d84e8342e29cca7d4297ae808037fe0: the
// folder is copied byte for byte and not edited here; a change is moved by the strategy (new hash), later a new
// version `person_directory_v2`.
pub mod person_directory_v1;

pub mod composite;
pub mod conformance;
pub mod doubles;
pub mod fake;
pub mod retention;

// Probe `person_directory_v2` (ohne Sperre): Entwurfsort, nichts davon ist ein Vertrag, bis ein Fixpunkt gesetzt ist.
pub mod draft;

pub use composite::CompositeDirectory;
pub use doubles::{FailingDirectory, Gate, SlowDirectory};
#[doc(hidden)]
pub use fake::Defect;
pub use fake::{Entry, FakeDirectory, FakeFixture, Rights, Status, sample, sample_ref};
pub use person_directory_v1::*;
pub use retention::ConsentRetention;
