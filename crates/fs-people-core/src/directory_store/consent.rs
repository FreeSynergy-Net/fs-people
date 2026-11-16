//! Reading and writing `people_consent`: one row per `Consent` of the locked trait, typed columns, no JSON.
//!
//! Pattern: Repository functions over a unit of work (`Conn`): the caller owns the transaction. The record is
//! written once; the only change is the withdrawal (compare-and-set on `rev`), which never deletes the row.

use fs_objects_v5::v5::codec::{
    Row, date_to_int, get_id, get_int, get_opt_int, get_opt_text, get_text, id_value, int,
    int_to_date, invalid, opt_text, person_of, person_values, text,
};
use fs_objects_v5::v5::{Conn, Error, Result};
use fs_people_directory::{
    Consent, ConsentEntry, Date, ObjectId, PersonRef, Revision, Utc, Withdrawal, Withdrawn,
};

use super::tables::CONSENT;
use super::words::{
    opt_person_cols, opt_person_of, opt_ref_col, opt_ref_of, origin_of, origin_word, purpose_cols,
    purpose_of, when_cols, when_of,
};

fn select() -> String {
    let names: Vec<&str> = CONSENT.cols.iter().map(|c| c.name).collect();
    format!("SELECT {} FROM people_consent", names.join(", "))
}

fn rev_int(r: Revision) -> i64 {
    i64::try_from(r.0).unwrap_or(i64::MAX)
}

/// Store a new consent (its `withdrawn` must be empty: a record starts active).
///
/// # Errors
/// `Invalid` naming `table.column` for a value that breaks a column rule or a purpose or origin the store does not
/// know; `Invalid` when the consent is already withdrawn.
pub async fn record(c: &Conn<'_>, consent: &Consent) -> Result<()> {
    if consent.withdrawn.is_some() || consent.retain_until.is_some() {
        return Err(invalid(
            "people_consent.withdrawn",
            "a new record is active",
        ));
    }
    let e = &consent.entry;
    let mut row: Row = vec![id_value(consent.id), int(rev_int(consent.rev))];
    row.extend(person_values(&e.person));
    row.extend(opt_person_cols(e.given_by.as_ref()));
    row.extend(purpose_cols(&e.purpose)?);
    row.extend(opt_ref_col(e.scope.as_ref()));
    row.extend(opt_person_cols(e.recipient.as_ref()));
    row.extend(when_cols(&e.given));
    row.push(text(origin_word(e.origin)?));
    row.extend(opt_ref_col(e.evidence.as_ref()));
    row.extend(opt_ref_col(e.text.as_ref()));
    row.extend(person_values(&consent.recorded_by));
    row.push(int(consent.recorded_at.millis()));
    row.extend([
        opt_text(None),
        opt_text(None),
        opt_text(None),
        opt_text(None),
    ]);
    row.extend([
        opt_text(None),
        opt_text(None),
        opt_text(None),
        opt_text(None),
        opt_text(None),
    ]);
    c.insert(&CONSENT, row).await
}

/// The consent with this id.
///
/// # Errors
/// `Unavailable` for a row that cannot be read back.
pub async fn get(c: &Conn<'_>, id: ObjectId) -> Result<Option<Consent>> {
    let rows = c
        .query(&format!("{} WHERE id = ?", select()), vec![id_value(id)])
        .await?;
    rows.first().map(of_row).transpose()
}

/// The consents of a person (the reference's id and kind), oldest record first.
///
/// # Errors
/// `Unavailable` for a row that cannot be read back.
pub async fn of_person(c: &Conn<'_>, person: &PersonRef) -> Result<Vec<Consent>> {
    let p = person_values(person);
    let rows = c
        .query(
            &format!(
                "{} WHERE person_id = ? AND person_art = ? ORDER BY recorded_at, id",
                select()
            ),
            vec![p[0].clone(), p[1].clone()],
        )
        .await?;
    rows.iter().map(of_row).collect()
}

/// Record the withdrawal of a consent, compare-and-set on the revision the caller read; `retain_until` is the day
/// the proof may be deleted (`ConsentRetention::after`). The row stays.
///
/// # Errors
/// `NotFound`; `Changed` when the revision moved on; `Invalid` when it is withdrawn already or the origin is unknown.
pub async fn withdraw(
    c: &Conn<'_>,
    id: ObjectId,
    expected: Revision,
    w: &Withdrawn,
    retain_until: Date,
) -> Result<()> {
    let mut params: Row = when_cols(&w.withdrawal.at);
    params.push(text(origin_word(w.withdrawal.origin)?));
    params.extend(opt_ref_col(w.withdrawal.evidence.as_ref()));
    params.extend(person_values(&w.recorded_by));
    params.push(int(w.recorded_at.millis()));
    params.push(int(date_to_int(&retain_until)));
    params.extend([id_value(id), int(rev_int(expected))]);
    let changed = c
        .exec(
            "UPDATE people_consent SET withdrawn_at = ?, withdrawn_on = ?, withdrawn_origin = ?, \
             withdrawn_evidence = ?, withdrawn_by_id = ?, withdrawn_by_art = ?, withdrawn_by_abzug = ?, \
             withdrawn_recorded_at = ?, retain_until = ?, rev = rev + 1 \
             WHERE id = ? AND rev = ? AND withdrawn_origin IS NULL",
            params,
        )
        .await?;
    if changed == 1 {
        return Ok(());
    }
    match get(c, id).await? {
        None => Err(Error::NotFound(id)),
        Some(now) if now.withdrawn.is_some() => Err(invalid(
            "people_consent.withdrawn_origin",
            "already withdrawn",
        )),
        Some(now) => Err(Error::Changed { current: now.rev }),
    }
}

fn of_row(row: &Row) -> Result<Consent> {
    let entry = ConsentEntry {
        person: person_of(row, 2)?,
        given_by: opt_person_of(row, 5)?,
        purpose: purpose_of(row, 8)?,
        scope: opt_ref_of(row, 10)?,
        recipient: opt_person_of(row, 11)?,
        given: when_of(row, 14)?,
        origin: origin_of(&get_text(row, 16)?)?,
        evidence: opt_ref_of(row, 17)?,
        text: opt_ref_of(row, 18)?,
    };
    let withdrawn = match get_opt_text(row, 25)? {
        None => None,
        Some(origin) => Some(Withdrawn {
            withdrawal: Withdrawal {
                at: when_of(row, 23)?,
                origin: origin_of(&origin)?,
                evidence: opt_ref_of(row, 26)?,
            },
            recorded_by: person_of(row, 27)?,
            recorded_at: Utc::from_millis(get_int(row, 30)?),
        }),
    };
    let retain_until = get_opt_int(row, 31)?.map(int_to_date).transpose()?;
    Ok(Consent {
        id: get_id(row, 0)?,
        rev: Revision(u64::try_from(get_int(row, 1)?).unwrap_or(0)),
        entry,
        recorded_by: person_of(row, 19)?,
        recorded_at: Utc::from_millis(get_int(row, 22)?),
        withdrawn,
        retain_until,
    })
}
