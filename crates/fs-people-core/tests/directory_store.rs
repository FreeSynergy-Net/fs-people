//! The typed tables of the People directory in fs-objects v5, part 1: consents and their withdrawal.
//!
//! Pattern: Contract test (what the locked trait names goes in and comes out equal; the engine refuses the rest).

use fs_objects_v5::v5::Error;
use fs_people_core::directory_store::consent;
use fs_people_directory::{ConsentOrigin, ConsentRetention, Purpose, Reference, Revision, When};
use fs_types::BasicValue;

mod common;
use common::dir::*;

#[tokio::test]
async fn a_consent_with_every_field_comes_back_equal() {
    let s = store().await;
    let mut e = entry(1);
    e.given_by = Some(person(2));
    e.purpose = Purpose::Other(Reference::parse("https://w3id.org/dpv#Research").unwrap());
    e.scope = Some(Reference::parse("urn:uuid:01927f00-0000-7000-8000-0000000000aa").unwrap());
    e.recipient = Some(person(3));
    e.given = When::On(day("2026-11-16"));
    e.origin = ConsentOrigin::Oral;
    e.evidence = Some(Reference::parse(&format!("urn:sha256:{}", "ab".repeat(32))).unwrap());
    e.text = Some(Reference::parse("https://example.org/consent/v2").unwrap());
    let c = consent_of(1, e);
    let tx = s.begin().await.unwrap();
    let r = consent::record(&tx, &c).await;
    tx.finish(r).await.unwrap();
    let c2 = consent_of(2, entry(1));
    let tx = s.begin().await.unwrap();
    let r = consent::record(&tx, &c2).await;
    tx.finish(r).await.unwrap();
    let read = s.read().await;
    assert_eq!(consent::get(&read, c.id).await.unwrap(), Some(c.clone()));
    assert_eq!(
        consent::of_person(&read, &person(1)).await.unwrap(),
        vec![c, c2]
    );
    let nobody = consent::of_person(&read, &person(7)).await.unwrap();
    assert_eq!(nobody.len(), 0);
}

#[tokio::test]
async fn a_withdrawal_keeps_the_row_bumps_the_revision_and_sets_the_retention() {
    let s = store().await;
    let c = consent_of(1, entry(1));
    let tx = s.begin().await.unwrap();
    consent::record(&tx, &c).await.unwrap();
    tx.finish(Ok(())).await.unwrap();
    let w = withdrawal(When::On(day("2026-03-02")));
    let until = ConsentRetention::default().after(&w.withdrawal.at).unwrap();
    assert_eq!(until, day("2029-12-31"));
    let tx = s.begin().await.unwrap();
    consent::withdraw(&tx, c.id, Revision(1), &w, until)
        .await
        .unwrap();
    tx.finish(Ok(())).await.unwrap();
    let now = consent::get(&s.read().await, c.id).await.unwrap().unwrap();
    assert_eq!(now.rev, Revision(2));
    assert_eq!(now.withdrawn, Some(w.clone()));
    assert_eq!(now.retain_until, Some(until));
    // The same revision again: the consent is withdrawn already; a stale one on an active row is `Changed`.
    let tx = s.begin().await.unwrap();
    let again = consent::withdraw(&tx, c.id, Revision(2), &w, until).await;
    assert!(matches!(again, Err(Error::Invalid(_))), "{again:?}");
    tx.finish(Ok(())).await.unwrap();
    let other = consent_of(2, entry(1));
    let tx = s.begin().await.unwrap();
    consent::record(&tx, &other).await.unwrap();
    let stale = consent::withdraw(&tx, other.id, Revision(5), &w, until).await;
    assert_eq!(
        stale,
        Err(Error::Changed {
            current: Revision(1)
        })
    );
    let missing = id(77);
    let gone = consent::withdraw(&tx, missing, Revision(1), &w, until).await;
    assert_eq!(gone, Err(Error::NotFound(missing)));
    tx.finish(Ok(())).await.unwrap();
}

#[tokio::test]
async fn the_engine_refuses_what_the_columns_forbid() {
    let s = store().await;
    // A new record cannot start withdrawn.
    let mut c = consent_of(1, entry(1));
    c.withdrawn = Some(withdrawal(When::On(day("2026-03-02"))));
    let tx = s.begin().await.unwrap();
    assert!(matches!(
        consent::record(&tx, &c).await,
        Err(Error::Invalid(_))
    ));
    // Erasure is an origin of a withdrawal only.
    let mut e = entry(1);
    e.origin = ConsentOrigin::Erasure;
    let r = consent::record(&tx, &consent_of(2, e)).await;
    assert!(matches!(r, Err(Error::Invalid(_))), "{r:?}");
    // A half withdrawal (origin without a day) is refused by the table, not only by the code.
    tx.finish(Ok(())).await.unwrap();
    let tx = s.begin().await.unwrap();
    consent::record(&tx, &consent_of(3, entry(1)))
        .await
        .unwrap();
    let r = tx
        .run("UPDATE people_consent SET withdrawn_origin = 'form' WHERE rev = 1")
        .await;
    assert!(matches!(r, Err(Error::Invalid(_))), "{r:?}");
    tx.finish(Ok(())).await.unwrap();
}
