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
//! `excerpt.rs`, `errors.rs`, `api.rs`: Einfügungen, dazu in `errors.rs` und `api.rs` je eine geänderte `use`-Zeile und
//! eine geänderte v1-Doku-Zeile (R2-N1, Fähigkeits-Doku). Neu: `purposes.rs`, `catalog.rs` (Trait `PurposeCatalog`),
//! `consumed.rs`, `retention.rs`, `null.rs` (Null-Anbieter für beide Traits).
//!
//! Angekündigter Umzug (MOVE-NOTICE, auch Satz für die Sperrdatei): `RecordingKind`, `RecordingKeep`, `RecordingNotice`
//! in `consent.rs` werden mit dem nächsten fs-types-Tag durch `pub use fs_types::...` ersetzt, gleiche Namen, Felder,
//! Varianten und Sinn; das ist eine angekündigte Änderung (neuer Hash mit `vorgaenger`, kein v3), jede andere
//! Abweichung ist ein Sprung. Trait-Aufteilung: `PersonDirectory` hat 15 Methoden, `PurposeCatalog` 5.
//! v1 bleibt UNVERÄNDERT (Hash 6c885cfe…). Bewusste Abweichungen vom Entwurf stehen im Probe-Bericht.
//!
//! Fester Ort nach dem Fixpunkt: `src/person_directory_v2/`; vor dem ersten Release ersetzt v2 dann v1.

mod api;
mod catalog;
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
pub use catalog::PurposeCatalog;
pub use consent::*;
pub use consumed::{BirthdayPrecision, FieldShares, NoFieldShares, ShareField};
pub use errors::*;
pub use excerpt::*;
pub use null::{NullCatalog, NullDirectory};
pub use purposes::*;
pub use retention::{ConsentRetention, RetentionPreview, RetentionSettings, ZoneOffsets};
pub use shared::*;
pub use view::*;
