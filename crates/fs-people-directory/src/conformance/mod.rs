//! Conformance rules of `person_directory_v1`, runnable against any provider (fake, real, remote).
//!
//! Pattern: Template method (a fixed list of rules, each checked against the `PersonDirectory` under test).
//!
//! `run_conformance` seeds its own data through `propose`, so it needs a provider with `person.propose` and
//! works on an empty or a filled directory (all seeds carry the marker `Zzconf`). A provider without
//! `person.propose` runs `run_unsupported` instead.

mod extra;
mod fixture;
mod fixture_rules;
mod read;
mod run;
mod write;

pub use fixture::ConformanceFixture;
pub use run::{Report, run_conformance, run_fixture_conformance, run_unsupported};
pub(crate) use run::{Seeds, address, entry};
