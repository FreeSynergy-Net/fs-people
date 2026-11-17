//! v2 (PROBE; angehängt, nichts entfernt): `Lifetime` und `end_situation` (N3), `SituationEnded`, `RecordingNotice` (Aufnahmedauer
//! mitteilen, b7 17.11.), `Withdrawal::zone` (Jahr der Aufbewahrung in der gespeicherten Zone).
//!
//! Einwilligungen (DSGVO Art. 7) und Zwecke (Zweckbindung, Art. 5 Abs. 1 lit. b): was eine Person wofür, wann, wo
//! und wem erlaubt hat, mit Herkunft und Beleg (Nutzer 16.11., Antwort 27). Felder nach ISO/IEC TS 27560
//! (Einwilligungs-Aufzeichnung) und W3C DPV (Zwecke, ADR 0029); Export als Teil von People v1 `Exchange`.
//!
//! Pattern: Value Object (Aufzeichnung, Eingaben, Abfrage; `Purpose::needs_consent` ist die einzige Regel).
//!
//! Ehrlich (Nutzer): die Aufzeichnung ist technisch umgehbar (wer Daten hat, kann sie nutzen); sie ist der Nachweis
//! nach Art. 7 Abs. 1 und die Grundlage, auf der `excerpt` Zwecke mit Einwilligungspflicht prüft.
//! Löschen und Aufbewahren (ADR 0073 §7, ADR 0085 §4): ein Widerruf löscht nie, er setzt `withdrawn` (der Nachweis
//! bleibt). Wird die Person gelöscht, gelten ihre aktiven Einwilligungen als widerrufen (`ConsentOrigin::Erasure`).
//! `retain_until` setzt der Anbieter beim Widerruf aus einer Einstellung von People; danach löscht er die
//! Aufzeichnung wirklich, auch aus dem Verlauf.
//! Befristung: in v1 gibt es kein Ablaufdatum; eine Einwilligung gilt bis zum Widerruf, eine enge Geltung (ein
//! Ereignis, ein Raum, ein Verteiler) drückt `scope` aus.

use super::{Date, ObjectId, PersonRef, Reference, Revision, Utc};
use fs_types::Zone;

/// Zweck einer Verarbeitung (DSGVO Art. 5 Abs. 1 lit. b; Begriffe nach W3C DPV, ADR 0029). Offene Menge über
/// `Other`. `non_exhaustive`: Verbraucher brauchen einen Rest-Arm.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Purpose {
    /// Vertrag erfüllen, auch Buchung, Bestellung, Mitgliedschaft (Art. 6 Abs. 1 lit. b; DPV `ServiceProvision`).
    Contract,
    /// Rechtliche Pflicht, z. B. Rechnung, Quittung, Aufbewahrung (Art. 6 Abs. 1 lit. c; DPV `LegalCompliance`).
    LegalObligation,
    /// Eigene Ablage der fragenden Person, nur für sie sichtbar (z. B. Geburtstag im eigenen Kalender). Für rein
    /// persönliche Tätigkeit gilt die DSGVO nicht (Art. 2 Abs. 2 lit. c); ob das zutrifft, sagt der Mensch.
    OwnUse,
    /// Fotos der Person aufnehmen und nutzen.
    Photo,
    /// Veröffentlichung (Website, Aushang, Mitgliederliste nach außen).
    Publication,
    /// Newsletter und Rundschreiben (DPV `DirectMarketing`).
    Newsletter,
    /// KI-Hilfe: eine KI wird zugeschaltet und sieht Daten der Person (ADR 0054: alle Anwesenden stimmen VORHER zu).
    AiAssistance,
    /// Weitergabe an Dritte (Empfänger in `ConsentEntry::recipient`).
    DisclosureToThirdParty,
    /// Anderer Zweck: IRI eines DPV-Begriffs oder ein eigener Zweck des Betreibers.
    Other(Reference),
}

impl Purpose {
    /// Braucht dieser Zweck eine aktive Einwilligung? Nein nur für `Contract`, `LegalObligation`, `OwnUse`
    /// (andere Rechtsgrundlage); alles andere, auch `Other`, ja.
    #[must_use]
    pub fn needs_consent(&self) -> bool {
        !matches!(self, Self::Contract | Self::LegalObligation | Self::OwnUse)
    }
}

