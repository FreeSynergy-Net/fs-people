//! Rules that close gaps of the consent rules: the AI mark that sticks, a member without `consent.check`, a request
//! without a login, an organization, who records an erasure and the retention time.
//!
//! Pattern: Template method (rules as plain checks; states, views and the clock come from the `ConformanceFixture`).

use super::consent_rules::{entry, forbidden, person, query, withdrawal};
use super::{ConformanceFixture, Report, address, purposed};
use crate::person_directory_v1::{
    Date, ExcerptField, PersonArt, PersonDirectory, PersonRef, Purpose,
};
use fs_types::BasicValue;

const ALL: [&str; 3] = ["consent.read", "consent.check", "consent.write"];

pub(super) async fn rules(fx: &dyn ConformanceFixture, r: &mut Report) {
    ai_sticks(fx, r).await;
    let Some(admin) = fx.admin_view().await else {
        return;
    };
    human_without_check(fx, r).await;
    anonymous(fx, &*admin, r).await;
    organization(fx, &*admin, r).await;
    erased_by(fx, &*admin, r).await;
    retention(fx, &*admin, r).await;
}

/// The AI mark does not depend on the state of the entry; it follows the merge target (a profile merged into an AI)
/// and it stays with the AI itself (an AI merged into a person).
async fn ai_sticks(fx: &dyn ConformanceFixture, r: &mut Report) {
    for label in ["unshare", "delete", "merge", "merge into a person"] {
        let Some(ai) = fx.add_ai("Zzconf Sticky Robot").await else {
            r.skip("AI: fixture cannot add an AI");
            return;
        };
        let (asker, done) = match label {
            "unshare" => (Some(ai.clone()), fx.unshare(&ai).await),
            "delete" => (Some(ai.clone()), fx.delete(&ai).await),
            "merge into a person" => {
                let human = fx
                    .add_confirmed(PersonArt::Person, "Zzconf Sticky Human", None)
                    .await;
                let done = match &human {
                    Some(h) => fx.merge(&ai, h).await,
                    None => false,
                };
                (Some(ai.clone()), done)
            }
            _ => {
                let alias = fx
                    .add_confirmed(PersonArt::Profile, "Zzconf Sticky Alias", None)
                    .await;
                let done = match &alias {
                    Some(a) => fx.merge(a, &ai).await,
                    None => false,
                };
                (alias, done)
            }
        };
        let Some(asker) = asker.filter(|_| done) else {
            r.skip(&format!("AI: fixture cannot {label} an AI"));
            continue;
        };
        if let Some(bot) = fx.member_view(&asker, &ALL).await {
            still_ai(&*bot, &asker, label, r).await;
        }
    }
}

async fn still_ai(bot: &dyn PersonDirectory, p: &PersonRef, label: &str, r: &mut Report) {
    let Some(addr) = address("mailto:sticky.zzconf@example.org") else {
        return;
    };
    let rule = |what: &str| format!("AI: {what} stays Forbidden after {label} (an AI stays an AI)");
    r.is_err(
        &bot.find_by_address(&addr).await,
        &rule("find_by_address"),
        forbidden,
    );
    r.is_err(&bot.consents_of(p).await, &rule("consents_of"), forbidden);
    let q = query(p, Purpose::Newsletter);
    r.is_err(
        &bot.check_consent(&q).await,
        &rule("check_consent"),
        forbidden,
    );
    let rec = bot.record_consent(&entry(p, Purpose::Photo)).await;
    r.is_err(&rec, &rule("record_consent"), forbidden);
}

async fn human_without_check(fx: &dyn ConformanceFixture, r: &mut Report) {
    let Some(m) = person(fx, "No Check").await else {
        return;
    };
    let q = query(&m, Purpose::Newsletter);
    for (rights, who) in [
        (vec![], "no right"),
        (vec!["consent.read", "consent.write"], "read and write"),
    ] {
        if let Some(view) = fx.member_view(&m, &rights).await {
            r.is_err(
                &view.check_consent(&q).await,
                &format!(
                    "check_consent: a person with {who} but without consent.check is Forbidden"
                ),
                forbidden,
            );
        }
    }
}

