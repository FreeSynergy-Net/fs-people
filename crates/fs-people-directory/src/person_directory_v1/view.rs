//! Was der Verzeichnis-Trait liefert: Ansicht eines Verweises, Auswahl-Vorschläge, Vorschlag neuer Einträge,
//! Änderungen (der Auszug für den Abzug steht in `excerpt.rs`).
//!
//! Pattern: Value Object / Data Transfer Object (nur Daten, keine Logik; kein freier Name außerhalb von `PersonView`).

use super::{ContactAddress, FileRef, PersonArt, PersonRef, Reference, Revision, Utc};

/// Stand eines Eintrags (ADR 0058: vorgeschlagen, dann bestätigt; abgelehnte Paare kommen nicht wieder).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryState {
    /// Von einem Menschen angelegt oder bestätigt.
    Confirmed,
    /// Durch `propose` entstanden (Import, Migration, Einladung an unbekannte Adresse); noch niemand hat bestätigt.
    Proposed,
}

/// Die aktuelle Ansicht eines Verweises (nicht eingefroren). Der EINZIGE Ort, an dem ein Name zum Verbraucher
/// kommt; der Verbraucher speichert ihn nie (ADR 0085 Regel 3), er darf ihn nur zwischenspeichern.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonView {
    /// Gültiger Verweis: `art` aus dem Ziel; bei zusammengelegten Profilen das Ziel der Zusammenlegung. Weicht er vom
    /// gefragten ab, zieht der Verbraucher nach und meldet (ADR 0085 §3). `abzug` ist hier immer `None`.
    pub person: PersonRef,
    /// Anzeigename nach ADR 0032 (voller Name, sonst Bestandteile, Spitzname, Organisation, Adresse).
    pub display: String,
    /// Sortierschlüssel ("Nachname, Vorname"), Kleinschreibung, NFC.
    pub sort_key: String,
    /// Bevorzugte Adresse (kleinster `pref`, sonst erste), wenn es eine gibt und sie gezeigt werden darf.
    pub address: Option<ContactAddress>,
    pub state: EntryState,
    /// KI-Markierung (ADR 0054, 0076): eine `Person` mit Markierung, keine eigene Art.
    pub ai: bool,
    /// Profil ist mit dieser Person verknüpft (AS2 `describes`, ADR 0058); `None` = unverknüpft.
    pub describes: Option<PersonRef>,
    /// Bild der Karte (fs-files), wenn es eines gibt und es gezeigt werden darf.
    pub avatar: Option<FileRef>,
    /// Stand des Ziels; ein Verbraucher-Zwischenspeicher vergleicht ihn.
    pub rev: Revision,
}

/// Ergebnis der Auflösung eines Verweises. Nie ein Fehler, wenn der Anbieter erreichbar ist.
#[derive(Clone, Debug, PartialEq, Eq)]
// `Found` ist der Regelfall, die anderen Varianten sind klein: Boxen würde jede Übereinstimmung für alle Anbieter
// ändern.
#[allow(clippy::large_enum_variant)]
pub enum Resolved {
    Found(PersonView),
    /// Gelöscht: Löschmarke (AS2 `Tombstone`, ADR 0085 §4); Anzeige "gelöschte Person".
    Deleted {
        since: Utc,
    },
    /// Nicht (mehr) freigegeben (ADR 0058, ADR 0085 §4 Freigabe-Rücknahme): der Verweis löst nichts auf.
    NotShared,
    /// Diesem Anbieter unbekannt; ein `CompositeDirectory` fragt den nächsten.
    Unknown,
}

/// Worauf ein Auswahl-Vorschlag passt (für die Hervorhebung im Personenwähler). `non_exhaustive`: Erweiterung additiv.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Matched {
    Name,
    Nickname,
    Address,
    Organization,
}

/// Anfrage des Personenwählers (Teilnehmer, Zuständige, Mitglieder).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Suggest {
    /// Getippter Text; leer = die ersten Einträge in Namensordnung.
    pub text: String,
    /// Erlaubte Arten; leer = alle. Der Kalender fragt z. B. `Person`, `Profile`, `Group`.
    pub arten: Vec<PersonArt>,
    /// Höchstens so viele (1 bis 50; mehr ist `Invalid`).
    pub limit: u16,
    /// Auch Einträge im Zustand `Proposed` zeigen.
    pub include_proposed: bool,
}

/// Ein Vorschlag des Personenwählers. Die Reihenfolge der Liste ist der RANG (Vertrag, anders als sonst).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub view: PersonView,
    pub matched: Matched,
}

/// Neuer Eintrag im Zustand `Proposed` (ADR 0085 §3: nie ein freier Name im Verbraucher). Mindestens eins von
/// `display_hint` (nicht leer) und `address`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProposeEntry {
    /// Name, wie ihn die Quelle nannte (z. B. `CN=` einer Einladung); landet nur im Eintrag, nie beim Verbraucher.
    pub display_hint: String,
    pub address: Option<ContactAddress>,
    /// Herkunft (ADR 0085 §1 `Origin`): Datei `urn:sha256:…`, Konto-IRI oder Objekt, das den Vorschlag auslöste.
    pub origin: Option<Reference>,
}

/// Art einer Änderung im Verzeichnis (für Zwischenspeicher und Live-Anzeige der Verbraucher). `non_exhaustive`:
/// Erweiterung ist additiv; ein Verbraucher behandelt Unbekanntes wie `Changed` (neu `resolve`).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ChangeKind {
    Created,
    /// Anzeige, Adresse, Art, Bild, Zustand oder Verknüpfung geändert.
    Changed,
    Deleted,
    NotShared,
    /// Aus dem Papierkorb zurückgeholt (People v1 `restore`) oder wieder freigegeben (vorher `NotShared`); `resolve`
    /// liefert danach wieder `Found`.
    Restored,
    /// Zusammengelegt: der Verweis bleibt gültig und löst künftig auf `into` auf.
    Merged {
        into: PersonRef,
    },
}

/// Eine Änderung im Verzeichnis, älteste zuerst.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectoryChange {
    pub person: PersonRef,
    pub kind: ChangeKind,
    pub at: Utc,
}
