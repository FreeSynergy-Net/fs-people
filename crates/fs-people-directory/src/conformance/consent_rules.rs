//! Rules of the consent methods: record, check, list, withdraw, rights and the purpose binding of `excerpt`.
//!
//! Pattern: Template method (rules as plain checks; people, rights and states come from the `ConformanceFixture`).

use super::{ConformanceFixture, Report, purposed};
use crate::person_directory_v1::{
    Consent, ConsentEntry, ConsentOrigin, ConsentQuery, Date, Error, ExcerptField, NoExcerptReason,
    ObjectId, PersonArt, PersonDirectory, PersonRef, Purpose, Reference, When, Withdrawal,
};
use fs_types::BasicValue;

pub(super) fn day() -> When {
    Date::parse("2026-11-16").map_or(When::At(fs_types::Utc::from_millis(0)), When::On)
}

/// A reference that stands for a room, a distribution list or a document (`scope`).
pub(super) fn scope(n: u32) -> Option<Reference> {
    Reference::parse(&format!("urn:uuid:00000000-0000-7000-8000-{n:012x}")).ok()
}

pub(super) fn entry(person: &PersonRef, purpose: Purpose) -> ConsentEntry {
    ConsentEntry {
        person: person.clone(),
        given_by: None,
        purpose,
        scope: None,
        recipient: None,
        given: day(),
        origin: ConsentOrigin::Form,
        evidence: None,
        text: None,
    }
}

pub(super) fn query(person: &PersonRef, purpose: Purpose) -> ConsentQuery {
    ConsentQuery {
        person: person.clone(),
        purpose,
        scope: None,
        recipient: None,
    }
}

pub(super) fn withdrawal() -> Withdrawal {
    Withdrawal {
        at: day(),
        origin: ConsentOrigin::Form,
        evidence: None,
    }
}

pub(super) async fn person(fx: &dyn ConformanceFixture, name: &str) -> Option<PersonRef> {
    fx.add_confirmed(PersonArt::Person, &format!("Zzconf {name}"), None)
        .await
}

pub(super) fn forbidden(e: &Error) -> bool {
    matches!(e, Error::Forbidden { .. })
}

pub(super) fn invalid(e: &Error) -> bool {
    matches!(e, Error::Invalid(_))
}

pub(super) async fn rules(fx: &dyn ConformanceFixture, r: &mut Report) {
    let Some(admin) = fx.admin_view().await else {
        r.skip("consent: fixture has no administrator view");
        return;
    };
    let (Some(p), Some(q)) = (person(fx, "Consent P").await, person(fx, "Consent Q").await) else {
        r.skip("consent: fixture cannot add_confirmed");
        return;
    };
    record_rules(&*admin, &p, &q, r).await;
    withdraw_rules(&*admin, &p, r).await;
    purpose_rules(fx, &*admin, r).await;
    super::consent_rights::rank_rules(fx, &*admin, &p, r).await;
    super::consent_rights::erasure_rules(fx, &*admin, r).await;
}

async fn record_rules(d: &dyn PersonDirectory, p: &PersonRef, q: &PersonRef, r: &mut Report) {
    let by = d.asker().await.ok().flatten();
    let first = d.record_consent(&entry(p, Purpose::Newsletter)).await;
    let Ok(first) = first else {
        r.ensure(
            false,
            "record_consent: an administrator records for another person",
        );
        return;
    };
    r.ensure(
        first.entry == entry(p, Purpose::Newsletter)
            && first.withdrawn.is_none()
            && first.retain_until.is_none()
            && by.is_some_and(|b| b.id == first.recorded_by.id),
        "record_consent: echoes the entry, active, recorded_by is the asker",
    );
    let got = d.check_consent(&query(p, Purpose::Newsletter)).await;
    r.ensure(
        got.is_ok_and(|id| id == Some(first.id)),
        "check_consent: the active record is found",
    );
    let other = d.check_consent(&query(p, Purpose::Photo)).await;
    r.ensure(
        other.is_ok_and(|id| id.is_none()),
        "check_consent: another purpose is None",
    );
    let other = d.check_consent(&query(q, Purpose::Newsletter)).await;
    r.ensure(
        other.is_ok_and(|id| id.is_none()),
        "check_consent: another person is None",
    );
    // A new proof is never dropped: a second record is an own entry; the check names the oldest.
    let second = d.record_consent(&entry(p, Purpose::Newsletter)).await;
    r.ensure(
        second.as_ref().is_ok_and(|s| s.id != first.id),
        "record_consent: a matching active consent still gets its own entry",
    );
    let got = d.check_consent(&query(p, Purpose::Newsletter)).await;
    r.ensure(
        got.is_ok_and(|id| id == Some(first.id)),
        "check_consent: several match, the oldest",
    );
    let listed = d.consents_of(p).await.unwrap_or_default();
    let ids: Vec<ObjectId> = listed.iter().map(|c| c.id).collect();
    r.ensure(
        second.is_ok_and(|s| ids == vec![first.id, s.id]),
        "consents_of: all records of the person, oldest first",
    );
    r.ensure(
        d.consents_of(q).await.is_ok_and(|l| l.is_empty()),
        "consents_of: a person without records has an empty list",
    );
    let mut by_guardian = entry(q, Purpose::Photo);
    by_guardian.given_by = Some(p.clone());
    let kept = d.record_consent(&by_guardian).await;
    r.ensure(
        kept.is_ok_and(|c| c.entry.given_by.as_ref().is_some_and(|g| g.id == p.id)),
        "record_consent: given_by is kept (guardian, Art. 8)",
    );
    r.ensure(
        d.check_consent(&query(p, Purpose::Photo))
            .await
            .is_ok_and(|id| id.is_none()),
        "check_consent: given_by is not a person the consent covers",
    );
}