async fn anonymous(fx: &dyn ConformanceFixture, admin: &dyn PersonDirectory, r: &mut Report) {
    let Some(anon) = fx.anonymous_view().await else {
        r.skip("consent: fixture has no view without a login");
        return;
    };
    let Some(p) = person(fx, "Anon").await else {
        return;
    };
    let Ok(c) = admin.record_consent(&entry(&p, Purpose::Newsletter)).await else {
        return;
    };
    // The anonymous view holds consent.write, so Forbidden can only come from the missing login.
    r.is_err(
        &anon.record_consent(&entry(&p, Purpose::Photo)).await,
        "record_consent: without a login is Forbidden",
        forbidden,
    );
    r.is_err(
        &anon.withdraw_consent(c.id, &withdrawal()).await,
        "withdraw_consent: without a login is Forbidden",
        forbidden,
    );
}

async fn organization(fx: &dyn ConformanceFixture, admin: &dyn PersonDirectory, r: &mut Report) {
    let org = fx
        .add_confirmed(PersonArt::Organization, "Zzconf Org", None)
        .await;
    let Some(org) = org else { return };
    let ask = purposed(&[ExcerptField::FullName], Purpose::Photo);
    r.ensure(
        admin
            .excerpt(&org, &ask)
            .await
            .is_ok_and(|x| x.consent.is_none() && !x.values.is_empty()),
        "excerpt: a purpose with consent duty on an organization needs no consent",
    );
}

async fn erased_by(fx: &dyn ConformanceFixture, admin: &dyn PersonDirectory, r: &mut Report) {
    let (Some(gone), Some(by)) = (person(fx, "Gone").await, person(fx, "Eraser").await) else {
        return;
    };
    if admin
        .record_consent(&entry(&gone, Purpose::Newsletter))
        .await
        .is_err()
        || !fx.delete_by(&gone, &by).await
    {
        r.skip("consent erasure: fixture cannot delete as someone");
        return;
    }
    let list = admin.consents_of(&gone).await.unwrap_or_default();
    r.ensure(
        list.first()
            .and_then(|c| c.withdrawn.as_ref())
            .is_some_and(|w| w.recorded_by.id == by.id),
        "delete: the Erasure withdrawal is recorded by the one who deletes",
    );
}

/// `retain_until` counts from the withdrawal; after it the record is really gone.
async fn retention(fx: &dyn ConformanceFixture, admin: &dyn PersonDirectory, r: &mut Report) {
    let Some(p) = person(fx, "Retain").await else {
        return;
    };
    let Ok(c) = admin.record_consent(&entry(&p, Purpose::Newsletter)).await else {
        return;
    };
    let Ok(w) = admin.withdraw_consent(c.id, &withdrawal()).await else {
        return;
    };
    // The withdrawal of `withdrawal()` is on 2026-11-16: three years from the end of 2026.
    let (Ok(end), Ok(after)) = (Date::parse("2029-12-31"), Date::parse("2030-01-01")) else {
        return;
    };
    r.ensure(
        w.retain_until == Some(end),
        "withdraw_consent: retain_until is the end of the third year after the withdrawal, not after the record",
    );
    let kept = |d: Vec<_>| {
        d.iter()
            .any(|x: &crate::person_directory_v1::Consent| x.id == c.id)
    };
    if !fx.purge_expired(end).await {
        r.skip("retention: fixture cannot purge expired records");
        return;
    }
    r.ensure(
        admin.consents_of(&p).await.is_ok_and(kept),
        "retention: a record stays until the end of retain_until",
    );
    fx.purge_expired(after).await;
    r.ensure(
        admin.consents_of(&p).await.is_ok_and(|l| !kept(l)),
        "retention: after retain_until the record is deleted for good",
    );
}
