//! F2: roles are given and taken back in the form (`manager`, `owner`: ADR 0095, held within a group),
//! checked before anything is written, carried out after the card is saved; the AI role is not the
//! form's.
//!
//! Pattern: Integration tests over the real core (its role book is the oracle: `roles_of`) and the
//! real state machine. Mutants: (1) `role_edit::check` skipping the reach test
//! (`a_group_role_needs_its_group`); (2) `plan` letting `ai` through (`the_ai_role_is_not_given_here`);
//! (3) `Replace` ending the old assignment before giving the new one
//! (`a_role_that_cannot_be_given_leaves_the_old_one`); (4) `group_of` listing the AI marking
//! (`the_form_lists_the_roles_but_not_the_ai_marking`).

mod common;

use std::sync::Arc;

use common::{World, added_key, error_of, person};
use fs_components_workbenches::people::PeopleState;
use fs_people_app::{NoRoles, PeopleAdapter};

const GROUP: &str = "urn:group:1";

fn roles_of(w: &World, key: &str) -> Vec<(String, Option<String>)> {
    let mut held: Vec<_> = w
        .runtime
        .block_on(fs_people_core::RoleBook::roles_of(
            &*w.repo,
            key.parse().unwrap(),
        ))
        .unwrap()
        .into_iter()
        .map(|a| (a.role.rsplit('/').next().unwrap().to_owned(), a.scope))
        .collect();
    held.sort();
    held
}

fn start(w: &World, key: &str) -> PeopleState {
    let mut s = w.state();
    s.select(key);
    s.begin_edit();
    s
}

fn give(s: &mut PeopleState, text: &str) {
    let field = added_key(s, "roles");
    s.set_field(&field, text);
}

fn role_field(s: &PeopleState, text: &str) -> String {
    s.editing()
        .unwrap()
        .draft
        .fields()
        .find(|f| f.key.starts_with("roles/") && f.value == text)
        .unwrap_or_else(|| panic!("no role field {text}"))
        .key
        .clone()
}

#[test]
fn a_role_within_a_group_is_given_and_shown_and_taken_back() {
    let w = World::in_memory();
    let key = w.add(person("Amara", "a@example.org"));
    let mut s = start(&w, &key);
    give(&mut s, &format!("manager @ {GROUP}"));
    give(&mut s, &format!("owner @ {GROUP}"));
    s.save();
    assert!(s.editing().is_none(), "{}", error_of(&s));
    assert_eq!(
        roles_of(&w, &key),
        [
            ("manager".into(), Some(GROUP.into())),
            ("owner".into(), Some(GROUP.into()))
        ]
    );
    s.begin_edit();
    let owner = role_field(&s, &format!("owner @ {GROUP}"));
    s.set_field(&owner, "");
    s.save();
    assert!(s.editing().is_none(), "{}", error_of(&s));
    assert_eq!(roles_of(&w, &key), [("manager".into(), Some(GROUP.into()))]);
}

#[test]
fn a_role_can_be_replaced() {
    let w = World::in_memory();
    let key = w.add(person("Amara", "a@example.org"));
    let mut s = start(&w, &key);
    give(&mut s, &format!("member @ {GROUP}"));
    s.save();
    s.begin_edit();
    let old = role_field(&s, &format!("member @ {GROUP}"));
    s.set_field(&old, &format!("lead @ {GROUP}"));
    s.save();
    assert!(s.editing().is_none(), "{}", error_of(&s));
    assert_eq!(roles_of(&w, &key), [("lead".into(), Some(GROUP.into()))]);
}

#[test]
fn a_group_role_needs_its_group() {
    let w = World::in_memory();
    let key = w.add(person("Amara", "a@example.org"));
    let mut s = start(&w, &key);
    s.set_field("name/full", "Amara Two");
    give(&mut s, "manager");
    s.save();
    assert!(s.editing().is_some());
    assert!(
        error_of(&s).contains("held within a group: write \"manager @ group address\""),
        "{}",
        error_of(&s)
    );
    assert_eq!(roles_of(&w, &key), []);
    assert_eq!(
        w.card(&key).display_name(),
        "Amara",
        "the card was not written either"
    );
}

