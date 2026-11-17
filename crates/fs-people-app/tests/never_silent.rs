//! B1: what a source does not offer is said, never left out. A source without roles shows the
//! section "Roles" with the code `unsupported` and the capability `role.read`; one that fails shows
//! the failure.
//!
//! Pattern: Integration test over the real adapter with the shipped `NoRoles` double (every method
//! answers `Unsupported`) and a double that fails; own oracle: the words of the section.
//! Mutant: the arm for `Unsupported` in `role_section` back to `Vec::new()` (section silently gone):
//! the first test is red.

mod common;

use std::sync::Arc;

use async_trait::async_trait;
use common::{World, person};
use fs_components_workbenches::people::PeopleSource;
use fs_people_app::{NoRoles, PeopleAdapter};
use fs_people_core::{Error, RoleBook};

/// A role book whose source is broken.
struct Broken;

#[async_trait]
impl RoleBook for Broken {
    async fn roles_of(
        &self,
        _person: fs_objects::ObjectId,
    ) -> fs_people_core::Result<Vec<fs_people_core::RoleAssignment>> {
        Err(Error::Stored("the role file is damaged".into()))
    }
}

fn roles_text(sheet: &fs_components_workbenches::people::PersonSheet) -> Option<String> {
    sheet.sections.iter().find(|s| s.title == "Roles").map(|s| {
        s.fields
            .iter()
            .map(|f| f.value.clone())
            .collect::<Vec<_>>()
            .join("|")
    })
}

#[test]
fn a_source_without_roles_shows_the_section_with_code_and_capability() {
    let w = World::in_memory();
    let key = w.add(person("Amara", "amara@x.org"));
    let adapter = PeopleAdapter::new(
        w.repo.clone(),
        Arc::new(NoRoles),
        w.runtime.handle().clone(),
    );
    let text = roles_text(&adapter.read(&key).unwrap()).expect("the section is there");
    assert!(text.contains("unsupported"), "{text}");
    assert!(text.contains("role.read"), "{text}");
    assert!(text.contains("not offered by this source"), "{text}");
}

#[test]
fn a_failing_role_book_shows_the_failure() {
    let w = World::in_memory();
    let key = w.add(person("Amara", "amara@x.org"));
    let adapter = PeopleAdapter::new(w.repo.clone(), Arc::new(Broken), w.runtime.handle().clone());
    let text = roles_text(&adapter.read(&key).unwrap()).expect("the section is there");
    assert!(text.contains("could not be read"), "{text}");
    assert!(text.contains("damaged"), "{text}");
}

#[test]
fn a_source_with_roles_and_none_assigned_has_no_such_line() {
    let w = World::in_memory();
    let key = w.add(person("Amara", "amara@x.org"));
    let sheet = w.adapter.read(&key).unwrap();
    // No role held: the section stays out, there is nothing to say.
    assert_eq!(roles_text(&sheet), None);
}
