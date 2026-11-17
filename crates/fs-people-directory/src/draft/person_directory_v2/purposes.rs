//! N6: die Zweck-Liste ist editierbar. Standard-Zwecke (die 9 Varianten von `Purpose`) bleiben fest: dieselben Wörter,
//! dasselbe `needs_consent`; editierbar werden nur TEXTE und EIGENE Zwecke (`Purpose::Other(urn:uuid:<id>)`).
//!
//! Pattern: Value Object (Entwurf, Definition) + Repository-Teil (die Methoden `purposes`, `define_purpose`,
//! `revise_purpose`, `retire_purpose` stehen im Trait, `api.rs`).
//!
//! Auflösung der Anzeige: Person, dann Instanz, dann die mitgelieferte MF2-Datei (`people-purpose-<wort>-label` und
//! `-explanation`, de+en, ADR 0057), dann der Platzhalter "Sonstiger Zweck" mit der IRI, nie leer (K21).
//! VORLÄUFIG (b7 17.11., Frage 5): ein eigener Zweck braucht IMMER eine Einwilligung (`needs_consent` = true); später
//! änderbar, darum steht das Feld schon in `PurposeDef`.

use super::{PersonRef, Purpose, RecordingKind, Reference, Revision, Text, Utc};
use fs_types::Language;

/// Wem der Zweck gehört: der Instanz (Recht `purpose.write`) oder einer Person (für sich, ohne Recht).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PurposeOwner {
    Instance,
    Person(PersonRef),
}

/// Ein Text in einer Sprache (BCP 47, `Language` aus fs-types).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PurposeText {
    pub language: Language,
    /// MF2, ohne Platzhalter.
    pub label: Text,
    pub explanation: Text,
}

/// Eingabe für `define_purpose` und `revise_purpose`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PurposeDraft {
    pub owner: PurposeOwner,
    /// Mindestens eine Sprache, sonst `Invalid`.
    pub texts: Vec<PurposeText>,
    /// DPV-Begriff des gleichen Sinns (`skos:closeMatch`), nur zur Einordnung.
    pub close_match: Option<Reference>,
    /// Aufnahmeart des Zwecks; mit `Some` ist `ConsentEntry::notice` Pflicht (K30).
    pub recording: Option<RecordingKind>,
}

/// Ein Zweck mit seinen Texten, wie `purposes` ihn liefert.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PurposeDef {
    pub purpose: Purpose,
    /// `None` = mitgeliefert (Standard-Zweck ohne eigenen Text).
    pub owner: Option<PurposeOwner>,
    pub rev: Revision,
    /// Fassung der Texte (`urn:sha256:...`); `ConsentEntry::text` zeigt darauf, sie bleibt lesbar (Art. 7 Abs. 1).
    pub version: Reference,
    pub texts: Vec<PurposeText>,
    pub needs_consent: bool,
    pub close_match: Option<Reference>,
    pub recording: Option<RecordingKind>,
    /// Zurückgezogen: sperrt nur neues Erfassen; bestehende Einwilligungen gelten bis zum Widerruf (K19).
    pub retired: Option<Utc>,
}
