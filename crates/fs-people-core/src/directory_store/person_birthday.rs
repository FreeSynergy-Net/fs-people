//! Reading and writing `people_birthday`: the birthday of a person as a date with year or a month and a day, in
//! typed columns (one row per person).
//!
//! Pattern: Repository functions over a unit of work (`Conn`); the columns come from `birthday`.

use fs_objects_v5::v5::codec::{Row, art_word, int, text};
use fs_objects_v5::v5::{Conn, Result};
use fs_people_directory::{Birthday, PersonRef, Utc};
use fs_types::BasicValue;

use super::birthday;
use super::tables::BIRTHDAY;

/// Set (or replace) the birthday of a person; `at` is when it was set.
///
/// # Errors
/// `Invalid` naming `table.column`.
pub async fn set_birthday(
    c: &Conn<'_>,
    person: &PersonRef,
    value: &Birthday,
    at: Utc,
) -> Result<()> {
    let mut row: Row = vec![text(&person.id.canonical()), text(art_word(person.art))];
    row.extend(birthday::cols(Some(value)));
    row.push(int(at.millis()));
    c.upsert(&BIRTHDAY, &["person_id"], row).await
}

/// The birthday of a person, if one was set.
///
/// # Errors
/// `Unavailable` for a row that cannot be read back.
pub async fn birthday_of(c: &Conn<'_>, person: &PersonRef) -> Result<Option<Birthday>> {
    let rows = c
        .query(
            "SELECT person_id, person_art, date, month, day FROM people_birthday WHERE person_id = ?",
            vec![text(&person.id.canonical())],
        )
        .await?;
    let Some(row) = rows.first() else {
        return Ok(None);
    };
    birthday::of(row, 2)
}
