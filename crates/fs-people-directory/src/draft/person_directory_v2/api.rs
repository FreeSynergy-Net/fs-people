//! Der Trait `PersonDirectory` v2 (PROBE): v1 unverändert plus `end_situation` (N3): 15 Methoden. Die Zweck-Methoden (N6) stehen im eigenen Trait `PurposeCatalog` (`catalog.rs`).
//! Keine Signatur einer v1-Methode ändert sich; Typen kommen aus diesem Ordner.
//!
//! Der Trait `PersonDirectory` und seine Fähigkeitskennungen.
//!
//! Pattern: Repository (Trait) mit Null Object: keine Standard-Implementierung; was ein Anbieter nicht kann,
//! beantwortet er ausdrücklich mit `not_possible` (`Error::Unsupported`).

use super::{
    Candidate, Capabilities, Consent, ConsentEntry, ConsentQuery, ContactAddress, DirectoryChange,
    Error, Excerpt, ExcerptRequest, ObjectId, Page, PersonRef, PersonView, Place, ProposeEntry,
    Purpose, PurposeDef, PurposeDraft, Reference, Resolved, Result, Revision, Suggest, When,
    Withdrawal,
};
use async_trait::async_trait;
use fs_types::Zone;

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
/// `consent.read`, `consent.check`, `consent.write`; v2: `consent.situation` (nur Programm-Konten der Situationsbesitzer),
/// `purpose.read` (alle angemeldeten Menschen), `purpose.write` (Verwaltung, Datenschutz). Mitglieder von Gruppen gibt es hier nicht (nur fs-groups,
/// `GroupMembers::members_of`), also auch kein `cardgroup.read`. `consent.check` ist ein eigenes kleines Recht NUR für
/// Programm-Konten (fs-messages, Newsletter), keine Voreinstellung für Rollen von Menschen; `consent.read` bleibt bei
/// Verwaltung und Datenschutz (Strategie 16.11.).
pub const CAPABILITIES: &[&str] = &[
    "person.read",
    "person.find",
    "person.pick",
    "person.propose",
    "person.excerpt",
    "consent.read",
    "consent.check",
    "consent.write",
    "consent.situation",
    "purpose.read",
    "purpose.write",
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
    /// dann ältester zuerst. Gelöschte und nicht freigegebene Einträge fehlen. KI: `Forbidden{person.find}`
    /// (kein Adress-Orakel).
    async fn find_by_address(&self, address: &ContactAddress) -> Result<Vec<PersonView>>;
    /// `person.pick`: Vorschläge für den Personenwähler; Reihenfolge = Rang. `limit == 0` oder über 50 ist `Invalid`.
    /// KI: sucht nie in Adressen, `Matched::Address` kommt nie vor.
    async fn suggest(&self, query: &Suggest) -> Result<Vec<Candidate>>;

    /// `person.propose`: Stapel für einen Sammelvorschlag öffnen; im `CompositeDirectory` geht das an das erste
    /// Glied mit `person.propose`, und die Id gilt nur bei dem Anbieter, der sie ausgegeben hat (Massenimport,
    /// Migration; ADR 0085 §1). Die Id ist die des Sammelvorschlags, den der Mensch gesammelt oder einzeln bestätigt
    /// (ADR 0058).
    async fn open_batch(&self, origin: &Reference) -> Result<ObjectId>;
    /// `person.propose`: Eintrag im Zustand `Proposed` anlegen, optional in einem offenen Stapel. Ist die Adresse
    /// schon bekannt, kommt der vorhandene Verweis zurück (erster aus `find_by_address`) und nichts wird angelegt.
    /// Nie wird etwas ohne Rückfrage mit einer bestehenden Person verknüpft. KI mit `address`:
    /// `Forbidden{person.propose}` (die Antwort würde die Adresse einer Person zuordnen).
    async fn propose(&self, entry: &ProposeEntry, batch: Option<ObjectId>) -> Result<PersonRef>;
    /// `person.excerpt`: Auszug der erbetenen Felder jetzt, für den genannten Zweck (ADR 0085 §3, Datensparsamkeit
    /// §4, Zweckbindung). Leere Feldliste: `Invalid`; gelöscht, nicht freigegeben oder unbekannt: `Error::NoExcerpt`
    /// mit Grund (bei `Unknown` fragt ein `CompositeDirectory` weiter). Braucht der Zweck eine Einwilligung
    /// (`Purpose::needs_consent`) und ist das Ziel `Person` oder `Profile`, muss `check_consent` mit Zweck, `scope`
    /// und `recipient` der Anfrage eine liefern, sonst `NoExcerpt{reason: NoConsent}`; ihre Id steht in
    /// `Excerpt::consent`. Die Einwilligung passt nach der Regel in `ConsentQuery` (Profil über `describes`,
    /// zusammengelegte Verweise über ihr Ziel). Fragt eine KI, fehlen sensible Felder spurlos (`missing`); fragt sie
    /// mit einem anderen Zweck als `AiAssistance`, ist die Antwort vor jeder anderen Prüfung
    /// `Forbidden{person.excerpt}` (kein Leck über `NoConsent`).
    ///
    /// v2 (N2): für `OwnUse` liefert der Anbieter ein sensibles Feld (`ExcerptField::sensitive`) nur aus einem EIGENEN
    /// Eintrag der fragenden Person oder wenn die beschriebene Person es ihr freigegeben hat (`FieldShares`, Stufe
    /// "ohne Jahr"/"mit Jahr"); sonst `Excerpt::withheld` mit Grund (`NotShared`, bei nicht erreichbarem Port
    /// `Unverifiable`, fail-closed), die Freigabe in `Excerpt::shares`. `Contract`, `LegalObligation` und Zwecke mit
    /// Einwilligung wie v1. KI: unverändert spurlos in `missing`, `withheld` bleibt leer. R2-N3: eine KI-Anfrage mit
    /// `AiAssistance` bindet `recipient` an die fragende KI.
    async fn excerpt(&self, person: &PersonRef, request: &ExcerptRequest) -> Result<Excerpt>;

    /// `consent.read`: alle Einwilligungen einer Person, auch widerrufene und aufbewahrte, älteste zuerst (Nachweis
    /// nach Art. 7 Abs. 1; Teil der Auskunft nach Art. 15 und des Exports nach Art. 20). Die Person selbst darf ihre
    /// eigenen immer lesen; andere brauchen das Recht `consent.read` (Verwaltung, Datenschutz; fs-access), das auch
    /// für gelöschte Personen gilt, solange `retain_until` läuft. Unbekannter Verweis: leere Liste. KI: `Forbidden`.
    async fn consents_of(&self, person: &PersonRef) -> Result<Vec<Consent>>;
    /// `consent.check`: Id einer aktiven Einwilligung, die die Abfrage deckt (Regel in `ConsentQuery`; mehrere: die
    /// älteste), sonst `None`. Verrät nichts über andere Zwecke (Datensparsamkeit). Zweck: das Zuschalten einer KI
    /// (`AiAssistance`, `scope` = Raum, je anwesender Person, ADR 0054) und `excerpt`. Das Recht halten
    /// Programm-Konten (siehe [`CAPABILITIES`]). KI: `Forbidden`.
    ///
    /// v2: `Lifetime::Until(t)` zählt nur für `jetzt < t` (halboffen, Uhr des Anbieters); danach hält der Anbieter
    /// beim nächsten Schreiben oder Aufräumlauf den Widerruf `SituationEnded` fest. Ein zurückgezogener eigener Zweck
    /// ändert nichts an bestehenden Einwilligungen (K19).
    async fn check_consent(&self, query: &ConsentQuery) -> Result<Option<ObjectId>>;
    /// `consent.write`: Einwilligung festhalten; `recorded_by` ist die fragende Person (ohne Anmeldung `Forbidden`).
    /// Die Person selbst darf für sich erfassen (`ByPerson`); für andere braucht es das Recht `consent.write`.
    /// Ziel muss `Found` sein, `AiAssistance` braucht `scope`, sonst `Invalid`. Jeder Aufruf legt einen eigenen
    /// Eintrag an, auch wenn schon eine aktive passt: ein neuer Nachweis (Formular, Zeitpunkt, Fassung) wird nie still
    /// verworfen; `check_consent` nennt weiter die älteste. Ohne `expected` (Anlegen). KI: `Forbidden`.
    ///
    /// v2: `AiAssistance` braucht `scope` UND `lifetime`, `lifetime` an anderen Zwecken ist `Invalid`; `Photo` und
    /// Zwecke mit `recording` brauchen `notice` (Art passt zum Zweck; `Days(0)` ist `Invalid`), an anderen ist sie
    /// freiwillig und wird gespeichert; Origin `SituationEnded`/`Erasure` ist `Invalid`; ein zurückgezogener eigener
    /// Zweck ist `PurposeRetired`. Die Ablaufregel für `retain_until` steht in `retention.rs`.
    async fn record_consent(&self, entry: &ConsentEntry) -> Result<Consent>;
    /// `consent.write`: Einwilligung widerrufen (Art. 7 Abs. 3: so einfach wie erteilt). Die Person selbst darf ihre
    /// immer widerrufen, andere brauchen `consent.write`. Löscht nie; setzt `withdrawn` und `retain_until`.
    /// Ausdrücklich ohne `expected`: ein Eintrag ändert sich nur hierdurch, ein zweiter Widerruf liefert den ersten
    /// unverändert. Rangfolge (kein Orakel, wie `NotFound` vor `Forbidden` in `groups_v1`): fragt eine KI,
    /// `Forbidden` ohne Blick auf die Id; sonst ist eine unbekannte oder für die fragende Person nicht sichtbare Id
    /// (nicht die eigene und ohne `consent.read`) `UnknownConsent`, erst eine sichtbare ohne Recht `Forbidden`.
    ///
    /// v2 (R2-N2): auch `given_by` darf ohne `consent.write` widerrufen (Art. 7 Abs. 3; betreute Konten: die
    /// verantwortliche Person widerruft, liest aber sonst nichts). `retain_until` = `ConsentRetention::after` mit der
    /// JEWEILS GÜLTIGEN Einstellung und `Withdrawal::zone`; ändert sich die Einstellung, rechnet der Anbieter alle
    /// bestehenden Nachweise neu (`RetentionSettings`).
    async fn withdraw_consent(&self, id: ObjectId, withdrawal: &Withdrawal) -> Result<Consent>;
    /// v2 `consent.situation`: die Situation `scope` endet (Gruppe gelöscht, "KI erlaubt" aus, STOP KI, Konferenz vorzeitig
    /// zu). Widerruft ALLE aktiven `AiAssistance`-Einwilligungen mit diesem `scope` (Origin `SituationEnded`,
    /// `retain_until` wie bei jedem Widerruf); andere Zwecke und andere `scope` bleiben. Idempotent; Antwort = die Ids,
    /// die dieser Aufruf widerrufen hat. Nur Programm-Konten der Besitzer (fs-groups, fs-messages): sonst `Forbidden`;
    /// KI: `Forbidden`. `zone` wie `Withdrawal::zone` (Jahr der Aufbewahrung in dieser Zone, `None` = Instanz-Zone). Ein
    /// abgelaufenes `Lifetime::Until(t)` hält der Anbieter als Widerruf mit `at = When::At(t)` fest (nicht mit der Zeit
    /// des Aufräumlaufs), Origin `SituationEnded`, Zone der Instanz.
    async fn end_situation(
        &self,
        scope: &Reference,
        at: When,
        zone: Option<&Zone>,
        evidence: Option<&Reference>,
    ) -> Result<Vec<ObjectId>>;
}
