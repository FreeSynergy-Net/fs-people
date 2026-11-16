//! Auszug für den eingefrorenen Abzug (ADR 0085 §3/§4): je Feld ein Wert mit dem Typ aus `fs-types` (ADR 0086),
//! kein untypisierter Text und kein Paar, das nicht zusammenpasst.
//!
//! Pattern: Value Object (typisierte Variante je Feld; `ExcerptValue::field` ist die einzige Zuordnung).

use super::{Address, EmailAddress, Name, PersonRef, PhoneNumber, Reference, Text, Utc};

/// Feld eines Auszugs (ADR 0085 §3/§4: je Dokumentart eine Feldliste, Datensparsamkeit). `non_exhaustive`: neue Felder
/// (z. B. USt-IdNr für Rechnungen an Organisationen) kommen additiv; ein Anbieter, der ein Feld nicht kennt, nennt
/// es in `Excerpt::missing`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ExcerptField {
    FullName,
    GivenName,
    Surname,
    Organization,
    Postal,
    Email,
    Phone,
}

/// Wert eines Auszugsfeldes, typisiert nach `fs-types`. Die Variante bestimmt das Feld (`field`), also gibt es keine
/// falschen Paare. `non_exhaustive`: Erweiterung gemeinsam mit `ExcerptField`, additiv.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExcerptValue {
    FullName(Name),
    GivenName(Text),
    Surname(Text),
    Organization(Text),
    /// Postanschrift (`fs_types::Address`, `JSContact` `Address`), nicht die Erreichbarkeit (`ContactAddress`).
    Postal(Address),
    Email(EmailAddress),
    Phone(PhoneNumber),
}

impl ExcerptValue {
    /// Das Feld, zu dem dieser Wert gehört.
    #[must_use]
    pub fn field(&self) -> ExcerptField {
        match self {
            Self::FullName(_) => ExcerptField::FullName,
            Self::GivenName(_) => ExcerptField::GivenName,
            Self::Surname(_) => ExcerptField::Surname,
            Self::Organization(_) => ExcerptField::Organization,
            Self::Postal(_) => ExcerptField::Postal,
            Self::Email(_) => ExcerptField::Email,
            Self::Phone(_) => ExcerptField::Phone,
        }
    }
}

/// Auszug zum Zeitpunkt `captured`: NUR die erbetenen Felder; was fehlt, steht in `missing` (nie erfunden).
/// Daraus schreibt das ausstellende Programm die Zeile `person_snapshot` (fs-objects v5) und setzt `abzug`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Excerpt {
    pub person: PersonRef,
    /// Profil, aus dem der Auszug gebildet wurde (`quelle` des Abzugs).
    pub source: Reference,
    pub captured: Utc,
    /// Je erbetenem und vorhandenem Feld ein Wert; das Feld ergibt sich aus `ExcerptValue::field`.
    pub values: Vec<ExcerptValue>,
    pub missing: Vec<ExcerptField>,
}
