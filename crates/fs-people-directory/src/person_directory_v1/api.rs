//! Der Trait `PersonDirectory` und seine Fähigkeitskennungen.
//!
//! Pattern: Repository (Trait) mit Null Object: keine Standard-Implementierung; was ein Anbieter nicht kann,
//! beantwortet er ausdrücklich mit `not_possible` (`Error::Unsupported`).

use super::{
    Candidate, Capabilities, Consent, ConsentEntry, ConsentQuery, ContactAddress, DirectoryChange,
    Error, Excerpt, ExcerptRequest, ObjectId, Page, PersonRef, PersonView, Place, ProposeEntry,
    Reference, Resolved, Result, Suggest, Withdrawal,
};
use async_trait::async_trait;

/// Das ausdrückliche "nicht möglich" einer Methode, die der Anbieter nicht kann.
///
/// # Errors
/// Immer `Error::Unsupported` mit der Fähigkeitskennung.
pub fn not_possible<T>(capability: &str) -> Result<T> {
    Err(Error::Unsupported {
        capability: capability.to_owned(),
        reason: "this provider does not offer it".to_owned(),
    })
}

/// Fähigkeitskennungen dieses Teils (Wortlaut für `capabilities`, Manifest, `access.toml` und `people.capnp`).
/// `person.read`, `person.find` gibt es schon (people.capnp); neu: `person.pick`, `person.propose`, `person.excerpt`,
/// `consent.read`, `consent.check`, `consent.write`. Mitglieder von Gruppen gibt es hier nicht (nur fs-groups,
/// `GroupMembers::members_of`), also auch kein `cardgroup.read`.
pub const CAPABILITIES: &[&str] = &[
    "person.read",
    "person.find",
    "person.pick",
    "person.propose",
    "person.excerpt",
    "consent.read",
    "consent.check",
    "consent.write",
];

/// Personen-Verzeichnis: Verweise auflösen, die fragende Person nennen, Personen für eine Auswahl vorschlagen,
/// Unbekannte als "vorgeschlagen" anlegen, Auszug mit Zweck liefern, Einwilligungen festhalten und prüfen.
/// Mitglieder von Gruppen liefert dieser Trait nicht (nur fs-groups, `GroupMembers::members_of`).
///
/// Ziel v1 (ADR 0045). Wer fragt, ist Eigenschaft des Anbieters (die fragende Person), kein Parameter; `asker` macht
/// sie lesbar. Rechte (`consent.*` für andere als sich selbst) prüft der Anbieter über fs-access.
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
    /// Glied mit `person.propose`, und die Id gilt nur bei dem Anbieter, der sie ausgegeben hat (Massenimport,
    /// Migration; ADR 0085 §1). Die Id ist die des Sammelvorschlags, den der Mensch gesammelt oder einzeln bestätigt
    /// (ADR 0058).
    async fn open_batch(&self, origin: &Reference) -> Result<ObjectId>;
    /// `person.propose`: Eintrag im Zustand `Proposed` anlegen, optional in einem offenen Stapel. Ist die Adresse
    /// schon bekannt, kommt der vorhandene Verweis zurück (erster aus `find_by_address`) und nichts wird angelegt.
    /// Nie wird etwas ohne Rückfrage mit einer bestehenden Person verknüpft.
    async fn propose(&self, entry: &ProposeEntry, batch: Option<ObjectId>) -> Result<PersonRef>;
    /// `person.excerpt`: Auszug der erbetenen Felder jetzt, für den genannten Zweck (ADR 0085 §3, Datensparsamkeit
    /// §4, Zweckbindung). Leere Feldliste: `Invalid`; gelöscht, nicht freigegeben oder unbekannt: `Error::NoExcerpt`
    /// mit Grund (bei `Unknown` fragt ein `CompositeDirectory` weiter). Braucht der Zweck eine Einwilligung
    /// (`Purpose::needs_consent`) und ist das Ziel `Person` oder `Profile`, muss `check_consent` mit Zweck, `scope`
    /// und `recipient` der Anfrage eine liefern, sonst `NoExcerpt{reason: NoConsent}`; ihre Id steht in
    /// `Excerpt::consent`. Fragt eine KI, fehlen sensible Felder spurlos (`missing`).
    async fn excerpt(&self, person: &PersonRef, request: &ExcerptRequest) -> Result<Excerpt>;

    /// `consent.read`: alle Einwilligungen einer Person, auch widerrufene und aufbewahrte, älteste zuerst (Nachweis
    /// nach Art. 7 Abs. 1; Teil der Auskunft nach Art. 15 und des Exports nach Art. 20). Die Person selbst darf ihre
    /// eigenen immer lesen; andere brauchen das Recht `consent.read` (Verwaltung, Datenschutz; fs-access), das auch
    /// für gelöschte Personen gilt, solange `retain_until` läuft. Unbekannter Verweis: leere Liste. KI: `Forbidden`.
    async fn consents_of(&self, person: &PersonRef) -> Result<Vec<Consent>>;
    /// `consent.check`: Id einer aktiven Einwilligung, die die Abfrage deckt (Regel in `ConsentQuery`; mehrere: die
    /// älteste), sonst `None`. Verrät nichts über andere Zwecke (Datensparsamkeit). Zweck: das Zuschalten einer KI
    /// (`AiAssistance`, `scope` = Raum, je anwesender Person, ADR 0054) und `excerpt`. KI: `Forbidden`.
    async fn check_consent(&self, query: &ConsentQuery) -> Result<Option<ObjectId>>;
    /// `consent.write`: Einwilligung festhalten; `recorded_by` ist die fragende Person (ohne Anmeldung `Forbidden`).
    /// Die Person selbst darf für sich erfassen (`ByPerson`); für andere braucht es das Recht `consent.write`.
    /// Ziel muss `Found` sein, `AiAssistance` braucht `scope`, sonst `Invalid`. Gibt es schon eine aktive mit gleicher
    /// Eingabe außer `given`/`evidence`, kommt sie zurück und nichts wird angelegt. Ohne `expected` (Anlegen).
    /// KI: `Forbidden`.
    async fn record_consent(&self, entry: &ConsentEntry) -> Result<Consent>;
    /// `consent.write`: Einwilligung widerrufen (Art. 7 Abs. 3: so einfach wie erteilt). Die Person selbst darf ihre
    /// immer widerrufen, andere brauchen `consent.write`. Löscht nie; setzt `withdrawn` und `retain_until`.
    /// Ausdrücklich ohne `expected`: ein Eintrag ändert sich nur hierdurch, ein zweiter Widerruf liefert den ersten
    /// unverändert. Unbekannte Id: `UnknownConsent`. KI: `Forbidden`.
    async fn withdraw_consent(&self, id: ObjectId, withdrawal: &Withdrawal) -> Result<Consent>;
}
