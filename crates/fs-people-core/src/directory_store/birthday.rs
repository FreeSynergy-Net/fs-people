//! The birthday as typed columns: a date with year (`DATE`) or a month and a day without a year (two `INT`), never a
//! text and never JSON (ADR 0086). The same three columns serve `people_birthday` and `people_snapshot`.
//!
//! Pattern: Adapter (`Birthday` <-> `(date, month, day)`); `MonthDay::new` stays the only check of a month-day.

use fs_objects_v5::v5::Result;
use fs_objects_v5::v5::codec::{Row, damaged, date_to_int, get_opt_int, int_to_date, opt_int};
use fs_people_directory::{Birthday, MonthDay};

/// The three columns `(date, month, day)` of an optional birthday.
pub fn cols(b: Option<&Birthday>) -> Row {
    match b {
        Some(Birthday::Date(d)) => {
            vec![opt_int(Some(date_to_int(d))), opt_int(None), opt_int(None)]
        }
        Some(Birthday::MonthDay(m)) => md_with(None, Some(*m)),
        None => vec![opt_int(None), opt_int(None), opt_int(None)],
    }
}

fn md_with(date: Option<i64>, md: Option<MonthDay>) -> Row {
    vec![
        opt_int(date),
        opt_int(md.map(|m| i64::from(m.month()))),
        opt_int(md.map(|m| i64::from(m.day()))),
    ]
}

/// The two columns `(month, day)` of an optional month and day (the field `BirthMonthDay`).
pub fn month_day_cols(m: Option<MonthDay>) -> Row {
    md_with(None, m)[1..].to_vec()
}

/// The month and day of two columns.
pub fn month_day_of(row: &Row, at: usize) -> Result<Option<MonthDay>> {
    match (get_opt_int(row, at)?, get_opt_int(row, at + 1)?) {
        (Some(m), Some(d)) => {
            let month = u8::try_from(m).map_err(|_| damaged("month"))?;
            let day = u8::try_from(d).map_err(|_| damaged("day"))?;
            MonthDay::new(month, day)
                .map(Some)
                .map_err(|_| damaged("month and day"))
        }
        (None, None) => Ok(None),
        _ => Err(damaged("half a month and day")),
    }
}

/// The birthday of three columns `(date, month, day)`.
pub fn of(row: &Row, at: usize) -> Result<Option<Birthday>> {
    if let Some(day) = get_opt_int(row, at)? {
        return Ok(Some(Birthday::Date(int_to_date(day)?)));
    }
    Ok(month_day_of(row, at + 1)?.map(Birthday::MonthDay))
}