async fn withdraw_rules(d: &dyn PersonDirectory, p: &PersonRef, r: &mut Report) {
    let Ok(c) = d.record_consent(&entry(p, Purpose::Publication)).await else {
        return;
    };
    let done = d.withdraw_consent(c.id, &withdrawal()).await;
    let Ok(done) = done else {
        r.ensure(false, "withdraw_consent: withdraws a record");
        return;
    };
    r.ensure(
        done.withdrawn.is_some()
            && done.retain_until.is_some()
            && done.entry == c.entry
            && done.id == c.id,
        "withdraw_consent: sets withdrawn and retain_until, keeps the entry",
    );
    let got = d.check_consent(&query(p, Purpose::Publication)).await;
    r.ensure(
        got.is_ok_and(|id| id.is_none()),
        "check_consent: a withdrawn record does not count",
    );
    let kept = d.consents_of(p).await.unwrap_or_default();
    r.ensure(
        kept.iter()
            .any(|k: &Consent| k.id == c.id && k.withdrawn.is_some()),
        "consents_of: a withdrawal never deletes the record",
    );
    let mut later = withdrawal();
    later.origin = ConsentOrigin::Oral;
    let again = d.withdraw_consent(c.id, &later).await;
    r.ensure(
        again.is_ok_and(|a| a.withdrawn == done.withdrawn),
        "withdraw_consent: a second withdrawal returns the first unchanged",
    );
    let unknown = d
        .withdraw_consent(ObjectId([0xAB; 16]), &withdrawal())
        .await;
    r.is_err(
        &unknown,
        "withdraw_consent: an unknown id is UnknownConsent",
        |e| matches!(e, Error::UnknownConsent(_)),
    );
}

async fn purpose_rules(fx: &dyn ConformanceFixture, d: &dyn PersonDirectory, r: &mut Report) {
    let Some(x) = person(fx, "Purpose").await else {
        return;
    };
    let fields = [ExcerptField::FullName];
    let ask = |purpose| purposed(&fields, purpose);
    let none = d.excerpt(&x, &ask(Purpose::Photo)).await;
    r.is_err(
        &none,
        "excerpt: a purpose with consent duty and no consent is NoConsent",
        |e| {
            matches!(
                e,
                Error::NoExcerpt {
                    reason: NoExcerptReason::NoConsent,
                    ..
                }
            )
        },
    );
    let Ok(c) = d.record_consent(&entry(&x, Purpose::Photo)).await else {
        return;
    };
    let with = d.excerpt(&x, &ask(Purpose::Photo)).await;
    r.ensure(
        with.is_ok_and(|e| e.consent == Some(c.id) && e.purpose == Purpose::Photo),
        "excerpt: with a consent the answer names it and the purpose",
    );
    let narrower = d.excerpt(&x, &ask(Purpose::Newsletter)).await;
    r.ensure(
        narrower.is_err(),
        "excerpt: a consent for one purpose does not carry another",
    );
    let mut in_room = ask(Purpose::Photo);
    in_room.scope = scope(7);
    r.ensure(
        d.excerpt(&x, &in_room)
            .await
            .is_ok_and(|e| e.consent == Some(c.id)),
        "excerpt: a consent without scope covers every scope",
    );
    if d.withdraw_consent(c.id, &withdrawal()).await.is_ok() {
        r.is_err(
            &d.excerpt(&x, &ask(Purpose::Photo)).await,
            "excerpt: after the withdrawal NoConsent again",
            |e| {
                matches!(
                    e,
                    Error::NoExcerpt {
                        reason: NoExcerptReason::NoConsent,
                        ..
                    }
                )
            },
        );
    }
    let bad = d.record_consent(&entry(&x, Purpose::AiAssistance)).await;
    r.is_err(
        &bad,
        "record_consent: AiAssistance without scope is Invalid",
        invalid,
    );
    let mut room = entry(&x, Purpose::AiAssistance);
    room.scope = scope(8);
    r.ensure(
        d.record_consent(&room).await.is_ok(),
        "record_consent: AiAssistance with scope",
    );
    let nobody = PersonRef {
        id: scope(0xdead).unwrap_or_else(|| x.id.clone()),
        art: PersonArt::Person,
        abzug: None,
    };
    let unknown = d.record_consent(&entry(&nobody, Purpose::Newsletter)).await;
    r.is_err(
        &unknown,
        "record_consent: an unknown person is Invalid",
        invalid,
    );
}