/// Zeitpunkt oder Datum (Normalisierung: Zeiten in UTC, ein Datum bleibt Datum). Ein Papierformular nennt oft nur
/// den Tag; eine Zustimmung im Programm den genauen Zeitpunkt. Geschlossene Form, daher nicht `non_exhaustive`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum When {
    At(Utc),
    On(Date),
}

/// Woher die Angabe stammt. `non_exhaustive`: Erweiterung additiv.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ConsentOrigin {
    /// Die Person selbst im Programm (Ich-Karte; auch die Zustimmung beim Zuschalten einer KI).
    ByPerson,
    /// Formular (Papier oder online); der Beleg gehört nach `evidence`.
    Form,
    /// Import aus einer Datei oder einem anderen Programm (`evidence` = Datei `urn:sha256:…`).
    Import,
    /// Mündlich, von der erfassenden Person festgehalten.
    Oral,
    /// Nur beim Widerruf: die Person wurde gelöscht (People v1 `delete`).
    Erasure,
    /// v2, nur beim Widerruf: die Situation (`scope`) ist zu Ende (`end_situation`); wie `Erasure` nie mit
    /// `record_consent` erfassbar.
    SituationEnded,
}

/// Eingabe für `record_consent`: was die Person erlaubt hat. Wer erfasst, ist der Anbieter (`asker`), kein Feld.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsentEntry {
    /// Die betroffene Person (Art `Person` oder `Profile`).
    pub person: PersonRef,
    /// Wer die Einwilligung erteilt hat: `None` = die Person selbst; sonst Eltern oder Sorgeberechtigte eines Kindes
    /// (DSGVO Art. 8) oder eine rechtliche Betreuung. Die Altersgrenze (Voreinstellung des Programms: 16 Jahre,
    /// einstellbar; Mitgliedstaaten dürfen bis 13 senken) ist eine Einstellung von People, keine Rechtsberatung.
    /// Die Passregel (`ConsentQuery`) sieht nur `person`, nicht `given_by`.
    pub given_by: Option<PersonRef>,
    pub purpose: Purpose,
    /// Wo die Einwilligung gilt: Raum, Unterhaltung, Dokument, Verteiler; `None` = allgemein für den Zweck. Bei
    /// `AiAssistance` Pflicht (Zustimmung je Raum, ADR 0054 Pkt 4/5; sonst `Invalid`).
    pub scope: Option<Reference>,
    /// Wem: der Dritte bei `DisclosureToThirdParty`, das KI-Konto bei `AiAssistance`; `None` = nicht eingeschränkt.
    pub recipient: Option<PersonRef>,
    pub given: When,
    pub origin: ConsentOrigin,
    /// Beleg: Datei des Formulars (`urn:sha256:…`), Objekt oder Nachricht; `None` = keiner (z. B. mündlich).
    pub evidence: Option<Reference>,
    /// Fassung des Einwilligungstexts, dem zugestimmt wurde (`urn:sha256:…` oder IRI mit Version); `None` = keiner.
    pub text: Option<Reference>,
    /// v2: Laufzeit. Pflicht bei `AiAssistance` (sonst `Invalid`), sonst nur `None` (Befristung bleibt außen, H5).
    pub lifetime: Option<Lifetime>,
    /// v2: Aufnahmedauer-Hinweis. Pflicht bei `Photo` und bei eigenen Zwecken mit `recording`; sonst freiwillig.
    pub notice: Option<RecordingNotice>,
}

/// v2: wie lange eine Zustimmung gilt. Geschlossene Form wie `When`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Lifetime {
    /// Solange die Situation `scope` besteht (Arbeitsgruppe, Chat-Raum). Ende meldet der Besitzer per `end_situation`.
    WhileScope,
    /// Bis zu diesem Zeitpunkt (Konferenz-Ende), halboffen: gilt für `jetzt < t` (Uhr des Anbieters).
    Until(Utc),
}

