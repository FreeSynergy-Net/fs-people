//! Thema `people`, Teil `directory`, Ziel v2 (PROBE, ohne Sperre, ohne Sperrdatei): das Delta zu `person_directory_v1`
//! (Workflow trait-delta; Entwurf `.scratch/katalog-plan/PERSON-DIRECTORY-V2-ENTWURF.md`, Nutzerantworten b7 17.11.).
//!
//! Pattern: Repository + Null Object (wie v1); Port (`consumed.rs`: FieldShares); Value Object (`purposes.rs`,
//! `retention.rs`).
//!
//! Vier Punkte plus eine Nutzerregel: N2 Geburtstag und andere sensible Felder bei `OwnUse` nur aus eigenem Eintrag oder
//! mit Freigabe (`excerpt.rs`, `consumed.rs`); N3 KI-Zustimmung nur für die Situation (`Lifetime`, `end_situation`,
//! `consent.rs`); N6 die Zweck-Liste ist editierbar (`purposes.rs`); Anpassung 6 Aufbewahrung nach Widerruf im Vertrag,
//! RÜCKWIRKEND (`retention.rs`); Aufnahmedauer mitteilen (`RecordingNotice` in `consent.rs`, VORLÄUFIG hier, zieht mit
//! dem nächsten fs-types-Tag nach fs-types um).
//!
//! Dateien: BYTE-GLEICH mit v1: `shared.rs`, `view.rs` (Probe prüft per `cmp`). Geändert, nur angehängt: `consent.rs`,
//! `excerpt.rs`, `errors.rs`, `api.rs`. Neu: `purposes.rs`, `consumed.rs`, `retention.rs`, `null.rs` (Null-Anbieter).
//! v1 bleibt UNVERÄNDERT (Hash 6c885cfe…). Bewusste Abweichungen vom Entwurf stehen im Probe-Bericht.
//!
//! Fester Ort nach dem Fixpunkt: `src/person_directory_v2/`; vor dem ersten Release ersetzt v2 dann v1.

mod api;
mod consent;
pub mod consumed;
mod errors;
mod excerpt;
pub mod null;
mod purposes;
pub mod retention;
pub mod shared;
mod view;

pub use api::*;
pub use consent::*;
pub use consumed::{BirthdayPrecision, FieldShares, NoFieldShares, ShareField};
pub use errors::*;
pub use excerpt::*;
pub use null::NullDirectory;
pub use purposes::*;
pub use retention::{ConsentRetention, RetentionPreview, RetentionSettings, ZoneOffsets};
pub use shared::*;
pub use view::*;
