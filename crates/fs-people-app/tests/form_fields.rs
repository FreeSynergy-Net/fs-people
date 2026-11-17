//! F1: the form edits every kind of entry the type description names for a person (nicknames,
//! e-mails, phones, organisations, titles, addresses, links, notes, the birthday), says the context of
//! an entry, and gives a new e-mail or phone a context by the group it is added to.
//!
//! Pattern: Integration tests over the real core and the real state machine; the oracle is a card
//! written as JSON by hand (JSContact, RFC 9553), not built by the code under test.
//! Mutants (each makes a test red): (1) `Kind::set` for addresses leaving `components` in place
//! (`address_edit_replaces_the_parts`); (2) `birthday::parse` accepting 30 February
//! (`form_birthday::a_birthday_is_a_real_day`); (3) `Kind::add` dropping the context
//! (`a_new_e_mail_in_the_work_group_gets_the_context`); (4) `birthday::set` replacing a timestamp
//! birth instead of adding beside it (`form_birthday::a_timestamp_birth_stays`); (5) `kind_of` accepting any suffix
//! (`an_unknown_group_is_refused_not_dropped`).

mod common;

use common::{World, added_key, error_of};
use fs_components_workbenches::people::PeopleState;
use fs_people_core::Card;
use serde_json::json;

