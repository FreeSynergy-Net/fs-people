//! Conformance rules for the reading methods: `resolve`, `suggest`, `find_by_address`, `excerpt`.
//!
//! Pattern: Template method (rules as plain checks collected in a `Report`).

use super::{Report, Seeds, address};
use crate::person_directory_v1::{
    EntryState, Error, ExcerptField, Matched, NoExcerptReason, PersonArt, PersonDirectory,
    PersonRef, Resolved, Suggest,
};
use fs_types::{BasicValue, Reference};

fn query(text: &str, limit: u16, proposed: bool) -> Suggest {
    Suggest {
        text: text.to_owned(),
        arten: Vec::new(),
        limit,
        include_proposed: proposed,
    }
}

fn unknown() -> Option<PersonRef> {
    Reference::parse("urn:uuid:00000000-0000-7000-8000-0000deadbeef")
        .ok()
        .map(|id| PersonRef {
            id,
            art: PersonArt::Person,
            abzug: None,
        })
}

pub(super) async fn resolve_rules(dir: &dyn PersonDirectory, seeds: &Seeds, r: &mut Report) {
    let Some(nobody) = unknown() else { return };
    let one = dir.resolve(&seeds.a).await;
    r.ensure(
        matches!(&one, Ok(Resolved::Found(v)) if v.person.abzug.is_none() && v.state == EntryState::Proposed),
        "resolve: Found, abzug is None, state Proposed",
    );
    r.ensure(
        matches!(dir.resolve(&nobody).await, Ok(Resolved::Unknown)),
        "resolve: an unknown reference is Unknown, not an error",
    );
    let input = [
        seeds.a.clone(),
        nobody.clone(),
        seeds.a.clone(),
        seeds.b.clone(),
    ];
    let many = dir.resolve_many(&input).await.unwrap_or_default();
    r.ensure(
        many.len() == input.len(),
        "resolve_many: same length as the input",
    );
    r.ensure(
        matches!(many.as_slice(), [Resolved::Found(p), Resolved::Unknown, Resolved::Found(q), Resolved::Found(t)]
            if p.person.id == seeds.a.id && q.person.id == seeds.a.id && t.person.id == seeds.b.id),
        "resolve_many: input order, duplicates answered twice",
    );
    r.ensure(
        dir.resolve_many(&[]).await.is_ok_and(|v| v.is_empty()),
        "resolve_many: empty input, empty answer",
    );
}

pub(super) async fn suggest_rules(dir: &dyn PersonDirectory, s: &Seeds, r: &mut Report) {
    let bad = |e: &Error| matches!(e, Error::Invalid(_));
    r.is_err(
        &dir.suggest(&query("", 0, true)).await,
        "suggest: limit 0 is Invalid",
        bad,
    );
    r.is_err(
        &dir.suggest(&query("", 51, true)).await,
        "suggest: limit 51 is Invalid",
        bad,
    );
    r.ensure(
        dir.suggest(&query("", 50, true)).await.is_ok(),
        "suggest: limit 50 is allowed",
    );
    let without = dir
        .suggest(&query("Zzconf", 50, false))
        .await
        .unwrap_or_default();
    r.ensure(
        without
            .iter()
            .all(|c| c.view.state == EntryState::Confirmed),
        "suggest: Proposed entries only with include_proposed",
    );
    let with = dir
        .suggest(&query("Zzconf", 50, true))
        .await
        .unwrap_or_default();
    let has = |p: &PersonRef| with.iter().any(|c| c.view.person.id == p.id);
    r.ensure(
        has(&s.a) && has(&s.b) && has(&s.c),
        "suggest: finds the seeded entries by name part",
    );
    let exact = dir
        .suggest(&query("Zzconf Beta", 50, true))
        .await
        .unwrap_or_default();
    r.ensure(
        exact.first().is_some_and(|c| c.view.person.id == s.b.id),
        "suggest: order is the rank, the exact name first",
    );
    let one = dir
        .suggest(&query("Zzconf", 1, true))
        .await
        .unwrap_or_default();
    r.ensure(one.len() == 1, "suggest: limit caps the list");
    // The seeds have the art of `s.a`: asking for that art finds it, asking for another art does not.
    let other = if s.a.art == PersonArt::Group {
        PersonArt::Person
    } else {
        PersonArt::Group
    };
    let mut same = query("Zzconf Alpha", 50, true);
    same.arten = vec![s.a.art];
    let hits = dir.suggest(&same).await.unwrap_or_default();
    r.ensure(
        hits.iter().any(|c| c.view.person.id == s.a.id),
        "suggest: arten of the entry finds it",
    );
    let mut different = query("Zzconf Alpha", 50, true);
    different.arten = vec![other];
    let hits = dir.suggest(&different).await.unwrap_or_default();
    r.ensure(
        hits.iter()
            .all(|c| c.view.person.id != s.a.id && c.view.person.art == other),
        "suggest: arten filters the others out",
    );
    let by_address = dir
        .suggest(&query("conf.a@example", 50, true))
        .await
        .unwrap_or_default();
    r.ensure(
        by_address
            .iter()
            .any(|c| c.view.person.id == s.a.id && c.matched == Matched::Address),
        "suggest: an address part matches as Address",
    );
}

