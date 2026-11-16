//! Reading and writing `people_snapshot`: the frozen excerpt of a person (ADR 0085 section 3) with its purpose, the
//! consent that carries it and the birthday, each in its own typed column. Insert-only: this file has no update.
//!
//! Pattern: Repository functions over a unit of work (`Conn`) + Value Object (`Snapshot`: the excerpt and what makes
//! it a document: its kind, the day it was issued, the day it may be deleted).
//!
//! Limits (stated, not hidden): the postal address keeps its full text and country code, not its components; a name
//! keeps its full text, not its components; `missing` is not stored (a snapshot is what was captured, not what was
//! absent).

use fs_objects_v5::v5::codec::{
    Row, art_of, art_word, damaged, date_to_int, get_id, get_int, get_opt_text, get_text, id_value,
    int, int_to_date, invalid, opt_text, parse_id, reference_of, text,
};
use fs_objects_v5::v5::{Conn, Result};
use fs_people_directory::{
    Address, Birthday, Date, EmailAddress, Excerpt, ExcerptValue, MonthDay, Name, ObjectId,
    PersonRef, PhoneNumber, Text, Utc,
};
use fs_types::BasicValue;

use super::tables::SNAPSHOT;
use super::words::{purpose_cols, purpose_of};
use super::{DocKind, birthday};

/// A snapshot row: the excerpt and what makes it a document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    /// The id of the snapshot (the `abzug` a booking points at).
    pub id: ObjectId,
    /// What kind of document it was issued for.
    pub doc_kind: DocKind,
    /// The day it was issued.
    pub issued: Date,
    /// The day it may be deleted (not before `issued`).
    pub keep_until: Date,
    /// The excerpt: person, source, instant, purpose, consent and the values.
    pub excerpt: Excerpt,
}

#[derive(Default)]
struct Slots {
    full_name: Option<String>,
    given_name: Option<String>,
    surname: Option<String>,
    organization: Option<String>,
    postal: Option<(String, String)>,
    email: Option<String>,
    phone: Option<String>,
    birthday: Option<Birthday>,
    birth_month_day: Option<MonthDay>,
}

fn slots_of(values: &[ExcerptValue]) -> Result<Slots> {
    let mut s = Slots::default();
    for v in values {
        let taken = match v {
            ExcerptValue::FullName(n) => s.full_name.replace(n.display()).is_some(),
            ExcerptValue::GivenName(t) => s.given_name.replace(t.canonical()).is_some(),
            ExcerptValue::Surname(t) => s.surname.replace(t.canonical()).is_some(),
            ExcerptValue::Organization(t) => s.organization.replace(t.canonical()).is_some(),
            ExcerptValue::Postal(a) => s
                .postal
                .replace((a.display(), a.country_code().to_owned()))
                .is_some(),
            ExcerptValue::Email(e) => s.email.replace(e.canonical()).is_some(),
            ExcerptValue::Phone(p) => s.phone.replace(p.canonical()).is_some(),
            ExcerptValue::Birthday(b) => s.birthday.replace(*b).is_some(),
            ExcerptValue::BirthMonthDay(m) => s.birth_month_day.replace(*m).is_some(),
            _ => {
                return Err(invalid(
                    "people_snapshot",
                    "a field this store does not know",
                ));
            }
        };
        if taken {
            return Err(invalid("people_snapshot", "a field twice"));
        }
    }
    Ok(s)
}

