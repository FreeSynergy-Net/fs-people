//! F3: the sensitive marking at card level is set in the form; it is a mark kept in this program (the
//! contract `person_directory` carries it with v2). Lifting is refused with a sentence (the core
//! never takes a marking away on an update; who may lift one is an open decision), and nothing of that
//! edit is written. An accident never reduces it.
//!
//! Pattern: Integration tests over the real core and store (the store's envelope is the oracle: it
//! must know what the card says). Mutants: (1) `apply_draft` not refusing `lifted`
//! (`lifting_a_category_is_refused_and_changes_nothing`, `renaming_a_category_is_a_lifting_and_refused_too`);
//! (2) `note` accepting spaces (`a_category_has_no_spaces`); (3) `Changes::apply` replacing the whole
//! list (`a_second_category_keeps_the_first`).

mod common;

use common::{World, added_key, error_of, person};

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
fn lifting_a_category_is_refused_and_changes_nothing() {
    let w = World::in_memory();
    let mut card = person("Amara", "a@example.org");
    card.mark_sensitive(["pd:Religion", "pd:HealthRecord"]);
    let key = w.add(card);
    let mut s = w.state();
    s.select(&key);
    s.begin_edit();
    s.set_field("name/full", "Augusta");
    s.set_field("sensitive/pd:Religion", "");
    s.save();
    assert!(s.editing().is_some());
    assert!(
        error_of(&s).contains("cannot be lifted yet"),
        "{}",
        error_of(&s)
    );
    let card = w.card(&key);
    assert_eq!(
        card.sensitive_categories(),
        ["pd:HealthRecord", "pd:Religion"]
    );
    assert_eq!(
        card.display_name(),
        "Amara",
        "nothing of the edit was written"
    );
    assert_eq!(stored_marks(&w, &key), ["pd:HealthRecord", "pd:Religion"]);
}

#[test]
fn renaming_a_category_is_a_lifting_and_refused_too() {
    let w = World::in_memory();
    let mut card = person("Amara", "a@example.org");
    card.mark_sensitive(["pd:Religion"]);
    let key = w.add(card);
    let mut s = w.state();
    s.select(&key);
    s.begin_edit();
    s.set_field("sensitive/pd:Religion", "pd:Belief");
    s.save();
    assert!(
        error_of(&s).contains("cannot be lifted yet"),
        "{}",
        error_of(&s)
    );
    assert_eq!(w.card(&key).sensitive_categories(), ["pd:Religion"]);
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
    assert_eq!(w.card(&key).sensitive_categories(), [] as [String; 0]);
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
