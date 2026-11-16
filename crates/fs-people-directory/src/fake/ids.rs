//! UUIDv7 for new ids of the fake: 48 bit time, version, variant and random bits (ADR 0027).
//!
//! Pattern: Value Object factory (pure functions; the random bits come from the std hasher seeds).

use std::hash::{BuildHasher, Hasher, RandomState};

fn random() -> u64 {
    RandomState::new().build_hasher().finish()
}

/// A UUIDv7 as 128 bits for the millisecond `ms` (negative counts as 0).
pub(crate) fn uuid7(ms: i64) -> u128 {
    let time = u128::from(u64::try_from(ms).unwrap_or(0)) & 0xFFFF_FFFF_FFFF;
    (time << 80)
        | (0x7 << 76)
        | (u128::from(random() & 0xFFF) << 64)
        | (0b10 << 62)
        | u128::from(random() & 0x3FFF_FFFF_FFFF_FFFF)
}

/// The same as an IRI `urn:uuid:…`.
pub(crate) fn uuid7_urn(ms: i64) -> String {
    let v = uuid7(ms);
    format!(
        "urn:uuid:{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        v >> 96,
        (v >> 80) & 0xFFFF,
        (v >> 64) & 0xFFFF,
        (v >> 48) & 0xFFFF,
        v & 0xFFFF_FFFF_FFFF
    )
}