/// Store a snapshot (never updated afterwards).
///
/// # Errors
/// `Invalid` naming `table.column` (also: a field twice, a field or purpose the store does not know).
pub async fn insert(c: &Conn<'_>, s: &Snapshot) -> Result<()> {
    let e = &s.excerpt;
    let f = slots_of(&e.values)?;
    let mut row: Row = vec![
        id_value(s.id),
        text(&e.person.id.canonical()),
        text(art_word(e.person.art)),
    ];
    row.push(int(e.captured.millis()));
    row.push(text(&e.source.canonical()));
    row.extend([
        text(s.doc_kind.word()),
        int(date_to_int(&s.issued)),
        int(date_to_int(&s.keep_until)),
    ]);
    row.extend(purpose_cols(&e.purpose)?);
    row.push(e.consent.map_or_else(|| opt_text(None), id_value));
    for t in [&f.full_name, &f.given_name, &f.surname, &f.organization] {
        row.push(opt_text(t.as_deref()));
    }
    row.push(opt_text(f.postal.as_ref().map(|p| p.0.as_str())));
    row.push(opt_text(f.postal.as_ref().map(|p| p.1.as_str())));
    row.push(opt_text(f.email.as_deref()));
    row.push(opt_text(f.phone.as_deref()));
    row.extend(birthday::cols(f.birthday.as_ref()));
    row.extend(birthday::month_day_cols(f.birth_month_day));
    c.insert(&SNAPSHOT, row).await
}

/// The snapshot with this id.
///
/// # Errors
/// `Unavailable` for a row that cannot be read back.
pub async fn get(c: &Conn<'_>, id: ObjectId) -> Result<Option<Snapshot>> {
    let names: Vec<&str> = SNAPSHOT.cols.iter().map(|x| x.name).collect();
    let rows = c
        .query(
            &format!(
                "SELECT {} FROM people_snapshot WHERE id = ?",
                names.join(", ")
            ),
            vec![id_value(id)],
        )
        .await?;
    rows.first().map(of_row).transpose()
}

fn bad<E>(what: &str, _: E) -> fs_objects_v5::v5::Error {
    damaged(what)
}

fn of_row(row: &Row) -> Result<Snapshot> {
    let mut values = Vec::new();
    let mut put = |v: ExcerptValue| values.push(v);
    if let Some(n) = get_opt_text(row, 11)? {
        put(ExcerptValue::FullName(
            Name::new(&n, &[]).map_err(|e| bad("name", e))?,
        ));
    }
    for (at, make) in [
        (12, ExcerptValue::GivenName as fn(Text) -> ExcerptValue),
        (13, ExcerptValue::Surname),
        (14, ExcerptValue::Organization),
    ] {
        if let Some(t) = get_opt_text(row, at)? {
            put(make(Text::parse(&t).map_err(|e| bad("text", e))?));
        }
    }
    if let Some(full) = get_opt_text(row, 15)? {
        let country = get_opt_text(row, 16)?.unwrap_or_default();
        put(ExcerptValue::Postal(
            Address::new(&[], &country, &full).map_err(|e| bad("address", e))?,
        ));
    }
    if let Some(m) = get_opt_text(row, 17)? {
        put(ExcerptValue::Email(
            EmailAddress::parse(&m).map_err(|_| bad("email", ()))?,
        ));
    }
    if let Some(p) = get_opt_text(row, 18)? {
        put(ExcerptValue::Phone(
            PhoneNumber::parse(&p).map_err(|_| bad("phone", ()))?,
        ));
    }
    if let Some(b) = birthday::of(row, 19)? {
        put(ExcerptValue::Birthday(b));
    }
    if let Some(m) = birthday::month_day_of(row, 22)? {
        put(ExcerptValue::BirthMonthDay(m));
    }
    let consent = get_opt_text(row, 10)?
        .map(|t| parse_id(&t).ok_or_else(|| damaged("consent id")))
        .transpose()?;
    Ok(Snapshot {
        id: get_id(row, 0)?,
        doc_kind: DocKind::of(&get_text(row, 5)?)?,
        issued: int_to_date(get_int(row, 6)?)?,
        keep_until: int_to_date(get_int(row, 7)?)?,
        excerpt: Excerpt {
            person: PersonRef {
                id: reference_of(&get_text(row, 1)?)?,
                art: art_of(&get_text(row, 2)?)?,
                abzug: None,
            },
            source: reference_of(&get_text(row, 4)?)?,
            captured: Utc::from_millis(get_int(row, 3)?),
            purpose: purpose_of(row, 8)?,
            consent,
            values,
            missing: Vec::new(),
        },
    })
}
