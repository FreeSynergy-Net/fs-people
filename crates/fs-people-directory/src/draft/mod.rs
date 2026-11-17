//! Entwurfsort für Trait-Deltas, die noch nicht gesperrt sind (Workflow trait-delta). Hier: `person_directory_v2`, das
//! Delta zu `person_directory_v1` (N2, N3, N6, Anpassung 6, Aufnahmedauer), mit seiner Probe.
//!
//! Pattern: Repository + Port (der Trait liegt in `person_directory_v2/`; die Konformitätsfälle K1–K33 in
//! `conformance_v2`).
//!
//! Fester Ort nach dem Fixpunkt: `crates/fs-people-directory/src/person_directory_v2/`; `person_directory_v1/` bleibt
//! unberührt. `people_v2.capnp` ist die Probe-Kopie von `people.capnp` (fs-interfaces v0.0.27) plus Anhang.

// Wie `groups_v2`: nicht umformatiert, damit `shared.rs` und `view.rs` byte-gleich mit v1 bleiben (`conformance_v2::pure`
// prüft es), mit denselben Ausnahmen wie v1.
#[allow(
    missing_docs,
    clippy::doc_markdown,
    clippy::must_use_candidate,
    clippy::wildcard_imports
)]
pub mod person_directory_v2;

#[cfg(test)]
mod conformance_v2;
