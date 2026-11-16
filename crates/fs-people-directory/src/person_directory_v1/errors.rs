//! Typisierte Fehler des Ordners `person_directory_v1/`. Jede Ursache ist eine Variante mit typisierten Zusatzfeldern;
//! `Display` dient nur der Anzeige (Entwicklertext, englisch); die Person sieht Übersetzungen über die Kennungen
//! (ADR 0057). Eine nicht gefundene Person ist KEIN Fehler, sondern `Resolved::Unknown`/`Deleted`/`NotShared`; nur
//! eine unbekannte Einwilligungs-Id beim Widerruf ist einer (`UnknownConsent`), weil dort geschrieben wird.
//!
//! Pattern: Error Enum (typisierte Ursache + Zusatzfelder), `Display` nur zur Anzeige.

use super::{BasicValue, ObjectId, PersonRef, Problem, TypeProblem};
use std::fmt;

/// Warum der Anbieter gerade nicht antwortet. `non_exhaustive`: Verbraucher brauchen einen Rest-Arm.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Unreachable {
    /// Dienst läuft nicht (Socket fehlt, Programm nicht installiert).
    NotRunning,
    /// Dienst antwortet nicht im Zeitlimit.
    Timeout,
    /// Speicher gesperrt (verschlüsselt und nicht entsperrt, ADR 0067).
    Locked,
    /// Speicher defekt oder nicht lesbar.
    Storage,
}

/// Warum `excerpt` keinen Auszug liefert. `Unknown` lässt ein `CompositeDirectory` den nächsten Anbieter fragen,
/// `Deleted`, `NotShared` und `NoConsent` sind endgültig für diese Anfrage (ADR 0085 §4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum NoExcerptReason {
    /// Der Eintrag ist gelöscht (Löschmarke).
    Deleted,
    /// Die Freigabe ist zurückgenommen.
    NotShared,
    /// Dem Anbieter unbekannter Verweis.
    Unknown,
    /// Der Zweck braucht eine Einwilligung (`Purpose::needs_consent`), und keine aktive passt (`check_consent`).
    /// Endgültig für diese Anfrage; der Verweis selbst löst weiter auf.
    NoConsent,
}

/// Fehler des Verzeichnisses. `non_exhaustive`: Verbraucher brauchen einen Rest-Arm.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// Der Anbieter kann es ausdrücklich nicht (ADR 0045), mit Fähigkeitskennung.
    Unsupported { capability: String, reason: String },
    /// Die fragende Person darf es nicht (Vorschlag in einem fremden Adressbuch; Einwilligungen ohne Recht; jede
    /// Einwilligungs-Methode, wenn eine KI fragt, unabhängig vom Ziel), mit Fähigkeitskennung.
    Forbidden { capability: String },
    /// Eingabe ungültig (leerer Vorschlag, `limit` außerhalb 1 bis 50, Feldliste leer, Einwilligung für einen
    /// Verweis, der nicht `Found` ist, `AiAssistance` ohne `scope`); nichts geschrieben.
    Invalid(Vec<Problem>),
    /// `excerpt` auf einen gelöschten, nicht freigegebenen oder unbekannten Verweis oder ohne nötige Einwilligung:
    /// kein Auszug (ADR 0085 §4).
    NoExcerpt {
        person: PersonRef,
        reason: NoExcerptReason,
    },
    /// `propose` an einen Stapel, der nicht (mehr) offen ist; auch eine nie geöffnete oder von einem anderen Anbieter
    /// ausgegebene Id.
    BatchClosed(ObjectId),
    /// `withdraw_consent` mit einer Id, die dieser Anbieter nicht (mehr) kennt.
    UnknownConsent(ObjectId),
    /// Der Anbieter ist nicht erreichbar (Dienst, Zeitlimit, gesperrter oder defekter Speicher); ein
    /// `CompositeDirectory` fragt dann den nächsten Anbieter, am Ende bleibt `Unavailable`.
    Unavailable(Unreachable),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<TypeProblem> for Error {
    fn from(p: TypeProblem) -> Self {
        Self::Invalid(vec![Problem::from_type("", p)])
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported { capability, .. } => write!(f, "not offered: {capability}"),
            Self::Forbidden { capability } => write!(f, "not allowed: {capability}"),
            Self::Invalid(p) => write!(f, "invalid input ({} problems)", p.len()),
            Self::NoExcerpt { person, reason } => {
                write!(f, "no excerpt for {} ({reason:?})", person.id.canonical())
            }
            Self::BatchClosed(_) => write!(f, "proposal batch is closed"),
            Self::UnknownConsent(_) => write!(f, "unknown consent record"),
            Self::Unavailable(u) => write!(f, "directory unavailable: {u:?}"),
        }
    }
}
impl std::error::Error for Error {}
