//! More rules that need only the trait: reference forms, UUIDv7 ids, `tel` comparison, cancellation.
//!
//! Pattern: Template method (rules as plain checks collected in a `Report`).

use super::{Report, Seeds, entry};
use crate::person_directory_v1::{ObjectId, PersonArt, PersonDirectory, PersonRef, Resolved};
use fs_types::{BasicValue, ContactAddress};
use std::future::Future;
use std::pin::pin;
use std::task::{Context, Waker};

/// `urn:uuid:` followed by a UUIDv7 (version 7, variant 10).
fn is_uuid7_urn(person: &PersonRef) -> bool {
    let text = person.id.canonical();
    let Some(uuid) = text.strip_prefix("urn:uuid:") else {
        return false;
    };
    let b: Vec<char> = uuid.chars().collect();
    b.len() == 36 && b[14] == '7' && matches!(b[19], '8' | '9' | 'a' | 'b' | 'A' | 'B')
}

pub(super) async fn rules(dir: &dyn PersonDirectory, s: &Seeds, r: &mut Report) {
    r.ensure(
        is_uuid7_urn(&s.a) && is_uuid7_urn(&s.b) && is_uuid7_urn(&s.c),
        "propose: new ids are UUIDv7",
    );
    r.ensure(
        s.a.id != s.b.id && s.b.id != s.c.id,
        "propose: new ids differ",
    );
    form_rules(dir, s, r).await;
    tel_rules(dir, r).await;
    cancel_rules(dir, r).await;
}

async fn form_rules(dir: &dyn PersonDirectory, s: &Seeds, r: &mut Report) {
    let Ok(Resolved::Found(truth)) = dir.resolve(&s.a).await else {
        return;
    };
    // A reference with `abzug` resolves like one without.
    let with_abzug = PersonRef {
        abzug: Some(ObjectId([1; 16])),
        ..s.a.clone()
    };
    let found = dir.resolve(&with_abzug).await;
    r.ensure(
        matches!(&found, Ok(Resolved::Found(v)) if v.person.id == s.a.id && v.person.abzug.is_none()),
        "resolve: a reference with abzug resolves like one without, abzug None in the answer",
    );
    // A different art in the input: still `Found`, with the art of the target.
    let other = if truth.person.art == PersonArt::Group {
        PersonArt::Person
    } else {
        PersonArt::Group
    };
    let wrong = PersonRef {
        art: other,
        ..s.a.clone()
    };
    r.ensure(
        matches!(dir.resolve(&wrong).await, Ok(Resolved::Found(v)) if v.person.art == truth.person.art),
        "resolve: a wrong art in the input is Found with the art of the target",
    );
}

async fn tel_rules(dir: &dyn PersonDirectory, r: &mut Report) {
    let Ok(spaced) = dir
        .propose(&entry("Zzconf Tel", Some("tel:+49-30-999000111")), None)
        .await
    else {
        r.ensure(false, "propose: accepts a tel address");
        return;
    };
    let plain = ContactAddress::parse("tel:+4930999000111").ok();
    let hits = match plain {
        Some(a) => dir.find_by_address(&a).await.unwrap_or_default(),
        None => Vec::new(),
    };
    r.ensure(
        hits.iter().any(|v| v.person.id == spaced.id),
        "find_by_address: tel numbers compare by their digits (E.164)",
    );
}

async fn cancel_rules(dir: &dyn PersonDirectory, r: &mut Report) {
    let address = "mailto:dropped.zzconf@example.org";
    let wanted = ContactAddress::parse(address).ok();
    let e = entry("Zzconf Dropped", Some(address));
    {
        // Polled once, then dropped: a time limit in the consumer.
        let mut cx = Context::from_waker(Waker::noop());
        let mut fut = pin!(dir.propose(&e, None));
        let _ = fut.as_mut().poll(&mut cx);
    }
    let Some(wanted) = wanted else { return };
    let after_drop = dir.find_by_address(&wanted).await.map_or(9, |v| v.len());
    r.ensure(
        after_drop <= 1,
        "propose: a dropped future leaves nothing half done",
    );
    let again = dir.propose(&e, None).await;
    let hits = dir.find_by_address(&wanted).await.map_or(0, |v| v.len());
    r.ensure(
        again.is_ok() && hits == 1,
        "propose: after a dropped future the same proposal is whole, once",
    );
}
