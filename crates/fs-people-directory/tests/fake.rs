//! Conformance and behaviour tests of `FakeDirectory` and `CompositeDirectory`.
//!
//! Pattern: Contract test (the rules of the locked trait run against the fake).

mod common;
use common::{Down, ReadOnly};
use fs_people_directory::conformance::{run_conformance, run_unsupported};
use fs_people_directory::{
    ChangeKind, CompositeDirectory, ContactAddress, Entry, EntryState, Error, Excerpt,
    ExcerptField, ExcerptValue, FakeDirectory, NoExcerptReason, PersonArt, PersonDirectory,
    PersonRef, Place, Resolved, Result, Suggest,
};
use fs_types::{BasicValue, Name, Reference, Text};

fn person(n: u32, art: PersonArt) -> PersonRef {
    let id = Reference::parse(&format!("urn:uuid:00000000-0000-7000-8000-{n:012x}")).expect("id");
    PersonRef {
        id,
        art,
        abzug: None,
    }
}

fn addr(s: &str) -> ContactAddress {
    ContactAddress::parse(s).expect("address")
}

fn query(text: &str, limit: u16) -> Suggest {
    Suggest {
        text: text.to_owned(),
        arten: vec![],
        limit,
        include_proposed: true,
    }
}

fn seeded() -> FakeDirectory {
    let mut ada = Entry::new(person(1, PersonArt::Person), "Ada Lovelace");
    ada.address = Some(addr("mailto:ada@example.org"));
    ada.excerpt = vec![
        ExcerptValue::FullName(Name::new("Ada Lovelace", &[]).expect("name")),
        ExcerptValue::Surname(Text::parse("Lovelace").expect("text")),
    ];
    let mut profile = Entry::new(person(2, PersonArt::Profile), "Ada L.");
    profile.address = Some(addr("mailto:ada@example.org"));
    profile.state = EntryState::Proposed;
    let team = Entry::new(person(3, PersonArt::Group), "Analytical Engine Team");
    FakeDirectory::new()
        .with_asker(Some(person(1, PersonArt::Person)))
        .with_entry(profile)
        .with_entry(ada)
        .with_entry(team)
}

#[tokio::test]
async fn fake_is_conformant() {
    let failures = run_conformance(&FakeDirectory::new()).await;
    assert!(failures.is_empty(), "{failures:?}");
    // Also on a filled directory.
    let failures = run_conformance(&seeded()).await;
    assert!(failures.is_empty(), "{failures:?}");
}

#[tokio::test]
async fn find_by_address_orders_person_before_profile() {
    let d = seeded();
    let found = d
        .find_by_address(&addr("mailto:ada@EXAMPLE.org"))
        .await
        .expect("find");
    // Confirmed before Proposed: the person (Confirmed) comes before the profile (Proposed, added first).
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].person.art, PersonArt::Person);
    assert_eq!(found[1].state, EntryState::Proposed);
}

#[tokio::test]
async fn trash_states_and_restored() {
    let d = seeded();
    let ada = person(1, PersonArt::Person);
    let team = person(3, PersonArt::Group);
    assert!(d.delete(&ada));
    assert!(matches!(
        d.resolve(&ada).await,
        Ok(Resolved::Deleted { .. })
    ));
    let gone = d.excerpt(&ada, &[ExcerptField::FullName]).await;
    assert!(matches!(
        gone,
        Err(Error::NoExcerpt {
            reason: NoExcerptReason::Deleted,
            ..
        })
    ));
    assert!(
        d.find_by_address(&addr("mailto:ada@example.org"))
            .await
            .expect("find")
            .iter()
            .all(|v| v.person.id != ada.id)
    );
    assert!(d.unshare(&team));
    assert!(matches!(d.resolve(&team).await, Ok(Resolved::NotShared)));
    let hidden = d.excerpt(&team, &[ExcerptField::FullName]).await;
    assert!(matches!(
        hidden,
        Err(Error::NoExcerpt {
            reason: NoExcerptReason::NotShared,
            ..
        })
    ));
    let hidden_team = d.suggest(&query("Analytical", 5)).await.expect("suggest");
    assert_eq!(hidden_team.len(), 0);
    assert!(d.restore(&ada));
    assert!(matches!(d.resolve(&ada).await, Ok(Resolved::Found(_))));
    let page = d
        .changes(Some(&Place("0".to_owned())), 100)
        .await
        .expect("changes");
    let kinds: Vec<&ChangeKind> = page.items.iter().map(|c| &c.kind).collect();
    assert!(kinds.contains(&&ChangeKind::Deleted));
    assert!(kinds.contains(&&ChangeKind::NotShared));
    assert!(kinds.contains(&&ChangeKind::Restored));
}

