//! The stored words and column groups of the trait's value types (`Purpose`, `ConsentOrigin`, `When`, `PersonRef`).
//!
//! Pattern: Adapter (value object <-> typed columns; the only place that knows the stored words).
//!
//! `Purpose` and `ConsentOrigin` are `non_exhaustive` in the locked trait: a member this layer does not know is a
//! refusal (`Invalid`), never a silent fallback.

use fs_objects_v5::v5::Result;
use fs_objects_v5::v5::codec::{
    Row, date_to_int, get_opt_int, get_opt_text, get_text, int_to_date, invalid, opt_int, opt_text,
    person_of, person_values, reference_of, text,
};
use fs_people_directory::{ConsentOrigin, PersonRef, Purpose, Reference, Utc, When};
use fs_types::BasicValue;

const ORIGINS: [(ConsentOrigin, &str); 5] = [
    (ConsentOrigin::ByPerson, "by_person"),
    (ConsentOrigin::Form, "form"),
    (ConsentOrigin::Import, "import"),
    (ConsentOrigin::Oral, "oral"),
    (ConsentOrigin::Erasure, "erasure"),
];
const PURPOSES: [(&str, Purpose); 8] = [
    ("contract", Purpose::Contract),
    ("legal_obligation", Purpose::LegalObligation),
    ("own_use", Purpose::OwnUse),
    ("photo", Purpose::Photo),
    ("publication", Purpose::Publication),
    ("newsletter", Purpose::Newsletter),
    ("ai_assistance", Purpose::AiAssistance),
    ("disclosure_to_third_party", Purpose::DisclosureToThirdParty),
];

/// The two columns of a purpose: its word and, for `Other`, its IRI.
pub fn purpose_cols(p: &Purpose) -> Result<Row> {
    if let Purpose::Other(iri) = p {
        return Ok(vec![text("other"), text(&iri.canonical())]);
    }
    let word = PURPOSES
        .iter()
        .find(|(_, known)| known == p)
        .map(|(w, _)| *w)
        .ok_or_else(|| invalid("purpose", "a purpose this store does not know"))?;
    Ok(vec![text(word), opt_text(None)])
}

/// The purpose of its two columns.
pub fn purpose_of(row: &Row, at: usize) -> Result<Purpose> {
    let word = get_text(row, at)?;
    if word == "other" {
        let iri = get_opt_text(row, at + 1)?.ok_or_else(|| invalid("purpose_other", "missing"))?;
        return Ok(Purpose::Other(reference_of(&iri)?));
    }
    PURPOSES
        .iter()
        .find(|(w, _)| *w == word)
        .map(|(_, known)| known.clone())
        .ok_or_else(|| invalid("purpose", "unknown word"))
}

/// The stored word of an origin.
pub fn origin_word(o: ConsentOrigin) -> Result<&'static str> {
    ORIGINS
        .iter()
        .find(|(x, _)| *x == o)
        .map(|(_, w)| *w)
        .ok_or_else(|| invalid("origin", "an origin this store does not know"))
}

/// The origin of a stored word.
pub fn origin_of(word: &str) -> Result<ConsentOrigin> {
    ORIGINS
        .iter()
        .find(|(_, w)| *w == word)
        .map(|(x, _)| *x)
        .ok_or_else(|| invalid("origin", "unknown word"))
}

/// The two columns of a `When`: the instant (`UTC`) or the day (`DATE`), exactly one is set.
pub fn when_cols(w: &When) -> Row {
    match w {
        When::At(at) => vec![opt_int(Some(at.millis())), opt_int(None)],
        When::On(day) => vec![opt_int(None), opt_int(Some(date_to_int(day)))],
    }
}

/// The `When` of its two columns.
pub fn when_of(row: &Row, at: usize) -> Result<When> {
    match (get_opt_int(row, at)?, get_opt_int(row, at + 1)?) {
        (Some(ms), None) => Ok(When::At(Utc::from_millis(ms))),
        (None, Some(day)) => Ok(When::On(int_to_date(day)?)),
        _ => Err(invalid("when", "exactly one of instant and day")),
    }
}

/// The three columns of an optional person reference (empty: all null).
pub fn opt_person_cols(p: Option<&PersonRef>) -> Row {
    p.map_or_else(
        || vec![opt_text(None), opt_text(None), opt_text(None)],
        |p| person_values(p).to_vec(),
    )
}

/// The optional person reference of its three columns.
pub fn opt_person_of(row: &Row, at: usize) -> Result<Option<PersonRef>> {
    if get_opt_text(row, at)?.is_none() {
        return Ok(None);
    }
    person_of(row, at).map(Some)
}

/// An optional reference as one text column.
pub fn opt_ref_col(r: Option<&Reference>) -> Row {
    let s = r.map(BasicValue::canonical);
    vec![opt_text(s.as_deref())]
}

/// The optional reference of a column.
pub fn opt_ref_of(row: &Row, at: usize) -> Result<Option<Reference>> {
    get_opt_text(row, at)?.map(|s| reference_of(&s)).transpose()
}
