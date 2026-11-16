//! Test doubles that are no data: a provider that always fails and a provider that holds its answers back.
//!
//! Pattern: Test double (Null Object variants of `PersonDirectory`).

mod failing;
mod slow;

pub use failing::FailingDirectory;
pub use slow::{Gate, SlowDirectory};
