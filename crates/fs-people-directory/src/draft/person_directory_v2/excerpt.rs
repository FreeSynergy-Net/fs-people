//! v2 (PROBE): `Withheld`/`WithheldWhy`/`ShareUse` (N2: sensible Felder bei `OwnUse` nur aus eigenem Eintrag oder mit
//! Freigabe; `Excerpt::withheld`, `Excerpt::shares`).
//!
//! Auszug für den eingefrorenen Abzug und für Facetten (ADR 0085 §3/§4): je Feld ein Wert mit dem Typ aus
//! `fs-types` (ADR 0086), kein untypisierter Text und kein Paar, das nicht zusammenpasst. Jeder Auszug nennt seinen
//! Zweck (Zweckbindung, DSGVO Art. 5 Abs. 1 lit. b) und, wo nötig, die Einwilligung, die ihn trägt.
//!
//! Pattern: Value Object (typisierte Variante je Feld; `ExcerptValue::field` ist die einzige Zuordnung,
//! `ExcerptField::sensitive` die einzige KI-Regel, `MonthDay::new` die einzige Prüfung).

use super::{
    Address, Date, EmailAddress, Name, ObjectId, PersonRef, PhoneNumber, Purpose, Reference, Text,
    TypeProblem, Utc,
};

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
    /// Geburtstag (`JSContact` `anniversaries` mit `kind: birth`), mit Jahr, wenn bekannt.
    Birthday,
    /// Geburtstag OHNE Jahr (immer `MonthDay`, auch wenn das Jahr bekannt ist; Datensparsamkeit): z. B. für die
    /// Geburtstags-Facette des Kalenders, die kein Alter braucht.
    BirthMonthDay,
}

impl ExcerptField {
    /// Sensibel im Sinn der KI-Regel (Nutzer 16.11., Antwort 29): Anschrift, E-Mail, Telefon, Geburtstag (mit und
    /// ohne Jahr). Fragt eine KI (`asker` mit KI-Markierung), stehen diese Felder immer in `Excerpt::missing`, spurlos
    /// wie fehlend. Auch E-Mail (Strategie 16.11.): die KI schreibt den Text, versenden tut das Programm.
    #[must_use]
    pub fn sensitive(self) -> bool {
        matches!(
            self,
            Self::Postal | Self::Email | Self::Phone | Self::Birthday | Self::BirthMonthDay
        )
    }
}

/// Monat und Tag ohne Jahr (vCard `BDAY:--0415`, `JSContact` `PartialDate` ohne `year`; ADR 0085: kein `Date`).
/// Der 29. Februar ist erlaubt.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MonthDay {
    month: u8,
    day: u8,
}

impl MonthDay {
    /// Monat 1 bis 12, Tag im Monat (Februar bis 29).
    ///
    /// # Errors
    /// `TypeProblem::BadDate`, wenn es den Tag in keinem Jahr gibt.
    pub fn new(month: u8, day: u8) -> Result<Self, TypeProblem> {
        let last = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => 29,
            _ => return Err(TypeProblem::BadDate),
        };
        if day == 0 || day > last {
            return Err(TypeProblem::BadDate);
        }
        Ok(Self { month, day })
    }
    #[must_use]
    pub fn month(self) -> u8 {
        self.month
    }
    #[must_use]
    pub fn day(self) -> u8 {
        self.day
    }
}

/// Geburtstag: mit Jahr ein `Date`, ohne Jahr ein `MonthDay`. Geschlossene Form, daher nicht `non_exhaustive`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Birthday {
    Date(Date),
    MonthDay(MonthDay),
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
    Birthday(Birthday),
    BirthMonthDay(MonthDay),
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
            Self::Birthday(_) => ExcerptField::Birthday,
            Self::BirthMonthDay(_) => ExcerptField::BirthMonthDay,
        }
    }
}

/// Anfrage eines Auszugs: welche Felder, wofür, wo und für wen (die Felder der Einwilligungsprüfung wie in
/// `ConsentQuery`, ohne Person).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExcerptRequest {
    /// Nicht leer; Doppelte zählen einmal.
    pub fields: Vec<ExcerptField>,
    pub purpose: Purpose,
    /// Zusammenhang (Dokument, Verteiler, Raum); `None` = allgemein.
    pub scope: Option<Reference>,
    /// Empfänger bei Weitergabe; `None` = keiner.
    pub recipient: Option<PersonRef>,
}

/// Auszug zum Zeitpunkt `captured`: NUR die erbetenen Felder; was fehlt, steht in `missing` (nie erfunden).
/// Daraus schreibt das ausstellende Programm die Zeile `person_snapshot` (fs-objects v5) und setzt `abzug`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Excerpt {
    pub person: PersonRef,
    /// Profil, aus dem der Auszug gebildet wurde (`quelle` des Abzugs).
    pub source: Reference,
    pub captured: Utc,
    /// Der Zweck der Anfrage (Zweckbindung; geht in den Abzug).
    pub purpose: Purpose,
    /// Die Einwilligung, die den Auszug trägt (`check_consent`), wenn der Zweck eine braucht; sonst `None`.
    pub consent: Option<ObjectId>,
    /// Je erbetenem und vorhandenem Feld ein Wert; das Feld ergibt sich aus `ExcerptValue::field`.
    pub values: Vec<ExcerptValue>,
    pub missing: Vec<ExcerptField>,
    /// v2: erbeten, vorhanden, aber nicht geliefert, mit Grund (nie still; nie bei einer KI, dort `missing`).
    pub withheld: Vec<Withheld>,
    /// v2: welche Freigabe welches sensible Feld trägt (`OwnUse` aus fremdem Profil).
    pub shares: Vec<ShareUse>,
}

/// v2: warum ein Feld zurückgehalten wurde. `non_exhaustive`: Ausgabe-Enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum WithheldWhy {
    /// Die beschriebene Person hat das Feld der fragenden Person nicht freigegeben (oder abgelaufen/zurückgenommen).
    NotShared,
    /// Der Port zur Freigabe war nicht erreichbar (fail-closed: kein Wert).
    Unverifiable,
}

/// v2: ein zurückgehaltenes Feld.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Withheld {
    pub field: ExcerptField,
    pub why: WithheldWhy,
}

/// v2: Beleg, welche Freigabe (fs-access) den Wert eines sensiblen Feldes trägt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShareUse {
    pub field: ExcerptField,
    pub share: ObjectId,
}