#[tokio::test]
async fn merged_profile_resolves_to_target() {
    let d = seeded();
    let (profile, ada) = (person(2, PersonArt::Profile), person(1, PersonArt::Person));
    assert!(d.merge(&profile, &ada));
    match d.resolve(&profile).await {
        Ok(Resolved::Found(v)) => assert_eq!(v.person.id, ada.id),
        other => panic!("expected Found, got {other:?}"),
    }
}

#[tokio::test]
async fn excerpt_lists_missing_and_asker_works() {
    let d = seeded();
    let ada = person(1, PersonArt::Person);
    let x = d
        .excerpt(&ada, &[ExcerptField::FullName, ExcerptField::Phone])
        .await
        .expect("excerpt");
    assert_eq!(x.values.len(), 1);
    assert_eq!(x.missing, vec![ExcerptField::Phone]);
    assert_eq!(d.asker().await.expect("asker").map(|p| p.id), Some(ada.id));
    assert!(FakeDirectory::new().asker().await.expect("asker").is_none());
}

#[tokio::test]
async fn closed_batch_is_refused() {
    let d = FakeDirectory::new();
    let origin = Reference::parse("urn:uuid:00000000-0000-7000-8000-00000000abcd").expect("ref");
    let b = d.open_batch(&origin).await.expect("batch");
    let e = fs_people_directory::ProposeEntry {
        display_hint: "X".into(),
        address: None,
        origin: None,
    };
    assert!(d.propose(&e, Some(b)).await.is_ok());
    d.close_batch(b);
    assert!(matches!(
        d.propose(&e, Some(b)).await,
        Err(Error::BatchClosed(_))
    ));
}

#[tokio::test]
async fn optional_methods_are_unsupported() {
    let failures = run_unsupported(&ReadOnly).await;
    assert!(failures.is_empty(), "{failures:?}");
    let x = ReadOnly
        .excerpt(&person(1, PersonArt::Person), &[ExcerptField::FullName])
        .await;
    assert!(matches!(x, Err(Error::Unsupported { .. })));
}

#[tokio::test]
async fn composite_chain_rules() {
    let chain =
        CompositeDirectory::new(vec![Box::new(Down), Box::new(ReadOnly), Box::new(seeded())]);
    let ada = person(1, PersonArt::Person);
    // Down and ReadOnly pass the question on; the fake answers.
    assert!(matches!(chain.resolve(&ada).await, Ok(Resolved::Found(_))));
    // Unknown with a member that is down is not "unknown": the chain cannot tell.
    assert!(matches!(
        chain.resolve(&person(99, PersonArt::Person)).await,
        Err(Error::Unavailable(_))
    ));
    let up = CompositeDirectory::new(vec![Box::new(ReadOnly), Box::new(seeded())]);
    assert!(matches!(
        up.resolve(&person(99, PersonArt::Person)).await,
        Ok(Resolved::Unknown)
    ));
    assert!(chain.excerpt(&ada, &[ExcerptField::FullName]).await.is_ok());
    let nobody = up
        .excerpt(&person(99, PersonArt::Person), &[ExcerptField::FullName])
        .await;
    assert!(matches!(
        nobody,
        Err(Error::NoExcerpt {
            reason: NoExcerptReason::Unknown,
            ..
        })
    ));
    assert_eq!(
        chain.asker().await.expect("asker").map(|p| p.id),
        Some(ada.id.clone())
    );
    let many = chain.resolve_many(&[ada.clone(), ada]).await.expect("many");
    assert_eq!(many.len(), 2);
    // person.propose goes to the first member that offers it (the fake), batch ids stay with their issuer.
    let origin = Reference::parse("urn:uuid:00000000-0000-7000-8000-00000000abcd").expect("ref");
    let b = chain.open_batch(&origin).await.expect("batch");
    let e = fs_people_directory::ProposeEntry {
        display_hint: "New".into(),
        address: None,
        origin: None,
    };
    assert!(chain.propose(&e, Some(b)).await.is_ok());
    let foreign = fs_types::ObjectId([7; 16]);
    assert!(matches!(
        chain.propose(&e, Some(foreign)).await,
        Err(Error::BatchClosed(_))
    ));
}

#[tokio::test]
async fn composite_deleted_is_final_and_all_down_is_unavailable() {
    let a = seeded();
    a.delete(&person(1, PersonArt::Person));
    let chain = CompositeDirectory::new(vec![Box::new(a), Box::new(seeded())]);
    assert!(matches!(
        chain.resolve(&person(1, PersonArt::Person)).await,
        Ok(Resolved::Deleted { .. })
    ));
    let down = CompositeDirectory::new(vec![Box::new(Down)]);
    assert!(matches!(
        down.resolve(&person(1, PersonArt::Person)).await,
        Err(Error::Unavailable(_))
    ));
    let x: Result<Excerpt> = down
        .excerpt(&person(1, PersonArt::Person), &[ExcerptField::FullName])
        .await;
    assert!(matches!(x, Err(Error::Unavailable(_))));
}
