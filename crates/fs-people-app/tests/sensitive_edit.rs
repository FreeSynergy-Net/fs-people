//! F3: the sensitive marking at card level is set and lifted in the form; it is a mark kept in this
//! program (the contract `person_directory` carries it with v2). An accident never lifts it; a mark
//! on a part of the card is not the form's to lift.
//!
//! Pattern: Integration tests over the real core and store (the store's envelope is the oracle: it
//! must know what the card says). Mutants: (1) `restore_except` ignoring `lifted` (the category comes
//! back: `a_category_can_be_lifted`); (2) `still_marked` always `None` (a nested mark is lifted behind
//! the person's back: `a_category_marked_on_a_part_is_not_lifted`); (3) `note` accepting spaces
//! (`a_category_has_no_spaces`); (4) `Changes::apply` replacing the whole list
//! (`a_second_category_keeps_the_first`).

mod common;

use common::{World, added_key, error_of, person};
use fs_people_core::Card;
use serde_json::json;

const KEY: &str = "freesynergy.net:sensitive";

fn stored_marks(w: &World, key: &str) -> Vec<String> {
    w.runtime
        .block_on(w.repo.get(key.parse().unwrap()))
        .unwrap()
        .unwrap()
        .envelope
        .sensitive
}

#[test]
fn a_category_can_be_set_and_the_store_knows() {
    let w = World::in_memory();
    let key = w.add(person("Amara", "a@example.org"));
    let mut s = w.state();
    s.select(&key);
    s.begin_edit();
    let field = added_key(&mut s, "sensitive");
    assert!(
        s.editing()
            .unwrap()
            .draft
            .groups
            .iter()
            .any(|g| g.key == "sensitive" && g.label.contains("contract v2"))
    );
    s.set_field(&field, "pd:HealthRecord");
    s.save();
    assert!(s.editing().is_none(), "{}", error_of(&s));
    assert_eq!(w.card(&key).sensitive_categories(), ["pd:HealthRecord"]);
    assert_eq!(stored_marks(&w, &key), ["pd:HealthRecord"]);
    // The person is now marked in the list and shown so.
    s.begin_edit();
    assert!(
        s.editing()
            .unwrap()
            .draft
            .marks
            .iter()
            .any(|m| m.contains("pd:HealthRecord"))
    );
}

#[test]
fn a_second_category_keeps_the_first() {
    let w = World::in_memory();
    let mut card = person("Amara", "a@example.org");
    card.mark_sensitive(["pd:Religion"]);
    let key = w.add(card);
    let mut s = w.state();
    s.select(&key);
    s.begin_edit();
    let field = added_key(&mut s, "sensitive");
    s.set_field(&field, "pd:HealthRecord");
    s.save();
    assert!(s.editing().is_none(), "{}", error_of(&s));
    assert_eq!(
        w.card(&key).sensitive_categories(),
        ["pd:HealthRecord", "pd:Religion"]
    );
}

#[test]
fn a_category_can_be_lifted() {
    let w = World::in_memory();
    let mut card = person("Amara", "a@example.org");
    card.mark_sensitive(["pd:Religion", "pd:HealthRecord"]);
    let key = w.add(card);
    let mut s = w.state();
    s.select(&key);
    s.begin_edit();
    s.set_field("sensitive/pd:Religion", "");
    s.save();
    assert!(s.editing().is_none(), "{}", error_of(&s));
    assert_eq!(w.card(&key).sensitive_categories(), ["pd:HealthRecord"]);
    assert_eq!(stored_marks(&w, &key), ["pd:HealthRecord"]);
    s.begin_edit();
    s.set_field("sensitive/pd:HealthRecord", "");
    s.save();
    assert!(w.card(&key).sensitive_categories().is_empty());
    assert!(
        !w.card(&key).extra.contains_key(KEY),
        "no empty list is left behind"
    );
    assert!(stored_marks(&w, &key).is_empty());
}

#[test]
fn a_category_can_be_renamed() {
    let w = World::in_memory();
    let mut card = person("Amara", "a@example.org");
    card.mark_sensitive(["pd:Religion"]);
    let key = w.add(card);
    let mut s = w.state();
    s.select(&key);
    s.begin_edit();
    s.set_field("sensitive/pd:Religion", "pd:Belief");
    s.save();
    assert_eq!(w.card(&key).sensitive_categories(), ["pd:Belief"]);
}

#[test]
fn a_category_marked_on_a_part_is_not_lifted() {
    let w = World::in_memory();
    let card: Card = serde_json::from_value(json!({
        "@type": "Card", "version": "1.0", "uid": "urn:uuid:01a1aaef-ba80-7f7a-8dff-9f85769beaec",
        "name": {"full": "Amara"}, KEY: ["pd:Religion"],
        "emails": {"e1": {"address": "a@example.org", KEY: ["pd:Religion"]}}
    }))
    .unwrap();
    let key = w.add(card);
    let mut s = w.state();
    s.select(&key);
    s.begin_edit();
    s.set_field("sensitive/pd:Religion", "");
    s.save();
    assert!(s.editing().is_some());
    assert!(
        error_of(&s).contains("The category pd:Religion is also marked on a part of the card"),
        "{}",
        error_of(&s)
    );
    assert_eq!(w.card(&key).sensitive_categories(), ["pd:Religion"]);
    assert!(w.card(&key).extra.contains_key(KEY), "nothing was changed");
}

#[test]
fn a_category_has_no_spaces() {
    let w = World::in_memory();
    let key = w.add(person("Amara", "a@example.org"));
    let mut s = w.state();
    s.select(&key);
    s.begin_edit();
    let field = added_key(&mut s, "sensitive");
    s.set_field(&field, "health data");
    s.save();
    assert!(
        error_of(&s).contains("A category has no spaces"),
        "{}",
        error_of(&s)
    );
    assert!(w.card(&key).sensitive_categories().is_empty());
}

#[test]
fn an_edit_of_something_else_still_never_reduces_the_marking() {
    let w = World::in_memory();
    let mut card = person("Amara", "a@example.org");
    card.mark_sensitive(["pd:Religion"]);
    let key = w.add(card);
    let mut s = w.state();
    s.select(&key);
    s.begin_edit();
    s.set_field("name/full", "Augusta");
    s.save();
    assert_eq!(w.card(&key).sensitive_categories(), ["pd:Religion"]);
}

#[test]
fn a_new_person_can_be_marked_from_the_start() {
    let w = World::in_memory();
    let mut s = w.state();
    s.begin_create();
    s.set_field("name/full", "Dee");
    let field = added_key(&mut s, "sensitive");
    s.set_field(&field, "pd:HealthRecord");
    s.save();
    assert!(s.editing().is_none(), "{}", error_of(&s));
    let key = s.selected().unwrap().to_string();
    assert_eq!(stored_marks(&w, &key), ["pd:HealthRecord"]);
}
