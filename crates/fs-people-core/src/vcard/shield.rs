//! Panic shield around calls into the foreign `calcard-next` converter.
//!
//! Pattern: Adapter (guard) — imported files are foreign input; a panic inside the converter becomes
//! `Error::VCard`, it never reaches the caller.

use std::panic::{UnwindSafe, catch_unwind};

use crate::error::{Error, Result};

/// Run `f`; a panic inside it is returned as `Error::VCard`.
pub(super) fn shielded<T>(f: impl FnOnce() -> Result<T> + UnwindSafe) -> Result<T> {
    catch_unwind(f)
        .unwrap_or_else(|_| Err(Error::VCard("the converter failed on this input".into())))
}

#[cfg(test)]
mod tests {
    use super::*;

    // No input is known that makes calcard-next 3.0.1 panic, so the seam itself is tested with a
    // closure that panics; import and export both go through this seam.
    #[test]
    fn a_panic_in_the_converter_becomes_an_error() {
        let r: Result<()> = shielded(|| panic!("boom"));
        assert!(matches!(r, Err(Error::VCard(m)) if m.contains("failed")));
    }
}