// MOVE-NOTICE: `RecordingKind`, `RecordingKeep` und `RecordingNotice` ziehen mit dem nächsten fs-types-Tag nach
// fs-types um. Angekündigte Änderung: NUR diese drei Definitionen werden durch `pub use fs_types::...` mit gleichen
// Namen, Feldern, Varianten und gleichem Sinn ersetzt (neuer Ordner-Hash mit `vorgaenger`, kein v3); jede andere
// Abweichung ist ein Sprung.
/// v2: was aufgenommen wird. `non_exhaustive`: Ausgabe-Enum.
// VORLÄUFIG hier; zieht mit dem nächsten fs-types-Tag nach fs-types um (b7 17.11., Frage 12).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RecordingKind {
    Photo,
    Audio,
    Video,
    Transcript,
    Location,
}

/// v2: wie lange Aufnahmen im Normalfall gespeichert werden. Geschlossene Form.
// VORLÄUFIG hier; zieht mit dem nächsten fs-types-Tag nach fs-types um.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RecordingKeep {
    /// Tage, mindestens 1 (`Days(0)` ist `Invalid`).
    Days(u32),
    UntilSituationEnds,
    /// Nur mit ausdrücklichem Text (Notfall-Daten).
    UntilDeletedByHand,
}

/// v2: Pflichtfeld der Einwilligung beim Aufnehmen oder Tracken: das Programm nennt die Dauer von selbst.
// VORLÄUFIG hier; zieht mit dem nächsten fs-types-Tag nach fs-types um.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordingNotice {
    pub kind: RecordingKind,
    pub default_retention: RecordingKeep,
    /// Fassung des MF2-Hinweises `people-recording-notice`, die gezeigt wurde.
    pub text: Reference,
}

/// Eingabe für `withdraw_consent`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Withdrawal {
    pub at: When,
    /// v2: Zone, in der `at` gemeint war (Zeit = UTC + Zone). Das Jahr der Aufbewahrung zählt in dieser Zone; `None` =
    /// Zone der Instanz. Gilt nur für `When::At`.
    pub zone: Option<Zone>,
    pub origin: ConsentOrigin,
    pub evidence: Option<Reference>,
}

/// Ein festgehaltener Widerruf: Eingabe plus wer ihn erfasst hat.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Withdrawn {
    pub withdrawal: Withdrawal,
    pub recorded_by: PersonRef,
    pub recorded_at: Utc,
}

/// Eine Einwilligungs-Aufzeichnung. Aktiv = nicht widerrufen. Sie ändert sich nur durch den Widerruf.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Consent {
    pub id: ObjectId,
    pub rev: Revision,
    pub entry: ConsentEntry,
    /// Wer erfasst hat (die fragende Person beim Erfassen).
    pub recorded_by: PersonRef,
    pub recorded_at: Utc,
    pub withdrawn: Option<Withdrawn>,
    /// Aufbewahrung des Nachweises bis (gesetzt beim Widerruf); `None` = aktiv, bleibt.
    pub retain_until: Option<Date>,
}

/// Abfrage für `check_consent`: deckt eine aktive Einwilligung diesen Zweck hier und für diesen Empfänger? Eine
/// Einwilligung passt, wenn der Zweck gleich ist, ihr `scope`/`recipient` `None` oder gleich der Abfrage ist und ihr
/// `person` die gefragte Person trifft. Treffen (ADR 0058, 0085 §3; Verweise werden nie umgeschrieben):
/// - Ein zusammengelegter Verweis gilt als sein Ziel. Beim Zusammenlegen ziehen die Einwilligungen mit: sie bleiben
///   mit ihrem alten Verweis gespeichert und zählen für das Ziel (auch in `consents_of` des Ziels).
/// - Fragt man nach einem Profil, passen Einwilligungen am Profil und an der Person, die es beschreibt
///   (`PersonView::describes`); die Einwilligung einer Person deckt alle ihre Profile.
/// - Eine Einwilligung an einem Profil deckt nur dieses Profil, nie die Person oder ihre anderen Profile.
/// - Wird die Verknüpfung (`describes`) gelöst, deckt die Einwilligung der Person das Profil nicht mehr.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsentQuery {
    pub person: PersonRef,
    pub purpose: Purpose,
    pub scope: Option<Reference>,
    pub recipient: Option<PersonRef>,
}