fn full_card() -> Card {
    serde_json::from_value(json!({
        "@type": "Card", "version": "1.0", "uid": "urn:uuid:01a1aaef-ba80-7f7a-8dff-9f85769beae9",
        "name": {"full": "Amara Test"},
        "nicknames": {"n1": {"name": "Ami", "contexts": {"private": true}}},
        "emails": {"e1": {"address": "a@example.org", "contexts": {"work": true}, "pref": 1}},
        "phones": {"p1": {"number": "+49 30 1234567", "features": {"mobile": true}}},
        "organizations": {"o1": {"name": "Looms"}},
        "titles": {"t1": {"name": "Analyst", "organizationId": "o1"}},
        "addresses": {"a1": {
            "components": [{"kind": "locality", "value": "Berlin"}],
            "countryCode": "DE", "contexts": {"private": true}}},
        "links": {"l1": {"uri": "https://example.org/amara", "kind": "contact"}},
        "notes": {"c1": {"note": "met at the fair"}},
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
fn every_kind_is_in_the_form_with_its_context_in_the_label() {
    let w = World::in_memory();
    let (_, s) = edit(&w, full_card());
    let draft = &s.editing().unwrap().draft;
    let text = |key: &str| {
        let f = draft.fields().find(|f| f.key == key).unwrap();
        (f.value.clone(), f.label.clone())
    };
    assert_eq!(
        text("nicknames/n1"),
        ("Ami".into(), "Nickname (private)".into())
    );
    assert_eq!(
        text("emails/e1"),
        ("a@example.org".into(), "E-mail (work)".into())
    );
    assert_eq!(text("phones/p1").0, "+49 30 1234567");
    assert_eq!(text("organizations/o1").0, "Looms");
    assert_eq!(text("titles/t1").0, "Analyst");
    assert_eq!(
        text("addresses/a1"),
        ("Berlin".into(), "Address (private)".into())
    );
    assert_eq!(text("links/l1").0, "https://example.org/amara");
    assert_eq!(text("notes/c1").0, "met at the fair");
    assert_eq!(text("birthday/day").0, "1990-05-17");
}

#[test]
fn a_change_of_text_keeps_what_else_the_entry_holds() {
    let w = World::in_memory();
    let (key, mut s) = edit(&w, full_card());
    s.set_field("emails/e1", "b@example.org");
    s.set_field("nicknames/n1", "Amy");
    s.set_field("titles/t1", "Lead analyst");
    s.set_field("links/l1", "https://example.org/a");
    s.set_field("notes/c1", "met in May");
    s.save();
    saved(&s);
    let card = w.card(&key).to_value();
    assert_eq!(
        card["emails"]["e1"],
        json!({"address": "b@example.org", "contexts": {"work": true}, "pref": 1})
    );
    assert_eq!(
        card["nicknames"]["n1"]["contexts"],
        json!({"private": true})
    );
    assert_eq!(card["titles"]["t1"]["organizationId"], "o1");
    assert_eq!(card["links"]["l1"]["kind"], "contact");
    assert_eq!(card["links"]["l1"]["uri"], "https://example.org/a");
    assert_eq!(card["notes"]["c1"]["note"], "met in May");
    assert_eq!(card["phones"]["p1"]["features"], json!({"mobile": true}));
}

#[test]
fn address_edit_replaces_the_parts() {
    let w = World::in_memory();
    let (key, mut s) = edit(&w, full_card());
    s.set_field("addresses/a1", "Hauptstrasse 1, Hamburg");
    s.save();
    saved(&s);
    let card = w.card(&key).to_value();
    assert_eq!(card["addresses"]["a1"]["full"], "Hauptstrasse 1, Hamburg");
    assert!(
        card["addresses"]["a1"].get("components").is_none(),
        "the old parts would keep Berlin"
    );
    assert_eq!(card["addresses"]["a1"]["countryCode"], "DE");
    assert_eq!(
        card["addresses"]["a1"]["contexts"],
        json!({"private": true})
    );
}

#[test]
fn a_new_e_mail_in_the_work_group_gets_the_context() {
    let w = World::in_memory();
    let (key, mut s) = edit(&w, full_card());
    let field = added_key(&mut s, "emails.work");
    assert_eq!(
        s.editing()
            .unwrap()
            .draft
            .fields()
            .find(|f| f.key == field)
            .unwrap()
            .label,
        "E-mail (work)"
    );
    s.set_field(&field, "work@example.org");
    let phone = added_key(&mut s, "phones.private");
    s.set_field(&phone, "+49 170 1234567");
    let plain = added_key(&mut s, "titles");
    s.set_field(&plain, "Chair");
    s.save();
    saved(&s);
    let card = w.card(&key);
    let added = card
        .emails
        .values()
        .find(|e| e.address == "work@example.org")
        .unwrap();
    assert_eq!(added.contexts.get("work"), Some(&true));
    let phone = card
        .phones
        .values()
        .find(|p| p.number == "+49 170 1234567")
        .unwrap();
    assert_eq!(phone.contexts.get("private"), Some(&true));
    assert!(card.titles.values().any(|t| t.name == "Chair"));
    assert_eq!(card.emails.len(), 2, "the one that was there stays");
}

#[test]
fn emptied_fields_remove_their_entries_and_a_new_note_and_link_are_added() {
    let w = World::in_memory();
    let (key, mut s) = edit(&w, full_card());
    s.set_field("notes/c1", "");
    s.set_field("addresses/a1", "");
    let link = added_key(&mut s, "links");
    s.set_field(&link, "https://example.org/b");
    let note = added_key(&mut s, "notes");
    s.set_field(&note, "second");
    s.save();
    saved(&s);
    let card = w.card(&key);
    assert!(card.addresses.is_empty());
    assert_eq!(
        card.notes
            .values()
            .map(|n| n.note.as_str())
            .collect::<Vec<_>>(),
        ["second"]
    );
    assert_eq!(card.links.len(), 2);
}

#[test]
fn a_new_person_can_be_made_with_the_new_kinds() {
    let w = World::in_memory();
    let mut s = w.state();
    s.begin_create();
    s.set_field("name/full", "Dee");
    let title = added_key(&mut s, "titles");
    s.set_field(&title, "Editor");
    s.set_field("birthday/day", "--12-24");
    s.save();
    saved(&s);
    let key = s.selected().unwrap().to_string();
    let card = w.card(&key);
    assert_eq!(card.titles.values().next().unwrap().name, "Editor");
    assert_eq!(
        w.card(&key).to_value()["anniversaries"]["b1"]["date"]["day"],
        24
    );
}

#[test]
fn an_unknown_group_is_refused_not_dropped() {
    let w = World::in_memory();
    let (key, mut s) = edit(&w, full_card());
    let before = w.card(&key);
    let mut draft = s.editing().unwrap().draft.clone();
    let mut f = fs_components_workbenches::people::DraftField::new("emails.vacation/+9", "x", "");
    f.value = "v@example.org".into();
    f.added = true;
    draft.groups[0].fields.push(f);
    let mut card = before.clone();
    let e = fs_people_app::apply_draft(&mut card, &draft).unwrap_err();
    assert!(e.message.contains("unknown field"), "{e:?}");
    let _ = &mut s;
}
