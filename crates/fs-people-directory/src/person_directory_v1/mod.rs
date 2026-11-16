//! Thema `people`, Teil `directory`, Ziel v1. Eigentümer: fs-people (ADR 0052, 0058, 0085 Abschnitt 3).
//!
//! Pattern: Repository (interface) + Null Object (ADR 0043/0045): JEDE Methode ist Pflicht, keine hat eine
//! Standard-Implementierung (Lehre `groups_v1`); wer etwas nicht kann, antwortet ausdrücklich mit
//! `not_possible("<kennung>")` (z. B. kann fs-groups keine Person vorschlagen) und meldet es in `capabilities`.
//!
//! Stand: Fixpunkt gelöst für das Delta „Einwilligungen“ (Nutzer 16.11., Antworten 27/29; Entscheidung 21: der
//! Trait ändert sich IN v1, solange es kein erstes Release gibt). Dieser ORDNER `person_directory_v1/` ist eine
//! Einheit (ADR 0043: kleine Dateien, jede <= 300 Zeilen); Hash = `/srv/FreeSynergy/.scratch/tools/hash-ordner.sh
//! person_directory_v1`. Dateien: `mod.rs` (dieser Kopf), `shared.rs` (`pub use` auf `fs-types` ab v0.0.22),
//! `view.rs` (Ansichten, Vorschläge, Änderungen), `excerpt.rs` (typisierter Auszug, Geburtstag), `consent.rs`
//! (Einwilligungen, Zwecke), `errors.rs` (typisierte Fehler), `api.rs` (Trait `PersonDirectory`).
//! `shared.rs` führt einen Alias: `Place` ist dort `PageCursor` (Weiterblätter-Stelle). `ContactAddress`
//! (Erreichbarkeit) und `Address` (Postanschrift) heißen wie in `fs-types`. Die Datei ist nicht byte-gleich mit
//! `appointments_v1/shared.rs` (gesperrter Kalender-Ordner, nicht angefasst). Weichen Namen, Konstruktoren
//! (`parse`/`new`) oder Zugriffsmethoden von `fs-types` ab, ist das ein Sprung.
//!
//! Zweck: die VERBRAUCHER-Seite von People. Kalender, Tasks, Groups und jedes Programm mit einem Feld "wer" halten nur
//! einen `PersonRef` (ADR 0085 Regel 3) und fragen diesen Trait nach Anzeige, Auswahl-Vorschlägen, Vorschlag neuer
//! Einträge ("vorgeschlagen", ADR 0058), der fragenden Person (`asker`, Ich-Profil), einem Auszug mit Zweck und nach
//! Einwilligungen (DSGVO Art. 7; erfassen, widerrufen, prüfen). Das volle Programm-Thema (Karten anlegen, ändern,
//! löschen, Rollen, Beziehungen) ist NICHT hier, sondern `People` v1 (eigener Trait, PLAN.md §2.3). Die DSGVO-Auskunft
//! je Person sammelt der Rechte-Teil-Trait von fs-access (Antwort 22); `consents_of` ist der Teil von People.
//!
//! Anbieter (ADR 0045, mindestens zwei): `PeopleDirectory` (fs-people, über `PeopleRepo`), `RemoteDirectory` (Socket,
//! `people.capnp`), `LocalAddressBook` (die kleine lokale Ablage ohne installiertes People, ADR 0085 §3: JSContact-Kern
//! Id, Name, Adresse), `GroupsDirectory` (fs-groups: Gruppen als `PersonArt::Group` für `resolve`/`suggest`),
//! `CompositeDirectory` (fragt der Reihe nach, erste Antwort ungleich `Unknown` gilt), `FakeDirectory` (Testdaten).
//! Der Kalender behält seinen gesperrten Verbraucher-Trait (`appointments_v1/exchange.rs`); ein Adapter in
//! fs-calendars (eine Datei) bildet ihn auf diesen Trait ab. Die Abhängigkeit zeigt immer vom Verbraucher auf People.
//!
//! Regeln (Konformitätstests, PLAN.md §10): Wer fragt, ist Eigenschaft des Anbieters, kein Parameter. Ein Verweis wird
//! nie umgeschrieben (ADR 0085 §3); weicht das Ziel ab (andere `art`, zusammengelegtes Profil), nennt die Antwort den
//! gültigen Verweis in `PersonView.person`, der Verbraucher zieht nach und meldet. `abzug` ist in jeder Antwort
//! `None`: den Abzug schreibt das ausstellende Programm (fs-objects v5 `person_snapshot`) aus `excerpt`.
//! Mitglieder von Gruppen gibt es hier NICHT: sie liegen nur in fs-groups (`GroupMembers::members_of`, wirksame
//! Mitglieder; People-Gruppenkarten tragen keine, O6/O7). Composite regelt nur das Lesen; `open_batch`/`propose`
//! gehen an das erste Glied mit `person.propose`, die Einwilligungs-Methoden an das erste mit `consent.*`.
//! KI-Regel (Antwort 29, ADR 0054/0076): eine KI ist eine `Person` mit KI-Markierung, keine eigene Art. Fragt eine
//! KI, fehlen sensible Felder (`ExcerptField::sensitive`) spurlos in jedem Auszug, `PersonView.address` ist `None`,
//! `display`/`sort_key` haben keine Adress-Stufe, `suggest` sucht nie in Adressen; `Forbidden` sind jede
//! Einwilligungs-Methode, `find_by_address`, `propose` mit Adresse und `excerpt` mit einem Zweck außer `AiAssistance`.
//! Einwilligungen: `given_by` nennt Eltern oder Betreuung (Art. 8); die Passregel (`ConsentQuery`) trifft Profile über
//! `describes` und zusammengelegte Verweise über ihr Ziel.
//! Erweiterung: `#[non_exhaustive]` steht an allen Ausgabe-Enums (nie an Structs; Lehre `groups_v1`), damit
//! Verbraucher schon heute einen Rest-Arm haben. Vor dem ersten Release ändert sich v1 selbst (Entscheidung 21);
//! danach ändert jede Erweiterung den Ordner-Hash und kommt nur mit neuer Version (v2).
//! Speicher (ADR 0086): der Trait ist speicherneutral; jedes Feld hat eine typisierte Spalte (PLAN.md §3).

mod api;
mod consent;
mod errors;
mod excerpt;
pub mod shared;
mod view;

pub use api::*;
pub use consent::*;
pub use errors::*;
pub use excerpt::*;
pub use shared::*;
pub use view::*;
