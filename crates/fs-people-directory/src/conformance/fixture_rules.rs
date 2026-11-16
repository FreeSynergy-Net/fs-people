//! State rules: `Deleted`, `NotShared`, `Restored`, `Merged`, `Changed`, ordering and arten with real data, Forbidden.
//!
//! Pattern: Template method (rules as plain checks; the states come from the `ConformanceFixture`).

use super::{ConformanceFixture, Report, Seeds};
use crate::person_directory_v1::{
    ChangeKind, Error, ExcerptField, NoExcerptReason, PersonArt, PersonDirectory, PersonRef, Place,
    Resolved, Suggest,
};
use fs_types::ContactAddress;

fn suggest(text: &str, arten: Vec<PersonArt>) -> Suggest {
    Suggest {
        text: text.to_owned(),
        arten,
        limit: 50,
        include_proposed: true,
    }
}

async fn cursor(dir: &dyn PersonDirectory) -> Option<Place> {
    dir.changes(None, 1).await.ok()?.next
}

/// The kinds of change of `person` since `from`.
async fn kinds(dir: &dyn PersonDirectory, from: &Place, person: &PersonRef) -> Vec<ChangeKind> {
    let page = dir.changes(Some(from), 200).await;
    page.map(|p| {
        p.items
            .into_iter()
            .filter(|c| c.person.id == person.id)
            .map(|c| c.kind)
            .collect()
    })
    .unwrap_or_default()
}

pub(super) async fn rules(fx: &dyn ConformanceFixture, s: &Seeds, r: &mut Report) {
    let Some(from) = cursor(fx.directory()).await else {
        r.ensure(false, "changes: a cursor to start from");
        return;
    };
    gone_rules(fx, "Deleted", &from, r).await;
    gone_rules(fx, "NotShared", &from, r).await;
    merge_rules(fx, &from, r).await;
    order_rules(fx, s, r).await;
    arten_rules(fx, r).await;
    forbidden_rules(fx, r).await;
}

/// `Deleted` or `NotShared`, then `Restored`.
async fn gone_rules(fx: &dyn ConformanceFixture, which: &str, from: &Place, r: &mut Report) {
    let dir = fx.directory();
    let name = format!("Zzconf {which} Person");
    let address = format!("mailto:zzconf.{}@example.org", which.to_lowercase());
    let Some(p) = fx
        .add_confirmed(PersonArt::Person, &name, Some(&address))
        .await
    else {
        r.skip(&format!("{which}: fixture cannot add_confirmed"));
        return;
    };
    let done = if which == "Deleted" {
        fx.delete(&p).await
    } else {
        fx.unshare(&p).await
    };
    if !done {
        r.skip(&format!("{which}: fixture cannot set the state"));
        return;
    }
    let resolved = dir.resolve(&p).await;
    let (state_ok, reason) = if which == "Deleted" {
        (
            matches!(resolved, Ok(Resolved::Deleted { .. })),
            NoExcerptReason::Deleted,
        )
    } else {
        (
            matches!(resolved, Ok(Resolved::NotShared)),
            NoExcerptReason::NotShared,
        )
    };
    r.ensure(
        state_ok,
        &format!("resolve: {which} is {which}, not Unknown"),
    );
    let excerpt = dir.excerpt(&p, &[ExcerptField::FullName]).await;
    r.is_err(
        &excerpt,
        &format!("excerpt: {which} is NoExcerpt with the reason {which}"),
        match reason {
            NoExcerptReason::Deleted => |e| {
                matches!(
                    e,
                    Error::NoExcerpt {
                        reason: NoExcerptReason::Deleted,
                        ..
                    }
                )
            },
            _ => |e| {
                matches!(
                    e,
                    Error::NoExcerpt {
                        reason: NoExcerptReason::NotShared,
                        ..
                    }
                )
            },
        },
    );
    let shown = dir
        .suggest(&suggest(&name, vec![]))
        .await
        .unwrap_or_default();
    r.ensure(
        shown.iter().all(|c| c.view.person.id != p.id),
        &format!("suggest: {which} never appears"),
    );
    let found = match ContactAddress::parse(&address) {
        Ok(a) => dir.find_by_address(&a).await.unwrap_or_default(),
        Err(_) => Vec::new(),
    };
    r.ensure(
        found.iter().all(|v| v.person.id != p.id),
        &format!("find_by_address: {which} is missing"),
    );
    let wanted = if which == "Deleted" {
        ChangeKind::Deleted
    } else {
        ChangeKind::NotShared
    };
    r.ensure(
        kinds(dir, from, &p).await.contains(&wanted),
        &format!("changes: {which} is reported"),
    );
    if !fx.restore(&p).await {
        r.skip(&format!("{which}: fixture cannot restore"));
        return;
    }
    r.ensure(
        matches!(dir.resolve(&p).await, Ok(Resolved::Found(_))),
        &format!("resolve: after restore from {which} the entry is Found"),
    );
    r.ensure(
        kinds(dir, from, &p).await.contains(&ChangeKind::Restored),
        &format!("changes: restore from {which} is Restored"),
    );
}

