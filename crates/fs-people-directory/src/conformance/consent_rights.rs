//! Rules of the consent rights (own records, `consent.read`/`consent.write`, rank of the answers) and of erasure.
//!
//! Pattern: Template method (rules as plain checks; the rights come from `ConformanceFixture::member_view`).

use super::consent_rules::{entry, forbidden, invalid, person, query, withdrawal};
use super::{ConformanceFixture, Report};
use crate::person_directory_v1::{ConsentOrigin, Error, PersonDirectory, PersonRef, Purpose};

pub(super) async fn rank_rules(
    fx: &dyn ConformanceFixture,
    d: &dyn PersonDirectory,
    p: &PersonRef,
    r: &mut Report,
) {
    let (Some(m), Some(other)) = (person(fx, "Member").await, person(fx, "Other").await) else {
        return;
    };
    let Ok(foreign) = d.record_consent(&entry(&other, Purpose::Newsletter)).await else {
        return;
    };
    let (Some(plain), Some(reader)) = (
        fx.member_view(&m, &[]).await,
        fx.member_view(&m, &["consent.read"]).await,
    ) else {
        r.skip("consent rights: fixture has no member view");
        return;
    };
    r.ensure(
        plain
            .record_consent(&entry(&m, Purpose::Photo))
            .await
            .is_ok(),
        "record_consent: a person records for themselves",
    );
    r.is_err(
        &plain.record_consent(&entry(p, Purpose::Photo)).await,
        "record_consent: for others needs consent.write",
        forbidden,
    );
    r.is_err(
        &plain.consents_of(&other).await,
        "consents_of: for others needs consent.read",
        forbidden,
    );
    r.ensure(
        plain.consents_of(&m).await.is_ok_and(|l| l.len() == 1),
        "consents_of: a person reads their own",
    );
    let hidden = plain.withdraw_consent(foreign.id, &withdrawal()).await;
    r.is_err(
        &hidden,
        "withdraw_consent: an invisible record is UnknownConsent, not Forbidden",
        |e| matches!(e, Error::UnknownConsent(_)),
    );
    let seen = reader.withdraw_consent(foreign.id, &withdrawal()).await;
    r.is_err(
        &seen,
        "withdraw_consent: a visible record without consent.write is Forbidden",
        forbidden,
    );
    let own = plain.consents_of(&m).await.unwrap_or_default();
    if let Some(mine) = own.first() {
        r.ensure(
            plain.withdraw_consent(mine.id, &withdrawal()).await.is_ok(),
            "withdraw_consent: a person always withdraws their own",
        );
    }
}

pub(super) async fn erasure_rules(
    fx: &dyn ConformanceFixture,
    d: &dyn PersonDirectory,
    r: &mut Report,
) {
    let Some(gone) = person(fx, "Erased").await else {
        return;
    };
    if d.record_consent(&entry(&gone, Purpose::Newsletter))
        .await
        .is_err()
        || !fx.delete(&gone).await
    {
        r.skip("consent erasure: fixture cannot delete");
        return;
    }
    let list = d.consents_of(&gone).await.unwrap_or_default();
    r.ensure(
        list.len() == 1
            && list[0]
                .withdrawn
                .as_ref()
                .is_some_and(|w| w.withdrawal.origin == ConsentOrigin::Erasure),
        "delete: active consents count as withdrawn with Erasure and stay readable",
    );
    r.ensure(
        d.check_consent(&query(&gone, Purpose::Newsletter))
            .await
            .is_ok_and(|id| id.is_none()),
        "check_consent: a deleted person's consent does not count",
    );
    let tombstone = d.record_consent(&entry(&gone, Purpose::Photo)).await;
    r.is_err(
        &tombstone,
        "record_consent: a deleted person is Invalid",
        invalid,
    );
}
