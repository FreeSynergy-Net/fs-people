//! Whether a person is supervised (a "betreutes Konto": someone else answers for the account).
//!
//! Pattern: Null Object with a gate. Supervision is not built (plan "betreute Konten"): nothing is
//! stored that says who answers for whom, so nobody is supervised and [`is_supervised`] says
//! `Some(false)`. That is true today, not "unknown" (`None` would let fs-groups refuse every owner,
//! ADR 0095 point 14). The constant [`SUPERVISION_NOT_BUILT`] carries the claim; the compile-time
//! assertion and `tests/supervision_gate.rs` turn red as soon as the card or the schema learns a
//! supervision field while the constant is still `true`.

use fs_objects::ObjectId;

/// Supervision is not built: no card holds who answers for it. Set to `false` only together with
/// the code that stores and reads that relation; then `is_supervised` must answer from it.
pub const SUPERVISION_NOT_BUILT: bool = true;

/// Why [`is_supervised`] answers as it does, for a problem text or a log.
pub const SUPERVISION_NOT_BUILT_REASON: &str = "Betreuung noch nicht eingebaut (Plan betreute Konten): keine Verantwortungs-Beziehung gespeichert";

// The gate: whoever sets the constant to `false` must rewrite `is_supervised` and this line.
const _: () = assert!(
    SUPERVISION_NOT_BUILT,
    "is_supervised answers Some(false) only while supervision is not built"
);

/// Is `person` supervised? `Some(false)` while supervision is not built ([`SUPERVISION_NOT_BUILT`]):
/// nobody can be, because nothing stores it. `None` would mean "unknown" and is not used yet.
#[must_use]
pub fn is_supervised(_person: ObjectId) -> Option<bool> {
    if SUPERVISION_NOT_BUILT {
        Some(false)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nobody_is_supervised_while_it_is_not_built() {
        assert_eq!(is_supervised(ObjectId::new()), Some(false));
    }
}
