//! The retention of the proof of a withdrawn consent: default, limits and the day it ends.
//!
//! Pattern: Contract test (the one calculation of `ConsentRetention::after`).

use fs_people_directory::{ConsentRetention, Date, Utc, When};
use fs_types::BasicValue;

fn day(s: &str) -> Date {
    Date::parse(s).unwrap()
}

#[test]
fn three_years_from_the_end_of_the_year_by_default() {
    let r = ConsentRetention::default();
    assert_eq!(r.years(), 3);
    assert_eq!(
        r.after(&When::On(day("2026-01-01"))),
        Some(day("2029-12-31"))
    );
    assert_eq!(
        r.after(&When::On(day("2026-12-31"))),
        Some(day("2029-12-31"))
    );
    let instant = Utc::parse("2026-11-16T10:00:00Z").unwrap();
    assert_eq!(r.after(&When::At(instant)), Some(day("2029-12-31")));
}

#[test]
fn the_period_is_one_to_ten_years() {
    assert_eq!(ConsentRetention::new(0), None);
    assert_eq!(ConsentRetention::new(11), None);
    assert_eq!(
        ConsentRetention::new(1).map(ConsentRetention::years),
        Some(1)
    );
    let ten = ConsentRetention::new(10).unwrap();
    assert_eq!(
        ten.after(&When::On(day("2026-05-05"))),
        Some(day("2036-12-31"))
    );
}