pub(super) async fn find_rules(dir: &dyn PersonDirectory, s: &Seeds, r: &mut Report) {
    let found = dir.find_by_address(&s.a_address).await.unwrap_or_default();
    r.ensure(
        found.iter().any(|v| v.person.id == s.a.id),
        "find_by_address: finds the entry (domain without case)",
    );
    let lower = address("mailto:Conf.A@example.org");
    let same = match lower {
        Some(a) => dir.find_by_address(&a).await.unwrap_or_default(),
        None => Vec::new(),
    };
    r.ensure(
        same.iter().any(|v| v.person.id == s.a.id),
        "find_by_address: the domain compares without case",
    );
    let none = match address("mailto:nobody-zzconf@example.org") {
        Some(a) => dir.find_by_address(&a).await,
        None => Ok(vec![]),
    };
    r.ensure(
        none.is_ok_and(|v| v.is_empty()),
        "find_by_address: no match is an empty list",
    );
    let states: Vec<bool> = found
        .iter()
        .map(|v| v.state == EntryState::Proposed)
        .collect();
    r.ensure(
        states.windows(2).all(|w| w[0] <= w[1]),
        "find_by_address: Confirmed before Proposed",
    );
}

pub(super) async fn excerpt_rules(dir: &dyn PersonDirectory, s: &Seeds, r: &mut Report) {
    let Some(nobody) = unknown() else { return };
    let empty = dir.excerpt(&s.a, &[]).await;
    r.is_err(&empty, "excerpt: an empty field list is Invalid", |e| {
        matches!(e, Error::Invalid(_))
    });
    let unknown_ref = dir.excerpt(&nobody, &[ExcerptField::FullName]).await;
    r.is_err(
        &unknown_ref,
        "excerpt: an unknown reference is NoExcerpt Unknown",
        |e| {
            matches!(
                e,
                Error::NoExcerpt {
                    reason: NoExcerptReason::Unknown,
                    ..
                }
            )
        },
    );
    let fields = [
        ExcerptField::FullName,
        ExcerptField::Phone,
        ExcerptField::Phone,
    ];
    match dir.excerpt(&s.a, &fields).await {
        Ok(x) => {
            r.ensure(x.person.abzug.is_none(), "excerpt: abzug is None");
            r.ensure(
                x.values.iter().all(|v| fields.contains(&v.field())),
                "excerpt: only the asked fields",
            );
            r.ensure(
                x.values.len() + x.missing.len() == 2,
                "excerpt: every asked field is a value or in missing, once",
            );
        }
        Err(_) => r.ensure(false, "excerpt: answers for a known entry"),
    }
}
