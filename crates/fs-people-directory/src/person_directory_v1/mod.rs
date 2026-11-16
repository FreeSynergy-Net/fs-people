//! Thema `people`, Teil `directory`, Ziel v1. Eigentümer: fs-people (ADR 0052, 0058, 0085 Abschnitt 3).
//!
//! KANDIDAT 1, NICHT GESPERRT (Pilot fs-people, Stufe 2, 2026-11-16). Dieser ORDNER `person_directory_v1/` wird
//! nach `strategie-ok` als Ganzes gesperrt (ADR 0043: kleine Dateien, jede <= 300 Zeilen). Hash =
//! `/srv/FreeSynergy/.scratch/tools/hash-ordner.sh person_directory_v1`. Dateien: `mod.rs` (dieser Kopf),
//! `shared.rs` (`pub use` auf `fs-types` ab v0.0.22, fs-interfaces), `view.rs` (Ansichten, Vorschläge,
//! Änderungen), `excerpt.rs` (typisierter Abzug-Auszug), `errors.rs` (typisierte Fehler), `api.rs` (Trait
//! `PersonDirectory`).
//! Eine Stand-in-Kopie gibt es nicht mehr. `shared.rs` führt einen Alias: `Place` ist dort `PageCursor`
//! (Weiterblätter-Stelle). `ContactAddress` (Erreichbarkeit) und `Address` (Postanschrift) heißen wie in `fs-types`.
//! Die Datei ist nicht byte-gleich mit `appointments_v1/shared.rs` (gesperrter Kalender-Ordner, nicht angefasst).
//! Weichen Namen, Konstruktoren (`parse`/`new`) oder Zugriffsmethoden von `fs-types` ab, ist das ein Sprung.
//!
//! Zweck: die VERBRAUCHER-Seite von People. Kalender, Tasks, Groups und jedes Programm mit einem Feld "wer" halten nur
//! einen `PersonRef` (ADR 0085 Regel 3) und fragen diesen Trait nach Anzeige, Auswahl-Vorschlägen, Vorschlag neuer
//! Einträge ("vorgeschlagen", ADR 0058), der fragenden Person (`asker`, Ich-Profil) und einem Auszug für den
//! eingefrorenen Abzug. Das volle
//! Programm-Thema (Karten anlegen, ändern, löschen, Rollen, Beziehungen) ist NICHT hier, sondern `People` v1 (eigener
//! Trait, eigene Sperre, PLAN.md §2.3); dieser kleine Trait wird ZUERST gesperrt, damit niemand auf People wartet.
//!
//! Anbieter (ADR 0045, mindestens zwei): `PeopleDirectory` (fs-people, über `PeopleRepo`), `RemoteDirectory` (Socket,
//! `people.capnp`), `LocalAddressBook` (die kleine lokale Ablage ohne installiertes People, ADR 0085 §3: JSContact-Kern
//! Id, Name, Adresse), `GroupsDirectory` (fs-groups: Gruppen als `PersonArt::Group` für `resolve`/`suggest`),
//! `CompositeDirectory` (fragt
//! der Reihe nach, erste Antwort ungleich `Unknown` gilt), `FakeDirectory` (Testdaten). Der Kalender behält seinen
//! gesperrten Verbraucher-Trait (`appointments_v1/exchange.rs`); ein Adapter in fs-calendars (eine Datei) bildet ihn
//! auf diesen Trait ab. Die Abhängigkeit zeigt immer vom Verbraucher auf People, nie umgekehrt.
//!
//! Pattern: Repository (interface) + Null Object (ADR 0043/0045) wie `appointments_v1`: Lesemethoden sind Pflicht,
//! Schreib- und Zusatzmethoden haben den Standard `not_possible("<kennung>")` (z. B. kann fs-groups keine Person
//! vorschlagen). `capabilities` meldet dasselbe.
//!
//! Regeln (Konformitätstests, PLAN.md §10): Wer fragt, ist Eigenschaft des Anbieters, kein Parameter. Ein Verweis wird
//! nie umgeschrieben (ADR 0085 §3); weicht das Ziel ab (andere `art`, zusammengelegtes Profil), nennt die Antwort den
//! gültigen Verweis in `PersonView.person`, der Verbraucher zieht nach und meldet. `abzug` ist in jeder Antwort
//! `None`: den Abzug schreibt das ausstellende Programm (fs-objects v5 `person_snapshot`) aus `excerpt`.
//! Mitglieder von Gruppen gibt es hier NICHT: sie liegen nur in fs-groups (`GroupMembers::members_of`, wirksame
//! Mitglieder; People-Gruppenkarten tragen keine, O6/O7). Composite regelt nur das Lesen; `open_batch`/`propose`
//! gehen an das erste Glied mit `person.propose`. Erweiterung = additiv: `ExcerptField`, `ExcerptValue`, `ChangeKind`,
//! `Matched`, `NoExcerptReason` sind `non_exhaustive` (neue Felder, z. B. USt-IdNr, brechen keinen
//! Verbraucher). Varianten sind additiv, aber jede Erweiterung ändert den Ordner-Hash und damit die
//! Version, also gibt es sie nur mit neuer Version (v2).
//! Speicher (ADR 0086): der Trait ist speicherneutral; jedes Feld hat eine typisierte Spalte (PLAN.md §3).

mod api;
mod errors;
mod excerpt;
pub mod shared;
mod view;

pub use api::*;
pub use errors::*;
pub use excerpt::*;
pub use shared::*;
pub use view::*;