async fn merge_rules(fx: &dyn ConformanceFixture, from: &Place, r: &mut Report) {
    let dir = fx.directory();
    let old = fx
        .add_confirmed(PersonArt::Profile, "Zzconf Merge Old", None)
        .await;
    let new = fx
        .add_confirmed(PersonArt::Person, "Zzconf Merge New", None)
        .await;
    let (Some(old), Some(new)) = (old, new) else {
        r.skip("Merged: fixture cannot add_confirmed");
        return;
    };
    if !fx.merge(&old, &new).await {
        r.skip("Merged: fixture cannot merge");
        return;
    }
    r.ensure(
        kinds(dir, from, &old)
            .await
            .iter()
            .any(|k| matches!(k, ChangeKind::Merged { into } if into.id == new.id)),
        "changes: Merged names the target",
    );
    r.ensure(
        matches!(dir.resolve(&old).await, Ok(Resolved::Found(v)) if v.person.id == new.id),
        "resolve: after Merged the old reference is Found with person == into",
    );
    r.ensure(
        dir.excerpt(&old, &[ExcerptField::FullName]).await.is_ok(),
        "excerpt: after Merged the old reference gives the excerpt of the target",
    );
    if fx.rename(&new, "Zzconf Merge Renamed").await {
        r.ensure(
            kinds(dir, from, &new).await.contains(&ChangeKind::Changed),
            "changes: a new display name is Changed",
        );
    } else {
        r.skip("Changed: fixture cannot rename");
    }
}

/// `find_by_address`: Confirmed before Proposed, Person before Profile, with entries that really tie.
async fn order_rules(fx: &dyn ConformanceFixture, seeds: &Seeds, r: &mut Report) {
    let dir = fx.directory();
    let addr = seeds.a_address.canonical();
    let person = fx
        .add_confirmed(PersonArt::Person, "Zzconf Order Person", Some(&addr))
        .await;
    let profile = fx
        .add_confirmed(PersonArt::Profile, "Zzconf Order Profile", Some(&addr))
        .await;
    let (Some(person), Some(profile)) = (person, profile) else {
        r.skip("find_by_address order: fixture cannot add_confirmed");
        return;
    };
    let hits = dir
        .find_by_address(&seeds.a_address)
        .await
        .unwrap_or_default();
    let at = |who: &PersonRef| hits.iter().position(|v| v.person.id == who.id);
    r.ensure(
        matches!((at(&person), at(&profile), at(&seeds.a)),
            (Some(first), Some(second), Some(third)) if first < second && second < third),
        "find_by_address: Confirmed Person, Confirmed Profile, then Proposed",
    );
}

async fn arten_rules(fx: &dyn ConformanceFixture, r: &mut Report) {
    let dir = fx.directory();
    let Some(group) = fx
        .add_confirmed(PersonArt::Group, "Zzconf Team", None)
        .await
    else {
        r.skip("arten: fixture cannot add_confirmed");
        return;
    };
    let groups = dir
        .suggest(&suggest("Zzconf", vec![PersonArt::Group]))
        .await
        .unwrap_or_default();
    r.ensure(
        groups.iter().any(|c| c.view.person.id == group.id)
            && groups.iter().all(|c| c.view.person.art == PersonArt::Group),
        "suggest: arten Group finds the group and only groups",
    );
    let people = dir
        .suggest(&suggest("Zzconf", vec![PersonArt::Person]))
        .await
        .unwrap_or_default();
    r.ensure(
        people.iter().all(|c| c.view.person.id != group.id),
        "suggest: arten Person leaves the group out",
    );
}

async fn forbidden_rules(fx: &dyn ConformanceFixture, r: &mut Report) {
    let Some(foreign) = fx.foreign_book().await else {
        r.skip("Forbidden: fixture has no foreign address book");
        return;
    };
    let e = super::entry("Zzconf Foreign", None);
    let result = foreign.propose(&e, None).await;
    r.is_err(
        &result,
        "propose: a foreign address book is Forbidden",
        |e| matches!(e, Error::Forbidden { .. }),
    );
}