#[test]
fn an_unknown_role_and_an_empty_group_and_the_ai_role_are_refused_before_a_write() {
    for (text, said) in [
        ("wizard @ urn:group:1", "The role wizard is not defined."),
        ("manager @", "Write the role, then \"@\""),
        ("ai", "The AI role is not set here"),
        (
            "https://freesynergy.net/ns/roles/ai",
            "The AI role is not set here",
        ),
    ] {
        let w = World::in_memory();
        let key = w.add(person("Amara", "a@example.org"));
        let mut s = start(&w, &key);
        s.set_field("name/full", "Changed");
        give(&mut s, text);
        s.save();
        assert!(error_of(&s).contains(said), "{text}: {}", error_of(&s));
        assert_eq!(w.card(&key).display_name(), "Amara", "{text}");
    }
}

#[test]
fn the_ai_role_is_not_given_here() {
    let w = World::in_memory();
    let key = w.add(person("Amara", "a@example.org"));
    let mut s = start(&w, &key);
    give(&mut s, "ai");
    s.save();
    assert_eq!(roles_of(&w, &key), []);
}

#[test]
fn a_personal_role_has_no_group() {
    let w = World::in_memory();
    let key = w.add(person("Amara", "a@example.org"));
    let personal = fs_people_core::RoleDefinition::new(
        "buddy",
        "org:Role",
        "fs-people",
        fs_people_core::RoleScope::Personal,
    );
    w.runtime
        .block_on(fs_people_core::RoleBook::define_role(&*w.repo, personal))
        .unwrap();
    let mut s = start(&w, &key);
    give(&mut s, "buddy @ urn:group:1");
    s.save();
    assert!(
        error_of(&s).contains("is personal: leave the group out"),
        "{}",
        error_of(&s)
    );
    s.set_field(&role_field(&s, "buddy @ urn:group:1"), "buddy");
    s.save();
    assert!(s.editing().is_none(), "{}", error_of(&s));
    assert_eq!(roles_of(&w, &key), [("buddy".into(), None)]);
}

#[test]
fn a_role_that_cannot_be_given_leaves_the_old_one() {
    let w = World::in_memory();
    let key = w.add(person("Amara", "a@example.org"));
    let mut s = start(&w, &key);
    give(&mut s, &format!("member @ {GROUP}"));
    s.save();
    s.begin_edit();
    let old = role_field(&s, &format!("member @ {GROUP}"));
    s.set_field(&old, "wizard @ urn:group:1");
    s.save();
    assert!(s.editing().is_some());
    assert_eq!(roles_of(&w, &key), [("member".into(), Some(GROUP.into()))]);
}

#[test]
fn the_form_lists_the_roles_but_not_the_ai_marking() {
    let w = World::in_memory();
    let key = w.add(person("Amara", "a@example.org"));
    w.runtime
        .block_on(fs_people_core::RoleBook::assign_role(
            &*w.repo,
            key.parse().unwrap(),
            "member",
            Some(GROUP),
            serde_json::Map::default(),
        ))
        .unwrap();
    let bot = w.add(fs_people_core::Card::application("Bot"));
    w.runtime
        .block_on(fs_people_core::RoleBook::mark_ai(
            &*w.repo,
            bot.parse().unwrap(),
        ))
        .unwrap();
    let s = start(&w, &key);
    assert_eq!(
        s.editing()
            .unwrap()
            .draft
            .fields()
            .filter(|f| f.key.starts_with("roles/"))
            .count(),
        1
    );
    let mut s2 = w.state();
    s2.select(&bot);
    s2.begin_edit();
    assert_eq!(
        s2.editing()
            .unwrap()
            .draft
            .fields()
            .filter(|f| f.key.starts_with("roles/"))
            .count(),
        0
    );
    assert!(
        s2.editing()
            .unwrap()
            .draft
            .groups
            .iter()
            .any(|g| g.key == "roles" && g.can_add)
    );
}

#[test]
fn a_source_without_roles_says_so_in_the_form() {
    let w = World::in_memory();
    let key = w.add(person("Amara", "a@example.org"));
    let adapter = PeopleAdapter::new(
        w.repo.clone(),
        Arc::new(NoRoles),
        w.runtime.handle().clone(),
    );
    let draft = fs_components_workbenches::people::PeopleSource::draft(&adapter, &key).unwrap();
    assert!(draft.groups.iter().all(|g| g.key != "roles"));
    assert!(
        draft
            .marks
            .iter()
            .any(|m| m.starts_with("Roles: not offered by this source (code unsupported")),
        "{:?}",
        draft.marks
    );
}
