//! F1, the birthday: one text field, `1990-05-17` or `--05-17` without the year; only a real day is
//! taken; a timestamp or place of a birth on the card is not lost.
//!
//! Pattern: Integration tests over the real core and the real state machine; the oracle is a card
//! written as JSON by hand (JSContact, RFC 9553). Mutants: (1) `birthday::parse` accepting 30
//! February (`a_birthday_is_a_real_day`); (2) `birthday::set` replacing a timestamp birth instead of
//! adding beside it (`a_timestamp_birth_stays`); (3) `set` removing a birth that has a place (the place is lost)
//! (`a_birth_with_a_place_keeps_its_date_and_says_why`).

mod common;

use common::{World, error_of};
use fs_components_workbenches::people::PeopleState;
use fs_people_core::Card;
use serde_json::json;

fn birth_card() -> Card {
    serde_json::from_value(json!({
        "@type": "Card", "version": "1.0", "uid": "urn:uuid:01a1aaef-ba80-7f7a-8dff-9f85769beae9",
        "name": {"full": "Amara Test"},
        "anniversaries": {"b1": {"kind": "birth", "date": {"@type": "PartialDate", "year": 1990, "month": 5, "day": 17}}}
    }))
    .unwrap()
}

fn edit(w: &World, card: Card) -> (String, PeopleState) {
    let key = w.add(card);
    let mut s = w.state();
    s.select(&key);
    s.begin_edit();
    (key, s)
}

fn saved(s: &PeopleState) {
    assert!(s.editing().is_none(), "{}", error_of(s));
}

#[test]
fn a_birthday_is_set_changed_written_without_a_year_and_removed() {
    let w = World::in_memory();
    let (key, mut s) = edit(&w, birth_card());
    s.set_field("birthday/day", "--02-29");
    s.save();
    saved(&s);
    let b = &w.card(&key).to_value()["anniversaries"]["b1"];
    assert_eq!(b["date"]["month"], 2);
    assert_eq!(b["date"]["day"], 29);
    assert!(b["date"].get("year").is_none());
    s.begin_edit();
    assert_eq!(
        s.editing()
            .unwrap()
            .draft
            .fields()
            .find(|f| f.key == "birthday/day")
            .unwrap()
            .value,
        "--02-29"
    );
    s.set_field("birthday/day", "");
    s.save();
    saved(&s);
    assert!(w.card(&key).anniversaries.is_empty());
}

#[test]
fn a_birthday_is_a_real_day() {
    for bad in [
        "1990-02-30",
        "1990-13-01",
        "1990-5-17",
        "17.05.1990",
        "--04-31",
        "2023-02-29",
    ] {
        let w = World::in_memory();
        let (key, mut s) = edit(&w, birth_card());
        s.set_field("birthday/day", bad);
        s.save();
        assert!(s.editing().is_some(), "{bad} was taken");
        assert!(
            error_of(&s).contains("Birthday: write 1990-05-17"),
            "{bad}: {}",
            error_of(&s)
        );
        assert_eq!(
            w.card(&key).to_value()["anniversaries"]["b1"]["date"]["day"],
            17
        );
    }
}

#[test]
fn a_timestamp_birth_stays() {
    let w = World::in_memory();
    let card: Card = serde_json::from_value(json!({
        "@type": "Card", "version": "1.0", "uid": "urn:uuid:01a1aaef-ba80-7f7a-8dff-9f85769beaea",
        "name": {"full": "Bo"},
        "anniversaries": {"b1": {"kind": "birth", "date": {"@type": "Timestamp", "utc": "1990-05-17T10:00:00Z"}}}
    })).unwrap();
    let (key, mut s) = edit(&w, card);
    assert_eq!(
        s.editing()
            .unwrap()
            .draft
            .fields()
            .find(|f| f.key == "birthday/day")
            .unwrap()
            .value,
        ""
    );
    s.set_field("birthday/day", "1991-01-02");
    s.save();
    saved(&s);
    let all = w.card(&key).to_value()["anniversaries"].clone();
    assert_eq!(all["b1"]["date"]["utc"], "1990-05-17T10:00:00Z");
    assert_eq!(all.as_object().unwrap().len(), 2);
}

#[test]
fn a_birth_with_a_place_keeps_its_date_and_says_why() {
    let w = World::in_memory();
    let card: Card = serde_json::from_value(json!({
        "@type": "Card", "version": "1.0", "uid": "urn:uuid:01a1aaef-ba80-7f7a-8dff-9f85769beaeb",
        "name": {"full": "Cy"},
        "anniversaries": {"b1": {"kind": "birth", "place": {"full": "Lyon"}, "date": {"@type": "PartialDate", "year": 1980, "month": 1, "day": 2}}}
    })).unwrap();
    let (key, mut s) = edit(&w, card);
    s.set_field("birthday/day", "");
    s.save();
    assert!(s.editing().is_some());
    assert!(
        error_of(&s).contains("cannot hold a birth without a date"),
        "{}",
        error_of(&s)
    );
    let b = &w.card(&key).to_value()["anniversaries"]["b1"];
    assert_eq!(b["place"]["full"], "Lyon");
    assert_eq!(b["date"]["year"], 1980);
}
