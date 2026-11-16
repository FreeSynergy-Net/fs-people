//! Tests of the P0' parts: `sample`, the fixture rules, the doubles and the composite refinements.
//!
//! Pattern: Contract test (shared rules run against fake, composite and doubles).

use fs_people_directory::conformance::{run_conformance, run_fixture_conformance};
use fs_people_directory::{
    CompositeDirectory, EntryState, Error, ExcerptField, FailingDirectory, FakeDirectory,
    FakeFixture, Gate, PersonArt, PersonDirectory, ProposeEntry, Resolved, SlowDirectory, Suggest,
    sample, sample_ref,
};
use std::time::Duration;

fn r(n: u8) -> fs_people_directory::PersonRef {
    sample_ref(n).expect("a sample number")
}

#[tokio::test]
async fn fixture_rules_run_against_the_fake_without_skips() {
    let report = run_fixture_conformance(&FakeFixture::new()).await;
    assert_eq!(report.failures, Vec::<String>::new());
    assert_eq!(report.skipped, Vec::<String>::new());
    let report = run_fixture_conformance(&FakeFixture::over(sample())).await;
    assert_eq!(report.failures, Vec::<String>::new());
}

#[tokio::test]
async fn composite_is_conformant_too() {
    let one = CompositeDirectory::new(vec![Box::new(FakeDirectory::new())]);
    assert_eq!(run_conformance(&one).await, Vec::<String>::new());
    // A member that is down would make an unknown reference `Unavailable` (documented), so the second member is a fake.
    let two = CompositeDirectory::new(vec![Box::new(FakeDirectory::new()), Box::new(sample())]);
    assert_eq!(run_conformance(&two).await, Vec::<String>::new());
}

#[tokio::test]
async fn sample_covers_every_state() {
    let d = sample();
    assert_eq!(d.asker().await.expect("asker"), Some(r(1)));
    assert!(matches!(
        d.resolve(&r(7)).await,
        Ok(Resolved::Deleted { .. })
    ));
    assert!(matches!(d.resolve(&r(8)).await, Ok(Resolved::NotShared)));
    assert!(matches!(d.resolve(&r(9)).await, Ok(Resolved::Found(v)) if v.person.id == r(10).id));
    assert!(matches!(d.resolve(&r(2)).await, Ok(Resolved::Found(v)) if v.describes == Some(r(1))));
    assert!(matches!(d.resolve(&r(3)).await, Ok(Resolved::Found(v)) if v.describes.is_none()));
    assert!(matches!(d.resolve(&r(4)).await, Ok(Resolved::Found(v)) if v.ai));
    assert!(
        matches!(d.resolve(&r(11)).await, Ok(Resolved::Found(v)) if v.state == EntryState::Proposed)
    );
    let amara = fs_types::ContactAddress::parse("mailto:amara@example.org").expect("address");
    let ids: Vec<_> = d
        .find_by_address(&amara)
        .await
        .expect("find")
        .into_iter()
        .map(|v| v.person.id)
        .collect();
    assert_eq!(ids, vec![r(1).id, r(2).id, r(3).id]);
    let q = Suggest {
        text: String::new(),
        arten: vec![PersonArt::Organization],
        limit: 5,
        include_proposed: true,
    };
    assert_eq!(d.suggest(&q).await.expect("suggest").len(), 1);
}

#[tokio::test]
async fn every_error_variant_is_reachable() {
    let all = FailingDirectory::each_variant();
    assert_eq!(all.len(), 11);
    for f in all {
        assert!(f.resolve(&r(1)).await.is_err());
        assert!(f.asker().await.is_err());
        assert!(f.excerpt(&r(1), &[ExcerptField::FullName]).await.is_err());
        let e = ProposeEntry {
            display_hint: "x".into(),
            address: None,
            origin: None,
        };
        assert!(f.propose(&e, None).await.is_err());
    }
}

#[tokio::test]
async fn a_slow_call_can_be_cancelled_and_then_goes_through() {
    let gate = Gate::closed();
    let slow = SlowDirectory::new(sample(), gate.clone());
    let late = tokio::time::timeout(Duration::from_millis(20), slow.resolve(&r(1))).await;
    assert!(late.is_err(), "the held call runs into the time limit");
    assert_eq!(
        slow.passed(),
        0,
        "a cancelled call never reached the inner provider"
    );
    gate.open();
    assert!(matches!(slow.resolve(&r(1)).await, Ok(Resolved::Found(_))));
    assert_eq!(slow.passed(), 1);
}

#[tokio::test]
async fn composite_resolve_many_asks_each_member_once() {
    let gate = Gate::closed();
    gate.open();
    let first = SlowDirectory::new(FakeDirectory::new(), gate.clone());
    let second = SlowDirectory::new(sample(), gate);
    let chain = CompositeDirectory::new(vec![Box::new(first), Box::new(second)]);
    let people = [r(1), r(2), r(7), r(1), r(3)];
    let answers = chain.resolve_many(&people).await.expect("many");
    assert_eq!(answers.len(), 5);
    assert!(matches!(answers[2], Resolved::Deleted { .. }));
    assert!(matches!(&answers[3], Resolved::Found(v) if v.person.id == r(1).id));
}

#[tokio::test]
async fn composite_bad_limit_is_invalid_even_when_all_are_down() {
    let down = FailingDirectory::new(Error::Unavailable(
        fs_people_directory::Unreachable::Timeout,
    ));
    let chain = CompositeDirectory::new(vec![Box::new(down)]);
    let q = Suggest {
        text: String::new(),
        arten: vec![],
        limit: 51,
        include_proposed: true,
    };
    assert!(matches!(chain.suggest(&q).await, Err(Error::Invalid(_))));
    let ok = Suggest { limit: 5, ..q };
    assert!(matches!(
        chain.suggest(&ok).await,
        Err(Error::Unavailable(_))
    ));
}

#[tokio::test]
async fn composite_suggest_merges_by_position_and_changes_skip_unsupported() {
    let a = FakeDirectory::new();
    let q = Suggest {
        text: "Amara".into(),
        arten: vec![],
        limit: 10,
        include_proposed: true,
    };
    let chain = CompositeDirectory::new(vec![Box::new(a), Box::new(sample())]);
    let list = chain.suggest(&q).await.expect("suggest");
    assert!(list.iter().any(|c| c.view.person.id == r(1).id));
    let unsupported = FailingDirectory::new(Error::Unsupported {
        capability: "person.read".into(),
        reason: "no".into(),
    });
    let chain = CompositeDirectory::new(vec![Box::new(unsupported), Box::new(sample())]);
    assert!(chain.changes(None, 5).await.is_ok());
}

#[tokio::test]
async fn fake_merge_bumps_rev_and_excerpt_follows() {
    let d = sample();
    let before = match d.resolve(&r(10)).await {
        Ok(Resolved::Found(v)) => v.rev,
        other => panic!("{other:?}"),
    };
    assert!(d.merge(&r(12), &r(1)));
    assert!(d.rename(&r(10), "Soo-jin K. Marchetti"));
    match d.resolve(&r(10)).await {
        Ok(Resolved::Found(v)) => assert!(v.rev > before),
        other => panic!("{other:?}"),
    }
    let x = d
        .excerpt(&r(12), &[ExcerptField::FullName])
        .await
        .expect("follows the merge");
    assert_eq!(x.person.id, r(1).id);
}
