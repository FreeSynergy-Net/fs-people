//! The typed tables of the People directory in fs-objects v5, part 2: snapshot, birthday, the setting
//! `consent_retention`, the registry steps and the storage check (ADR 0086).
//!
//! Pattern: Contract test (what the locked trait names goes in and comes out equal; the engine refuses the rest).

use fs_objects_v5::v5::schema::json_in_text;
use fs_objects_v5::v5::{Domain, Error, Step};
use fs_people_core::directory_store::{
    self, CONSENT, DOMAIN, SNAPSHOT, birthday_of, consent, consent_retention, set_birthday,
    set_consent_retention, snapshot,
};
use fs_people_directory::{
    Birthday, ConsentRetention, ExcerptValue, MonthDay, Name, Purpose, Reference, Utc, When,
};
use fs_types::BasicValue;

mod common;
use common::dir::*;

#[tokio::test]
async fn a_snapshot_keeps_purpose_consent_and_both_forms_of_the_birthday() {
    let s = store().await;
    let c = consent_of(1, entry(1));
    let mut with_year = snap(10, excerpt(Some(c.id)));
    with_year.excerpt.purpose =
        Purpose::Other(Reference::parse("https://w3id.org/dpv#Billing").unwrap());
    let mut no_year = snap(11, excerpt(None));
    no_year.excerpt.values = vec![ExcerptValue::Birthday(Birthday::MonthDay(
        MonthDay::new(2, 29).unwrap(),
    ))];
    let tx = s.begin().await.unwrap();
    for x in [&with_year, &no_year] {
        snapshot::insert(&tx, x).await.unwrap();
    }
    tx.finish(Ok(())).await.unwrap();
    let read = s.read().await;
    assert_eq!(
        snapshot::get(&read, with_year.id).await.unwrap(),
        Some(with_year)
    );
    assert_eq!(
        snapshot::get(&read, no_year.id).await.unwrap(),
        Some(no_year)
    );
    assert_eq!(snapshot::get(&read, id(99)).await.unwrap(), None);
}

#[tokio::test]
async fn a_snapshot_with_a_field_twice_or_a_wrong_keep_date_is_refused() {
    let s = store().await;
    let mut twice = snap(1, excerpt(None));
    twice
        .excerpt
        .values
        .push(ExcerptValue::FullName(Name::new("Other", &[]).unwrap()));
    let mut early = snap(2, excerpt(None));
    early.keep_until = day("2020-01-01");
    let tx = s.begin().await.unwrap();
    assert!(matches!(
        snapshot::insert(&tx, &twice).await,
        Err(Error::Invalid(_))
    ));
    let r = snapshot::insert(&tx, &early).await;
    assert!(matches!(r, Err(Error::Invalid(_))), "{r:?}");
    tx.finish(Ok(())).await.unwrap();
}

#[tokio::test]
async fn the_birthday_is_a_date_or_a_month_and_day_and_replaces_itself() {
    let s = store().await;
    let p = person(1);
    assert_eq!(birthday_of(&s.read().await, &p).await.unwrap(), None);
    for b in [
        Birthday::Date(day("1990-05-17")),
        Birthday::MonthDay(MonthDay::new(5, 17).unwrap()),
    ] {
        let tx = s.begin().await.unwrap();
        set_birthday(&tx, &p, &b, Utc::from_millis(5))
            .await
            .unwrap();
        tx.finish(Ok(())).await.unwrap();
        assert_eq!(birthday_of(&s.read().await, &p).await.unwrap(), Some(b));
    }
}

#[tokio::test]
async fn consent_retention_is_three_years_until_set_and_stays_in_range() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("people.db");
    let path = path.to_str().unwrap();
    let s = directory_store::open(path, 0).await.unwrap();
    assert_eq!(consent_retention(&s.read().await).await.unwrap().years(), 3);
    let tx = s.begin().await.unwrap();
    assert!(set_consent_retention(&tx, 0).await.is_err());
    assert!(set_consent_retention(&tx, 11).await.is_err());
    assert_eq!(set_consent_retention(&tx, 5).await.unwrap().years(), 5);
    tx.finish(Ok(())).await.unwrap();
    drop(s);
    let again = directory_store::open(path, 1).await.unwrap();
    let years = consent_retention(&again.read().await).await.unwrap();
    assert_eq!(years.years(), 5);
    assert_eq!(
        years.after(&When::On(day("2026-11-16"))),
        Some(day("2031-12-31"))
    );
    assert_eq!(ConsentRetention::new(3), Some(ConsentRetention::default()));
}

#[tokio::test]
async fn opening_twice_is_the_same_and_a_changed_step_is_found() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("people.db");
    let path = path.to_str().unwrap();
    drop(directory_store::open(path, 0).await.unwrap());
    let s = directory_store::open(path, 1).await.unwrap();
    let altered = Domain::new(
        "people",
        "altered",
        DOMAIN.tables,
        Box::leak(Box::new([Step::Sql("DELETE FROM meta WHERE key = 'none'")])),
    );
    let r = s.register_domain(&altered, 2).await;
    assert!(
        matches!(r, Err(Error::StepMismatch { step: 1, .. })),
        "{r:?}"
    );
}

#[tokio::test]
async fn every_column_is_typed_and_no_text_is_json() {
    let s = store().await;
    let tx = s.begin().await.unwrap();
    consent::record(&tx, &consent_of(1, entry(1)))
        .await
        .unwrap();
    snapshot::insert(&tx, &snap(2, excerpt(None)))
        .await
        .unwrap();
    // The registry checked STRICT and the blob rule when the step ran; the JSON heuristic runs on real rows.
    json_in_text(&tx, &[CONSENT, SNAPSHOT]).await.unwrap();
    tx.finish(Ok(())).await.unwrap();
}
