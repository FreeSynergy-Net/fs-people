//! Rules of the matching rule of `ConsentQuery`: scope, recipient, profiles over `describes`, merged references.
//!
//! Pattern: Template method (rules as plain checks; states come from the `ConformanceFixture`).

use super::consent_rules::{entry, person, query, scope};
use super::{ConformanceFixture, Report};
use crate::person_directory_v1::{ObjectId, PersonArt, PersonDirectory, Purpose};

async fn found(
    d: &dyn PersonDirectory,
    q: &crate::person_directory_v1::ConsentQuery,
) -> Option<ObjectId> {
    d.check_consent(q).await.ok().flatten()
}

pub(super) async fn rules(fx: &dyn ConformanceFixture, r: &mut Report) {
    let Some(admin) = fx.admin_view().await else {
        return;
    };
    let d = &*admin;
    scope_rules(fx, d, r).await;
    profile_rules(fx, d, r).await;
    merge_rules(fx, d, r).await;
}

async fn scope_rules(fx: &dyn ConformanceFixture, d: &dyn PersonDirectory, r: &mut Report) {
    let (Some(p), Some(who)) = (
        person(fx, "Match P").await,
        person(fx, "Match Recipient").await,
    ) else {
        return;
    };
    let mut narrow = entry(&p, Purpose::Publication);
    narrow.scope = scope(1);
    let mut for_who = entry(&p, Purpose::DisclosureToThirdParty);
    for_who.recipient = Some(who.clone());
    let wide = entry(&p, Purpose::Photo);
    for e in [narrow, for_who, wide] {
        if d.record_consent(&e).await.is_err() {
            r.ensure(false, "match: the setup records are accepted");
            return;
        }
    }
    let at = |purpose: Purpose, s: Option<u32>| {
        let mut q = query(&p, purpose);
        q.scope = s.and_then(scope);
        q
    };
    r.ensure(
        found(d, &at(Purpose::Publication, Some(1))).await.is_some(),
        "match: the same scope matches",
    );
    r.ensure(
        found(d, &at(Purpose::Publication, None)).await.is_none(),
        "match: a consent with scope does not match a question without",
    );
    r.ensure(
        found(d, &at(Purpose::Publication, Some(2))).await.is_none(),
        "match: another scope does not match",
    );
    r.ensure(
        found(d, &at(Purpose::Photo, Some(2))).await.is_some(),
        "match: a consent without scope matches every scope",
    );
    let mut to = query(&p, Purpose::DisclosureToThirdParty);
    to.recipient = Some(who.clone());
    r.ensure(
        found(d, &to).await.is_some(),
        "match: the same recipient matches",
    );
    to.recipient = Some(p.clone());
    r.ensure(
        found(d, &to).await.is_none(),
        "match: another recipient does not match",
    );
    r.ensure(
        found(d, &query(&p, Purpose::DisclosureToThirdParty))
            .await
            .is_none(),
        "match: a consent for a recipient does not match a question without",
    );
    let mut wide_to = query(&p, Purpose::Photo);
    wide_to.recipient = Some(who);
    r.ensure(
        found(d, &wide_to).await.is_some(),
        "match: a consent without recipient matches every recipient",
    );
}

async fn profile_rules(fx: &dyn ConformanceFixture, d: &dyn PersonDirectory, r: &mut Report) {
    let owner = person(fx, "Linked Owner").await;
    let one = fx
        .add_confirmed(PersonArt::Profile, "Zzconf Linked One", None)
        .await;
    let two = fx
        .add_confirmed(PersonArt::Profile, "Zzconf Linked Two", None)
        .await;
    let (Some(owner), Some(one), Some(two)) = (owner, one, two) else {
        return;
    };
    if !(fx.describe(&one, Some(&owner)).await && fx.describe(&two, Some(&owner)).await) {
        r.skip("match: fixture cannot link profiles");
        return;
    }
    let (Ok(_), Ok(_)) = (
        d.record_consent(&entry(&owner, Purpose::Newsletter)).await,
        d.record_consent(&entry(&one, Purpose::Photo)).await,
    ) else {
        return;
    };
    let news = Purpose::Newsletter;
    r.ensure(
        found(d, &query(&one, news.clone())).await.is_some(),
        "match: a person's consent covers the profile that describes them",
    );
    r.ensure(
        found(d, &query(&two, news.clone())).await.is_some(),
        "match: a person's consent covers all their profiles",
    );
    r.ensure(
        found(d, &query(&one, Purpose::Photo)).await.is_some(),
        "match: a profile's own consent covers the profile",
    );
    r.ensure(
        found(d, &query(&owner, Purpose::Photo)).await.is_none(),
        "match: a profile's consent never covers the person",
    );
    r.ensure(
        found(d, &query(&two, Purpose::Photo)).await.is_none(),
        "match: a profile's consent never covers another profile",
    );
    if fx.describe(&one, None).await {
        r.ensure(
            found(d, &query(&one, news)).await.is_none(),
            "match: a cut link no longer covers the profile",
        );
    }
}

async fn merge_rules(fx: &dyn ConformanceFixture, d: &dyn PersonDirectory, r: &mut Report) {
    let old = fx
        .add_confirmed(PersonArt::Profile, "Zzconf Consent Old", None)
        .await;
    let new = person(fx, "Consent New").await;
    let (Some(old), Some(new)) = (old, new) else {
        return;
    };
    let Ok(c) = d.record_consent(&entry(&old, Purpose::Newsletter)).await else {
        return;
    };
    if !fx.merge(&old, &new).await {
        r.skip("match: fixture cannot merge");
        return;
    }
    let news = Purpose::Newsletter;
    r.ensure(
        found(d, &query(&new, news.clone())).await == Some(c.id),
        "match: consents move with a merge (the target is covered)",
    );
    r.ensure(
        found(d, &query(&old, news)).await == Some(c.id),
        "match: a merged reference counts as its target",
    );
    let listed: Vec<ObjectId> = d
        .consents_of(&new)
        .await
        .unwrap_or_default()
        .iter()
        .map(|c| c.id)
        .collect();
    r.ensure(
        listed == vec![c.id],
        "consents_of: the target lists the consents of merged references",
    );
}
