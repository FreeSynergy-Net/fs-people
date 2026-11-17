//! The birthday of a card as one text field, with or without the year.
//!
//! Pattern: Adapter — the JSContact anniversary of kind `birth` (a `PartialDate`) written as
//! `1990-05-17` or `--05-17` (ISO 8601 without the year). Only a date with a month and a day is
//! shown; a birth that is a timestamp, or has only a year, stays on the card as it is and a new
//! birthday is added next to it. A birth with a place keeps the place when the date is emptied.

use fs_people_core::Card;
use fs_people_core::card::{Anniversary, AnniversaryDate};

/// The key of the field.
pub(super) const KEY: &str = "birthday/day";
/// The key of its group.
pub(super) const GROUP: &str = "birthday";

fn shown(a: &Anniversary) -> Option<String> {
    if a.kind.as_deref() != Some("birth") {
        return None;
    }
    let d = a.date.as_ref()?;
    let scale_ok = d.calendar_scale.as_deref().is_none_or(|s| s == "gregorian");
    match (d.year, d.month, d.day) {
        (Some(y), Some(m), Some(day)) if scale_ok => Some(format!("{y:04}-{m:02}-{day:02}")),
        (None, Some(m), Some(day)) if scale_ok => Some(format!("--{m:02}-{day:02}")),
        _ => None,
    }
}

/// The birthday as written, or an empty text.
pub(super) fn written(card: &Card) -> String {
    card.anniversaries
        .values()
        .find_map(shown)
        .unwrap_or_default()
}

/// `(year, month, day)` of a text like `1990-05-17` or `--05-17`; `None` if it is not a real day.
pub(super) fn parse(text: &str) -> Option<(Option<u32>, u32, u32)> {
    let (year, rest) = match text.strip_prefix("--") {
        Some(rest) => (None, rest),
        None => {
            let (y, rest) = text.split_once('-')?;
            if y.len() != 4 {
                return None;
            }
            (Some(y.parse::<u32>().ok().filter(|y| *y >= 1)?), rest)
        }
    };
    let (m, d) = rest.split_once('-')?;
    if m.len() != 2 || d.len() != 2 {
        return None;
    }
    let (month, day) = (m.parse::<u32>().ok()?, d.parse::<u32>().ok()?);
    let leap = year.is_none_or(|y| (y % 4 == 0 && y % 100 != 0) || y % 400 == 0);
    let longest = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return None,
    };
    (1..=longest).contains(&day).then_some((year, month, day))
}

/// Put `text` (already checked with [`parse`]) on the card as its birthday; an empty text removes it.
/// `false`: nothing was done, the birth holds more than the date.
pub(super) fn set(card: &mut Card, text: &str) -> bool {
    let found = card
        .anniversaries
        .iter()
        .find(|(_, a)| shown(a).is_some())
        .map(|(id, _)| id.clone());
    if text.is_empty() {
        let Some(id) = found else { return true };
        let keeps_more = card
            .anniversaries
            .get(&id)
            .is_some_and(|a| a.place.is_some() || !a.extra.is_empty());
        if keeps_more {
            return false;
        }
        card.anniversaries.remove(&id);
        return true;
    }
    let Some((year, month, day)) = parse(text) else {
        return true;
    };
    let date = AnniversaryDate {
        type_: Some("PartialDate".into()),
        year,
        month: Some(month),
        day: Some(day),
        ..AnniversaryDate::default()
    };
    match found.and_then(|id| card.anniversaries.get_mut(&id)) {
        Some(a) => {
            // Whatever else the date said (a calendar scale) described the old one.
            a.date = Some(date);
        }
        None => {
            let id = (1..=u32::MAX)
                .map(|i| format!("b{i}"))
                .find(|k| !card.anniversaries.contains_key(k))
                .unwrap_or_default();
            let item = Anniversary {
                type_: Some("Anniversary".into()),
                kind: Some("birth".into()),
                date: Some(date),
                ..Anniversary::default()
            };
            card.anniversaries.insert(id, item);
        }
    }
    true
}
