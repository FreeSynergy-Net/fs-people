//! Tests of the consent parts: composite routing, the capability list, the unsupported provider, the value objects.
//!
//! Pattern: Contract test (shared rules and plain checks against the fake, the composite and the doubles).

mod common;
use common::ReadOnly;
use fs_people_directory::conformance::{run_conformance, run_unsupported};
use fs_people_directory::{
    Birthday, CAPABILITIES, CompositeDirectory, ConsentEntry, ConsentOrigin, ConsentQuery, Date,
    Error, ExcerptField, ExcerptRequest, FailingDirectory, FakeDirectory, MonthDay,
    NoExcerptReason, PersonDirectory, Purpose, Rights, When, sample, sample_ref,
};
use fs_types::BasicValue;

fn entry(purpose: Purpose) -> ConsentEntry {
    ConsentEntry {
        person: sample_ref(12).expect("sample"),
        given_by: None,
        purpose,
        scope: None,
        recipient: None,
        given: When::On(Date::parse("2026-11-16").expect("date")),
        origin: ConsentOrigin::Form,
        evidence: None,
        text: None,
    }
}

fn request(purpose: Purpose) -> ExcerptRequest {
    ExcerptRequest {
        fields: vec![ExcerptField::FullName],
        purpose,
        scope: None,
        recipient: None,
    }
}

#[test]
fn eight_capabilities_and_the_purpose_rule() {
    assert_eq!(CAPABILITIES.len(), 8);
    assert!(CAPABILITIES.contains(&"consent.check") && CAPABILITIES.contains(&"consent.write"));
    for p in [Purpose::Contract, Purpose::LegalObligation, Purpose::OwnUse] {
        assert!(!p.needs_consent());
    }
    for p in [
        Purpose::Photo,
        Purpose::Publication,
        Purpose::Newsletter,
        Purpose::AiAssistance,
    ] {
        assert!(p.needs_consent());
    }
    assert!(ExcerptField::BirthMonthDay.sensitive() && ExcerptField::Birthday.sensitive());
    assert!(!ExcerptField::FullName.sensitive());
}

#[test]
fn month_day_and_birthday_are_checked() {
    assert!(MonthDay::new(2, 29).is_ok());
    assert!(
        MonthDay::new(2, 30).is_err()
            && MonthDay::new(13, 1).is_err()
            && MonthDay::new(4, 31).is_err()
    );
    let day = Date::parse("1990-07-04").expect("date");
    assert_ne!(
        Birthday::Date(day),
        Birthday::MonthDay(MonthDay::new(7, 4).expect("md"))
    );
}

#[tokio::test]
async fn composite_sends_consent_methods_to_the_first_member_with_the_capability() {
    let chain = CompositeDirectory::new(vec![
        Box::new(ReadOnly),
        Box::new(FailingDirectory::new(Error::Forbidden {
            capability: "x".to_owned(),
        })),
        Box::new(sample()),
    ]);
    let who = sample_ref(12).expect("sample");
    let recorded = chain.record_consent(&entry(Purpose::Newsletter)).await;
    assert!(
        recorded.is_ok(),
        "the fake is the first with consent.write: {recorded:?}"
    );
    let q = ConsentQuery {
        person: who.clone(),
        purpose: Purpose::Newsletter,
        scope: None,
        recipient: None,
    };
    assert!(matches!(chain.check_consent(&q).await, Ok(Some(_))));
    assert_eq!(chain.consents_of(&who).await.expect("list").len(), 1);
    let none = CompositeDirectory::new(vec![Box::new(ReadOnly)]);
    assert!(matches!(
        none.consents_of(&who).await,
        Err(Error::Unsupported { .. })
    ));
    assert!(matches!(
        none.record_consent(&entry(Purpose::Photo)).await,
        Err(Error::Unsupported { .. })
    ));
}

#[tokio::test]
async fn composite_excerpt_no_consent_is_final_but_unknown_passes_on() {
    let first = sample();
    let second = sample();
    let who = sample_ref(12).expect("sample");
    let chain = CompositeDirectory::new(vec![Box::new(first), Box::new(second)]);
    // The first member knows the person and refuses (no consent); the second would refuse too, but never asked.
    let refused = chain.excerpt(&who, &request(Purpose::Newsletter)).await;
    assert!(matches!(
        refused,
        Err(Error::NoExcerpt {
            reason: NoExcerptReason::NoConsent,
            ..
        })
    ));
    // A member that does not know the person passes the question on, to the member that has the consent.
    let knows = sample();
    knows
        .record_consent(&entry(Purpose::Newsletter))
        .await
        .expect("record");
    let chain = CompositeDirectory::new(vec![Box::new(FakeDirectory::new()), Box::new(knows)]);
    assert!(
        chain
            .excerpt(&who, &request(Purpose::Newsletter))
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn unsupported_provider_says_so_for_every_new_method() {
    let failures = run_unsupported(&ReadOnly).await;
    assert!(failures.is_empty(), "{failures:?}");
}

#[tokio::test]
async fn the_composite_of_fakes_is_conformant_with_rights_default_all() {
    let d = FakeDirectory::new().with_rights(Rights::all());
    assert_eq!(run_conformance(&d).await, Vec::<String>::new());
}

#[tokio::test]
async fn the_ai_of_the_sample_stays_an_ai_after_unshare() {
    let d = sample();
    let ai = sample_ref(4).expect("sample");
    let bot = d.view_as(Some(ai.clone()), Rights::all());
    let addr = fs_types::ContactAddress::parse("mailto:amara@example.org").expect("address");
    assert!(
        bot.find_by_address(&addr)
            .await
            .is_err_and(|e| matches!(e, Error::Forbidden { .. }))
    );
    assert!(d.unshare(&ai));
    assert!(
        bot.find_by_address(&addr)
            .await
            .is_err_and(|e| matches!(e, Error::Forbidden { .. }))
    );
    assert!(
        bot.consents_of(&ai)
            .await
            .is_err_and(|e| matches!(e, Error::Forbidden { .. }))
    );
}
