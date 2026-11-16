//! Der Trait `PersonDirectory` und seine Fähigkeitskennungen.
//!
//! Pattern: Repository (Trait) mit Null Object (nicht überschriebene Methoden sind `Unsupported`).

use super::{
    Candidate, Capabilities, ContactAddress, DirectoryChange, Error, Excerpt, ExcerptField,
    ObjectId, Page, PersonRef, PersonView, Place, ProposeEntry, Reference, Resolved, Result,
    Suggest,
};
use async_trait::async_trait;

/// Das ausdrückliche "nicht möglich" einer nicht überschriebenen Methode.
///
/// # Errors
/// Immer `Error::Unsupported` mit der Fähigkeitskennung.
pub fn not_possible<T>(capability: &str) -> Result<T> {
    Err(Error::Unsupported {
        capability: capability.to_owned(),
        reason: "this provider does not offer it".to_owned(),
    })
}

/// Fähigkeitskennungen dieses Teils (Wortlaut für `capabilities`, Manifest und `people.capnp`). `person.read`,
/// `person.find` gibt es schon (people.capnp); neu: `person.pick`, `person.propose`, `person.excerpt`. Mitglieder von
/// Gruppen gibt es hier nicht (nur fs-groups, `GroupMembers::members_of`), also auch kein `cardgroup.read`.
pub const CAPABILITIES: &[&str] = &[
    "person.read",
    "person.find",
    "person.pick",
    "person.propose",
    "person.excerpt",
];

/// Personen-Verzeichnis: Verweise auflösen, die fragende Person nennen, Personen für eine Auswahl vorschlagen,
/// Unbekannte als "vorgeschlagen" anlegen, Auszug für den Abzug liefern. Mitglieder von Gruppen liefert dieser Trait
/// nicht (nur fs-groups, `GroupMembers::members_of`).
///
/// Ziel v1 (ADR 0045). Wer fragt, ist Eigenschaft des Anbieters (die fragende Person), kein Parameter; `asker` macht
/// sie lesbar.
#[async_trait]
pub trait PersonDirectory: Send + Sync {
    /// Was dieser Anbieter kann, je Kennung aus [`CAPABILITIES`].
    fn capabilities(&self) -> Capabilities;

    /// `person.read`: aktuelle Ansicht eines Verweises.
    async fn resolve(&self, person: &PersonRef) -> Result<Resolved>;
    /// `person.read`: die fragende Person aus dem Ich-Profil ("Ich-Karte", People O11). Zweck: "Mir zuweisen" in
    /// Tasks und das Ich-Profil für Groups. `None` = nicht angemeldet oder keine Ich-Karte gesetzt (die Anzeige sagt
    /// dann "Ich-Karte festlegen"); kein Fehler.
    async fn asker(&self) -> Result<Option<PersonRef>>;
    /// `person.read`: viele auf einmal (Listen, Raster). Antwort hat GENAU die Länge und Reihenfolge der Eingabe
    /// (Doppelte erlaubt); eine Höchstlänge nennt der Anbieter (über den Socket begrenzt).
    async fn resolve_many(&self, people: &[PersonRef]) -> Result<Vec<Resolved>>;
    /// `person.read`: Änderungen seit `since` (`None` = ab jetzt), älteste zuerst, für Zwischenspeicher der
    /// Verbraucher. `size == 0` ist `Invalid`. `next` ist immer `Some` (Stelle zum Weiterfragen,
    /// auch bei leerer Seite; anders als sonst bei `Page`);
    /// Wiederherstellen aus dem Papierkorb meldet `ChangeKind::Restored`.
    async fn changes(&self, since: Option<&Place>, size: usize) -> Result<Page<DirectoryChange>>;
    /// `person.find`: Einträge mit genau dieser Adresse (Vergleich nach der Regel in `ContactAddress`); mehrere möglich
    /// (Profile derselben Person, ADR 0058). Reihenfolge: `Confirmed` vor `Proposed`, dann `Person` vor `Profile`,
    /// dann ältester zuerst. Gelöschte und nicht freigegebene Einträge fehlen.
    async fn find_by_address(&self, address: &ContactAddress) -> Result<Vec<PersonView>>;
    /// `person.pick`: Vorschläge für den Personenwähler; Reihenfolge = Rang. `limit == 0` oder über 50 ist `Invalid`.
    async fn suggest(&self, query: &Suggest) -> Result<Vec<Candidate>>;

    /// `person.propose`: Stapel für einen Sammelvorschlag öffnen; im `CompositeDirectory` geht das an das erste
    /// Glied mit
    /// `person.propose`, und die Id gilt nur bei dem Anbieter, der sie ausgegeben hat (Massenimport, Migration;
    /// ADR 0085 §1). Die Id ist
    /// die des Sammelvorschlags, den der Mensch gesammelt oder einzeln bestätigt (ADR 0058).
    async fn open_batch(&self, _origin: &Reference) -> Result<ObjectId> {
        not_possible("person.propose")
    }
    /// `person.propose`: Eintrag im Zustand `Proposed` anlegen, optional in einem offenen Stapel. Ist die Adresse
    /// schon bekannt, kommt der vorhandene Verweis zurück (erster aus `find_by_address`) und nichts wird angelegt.
    /// Nie wird etwas ohne Rückfrage mit einer bestehenden Person verknüpft.
    async fn propose(&self, _entry: &ProposeEntry, _batch: Option<ObjectId>) -> Result<PersonRef> {
        not_possible("person.propose")
    }
    /// `person.excerpt`: Auszug der erbetenen Felder jetzt (ADR 0085 §3, Datensparsamkeit §4). Leere Feldliste:
    /// `Invalid`; gelöscht, nicht freigegeben oder unbekannt: `Error::NoExcerpt` mit Grund (bei `Unknown` fragt ein
    /// `CompositeDirectory` weiter).
    async fn excerpt(&self, _person: &PersonRef, _fields: &[ExcerptField]) -> Result<Excerpt> {
        not_possible("person.excerpt")
    }
}
