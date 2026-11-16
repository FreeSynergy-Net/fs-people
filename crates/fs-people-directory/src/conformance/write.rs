//! Conformance rules around `propose`, batches and `changes`.
//!
//! Pattern: Template method (rules as plain checks collected in a `Report`).

use super::{Report, Seeds, address, entry};
use crate::person_directory_v1::{ChangeKind, Error, ObjectId, PersonDirectory};
use fs_types::{BasicValue, Reference};

fn is_invalid(e: &Error) -> bool {
    matches!(e, Error::Invalid(_))
}

pub(super) async fn seed(dir: &dyn PersonDirectory, r: &mut Report) -> Option<Seeds> {
    let a_text = "mailto:Conf.A@Example.ORG";
    let a = dir
        .propose(&entry("Zzconf Alpha", Some(a_text)), None)
        .await;
    let b = dir
        .propose(
            &entry("Zzconf Beta", Some("mailto:conf.b@example.org")),
            None,
        )
        .await;
    let c = dir.propose(&entry("Zzconf Gamma", None), None).await;
    let (Ok(a), Ok(b), Ok(c), Some(a_address)) = (a, b, c, address(a_text)) else {
        r.ensure(false, "propose: three valid entries are accepted");
        return None;
    };
    Some(Seeds { a, a_address, b, c })
}

pub(super) async fn propose_rules(dir: &dyn PersonDirectory, s: &Seeds, r: &mut Report) {
    // Idempotent: the same address (domain without case) returns the existing reference.
    let again = dir
        .propose(
            &entry("Other hint", Some("mailto:Conf.A@example.org")),
            None,
        )
        .await;
    r.ensure(
        again.as_ref().is_ok_and(|p| p.id == s.a.id),
        "propose: a known address returns the existing reference",
    );
    let empty = dir.propose(&entry("  ", None), None).await;
    r.is_err(
        &empty,
        "propose: no hint and no address is Invalid",
        is_invalid,
    );
    let closed = dir
        .propose(&entry("Zzconf Delta", None), Some(ObjectId([0xEE; 16])))
        .await;
    r.is_err(
        &closed,
        "propose: a never opened batch is BatchClosed",
        |e| matches!(e, Error::BatchClosed(_)),
    );
    let Ok(origin) = Reference::parse("urn:uuid:00000000-0000-7000-8000-00000000fffe") else {
        return;
    };
    let batch = dir.open_batch(&origin).await;
    r.ensure(batch.is_ok(), "open_batch: opens a batch");
    if let Ok(id) = batch {
        let (b6, b8) = (id.0[6], id.0[8]);
        r.ensure(
            b6 >> 4 == 7 && b8 >> 6 == 0b10,
            "open_batch: the batch id is a UUIDv7",
        );
        let inside = dir.propose(&entry("Zzconf Epsilon", None), Some(id)).await;
        r.ensure(inside.is_ok(), "propose: accepted into an open batch");
    }
}

pub(super) async fn changes_rules(dir: &dyn PersonDirectory, r: &mut Report) {
    r.is_err(
        &dir.changes(None, 0).await,
        "changes: size 0 is Invalid",
        is_invalid,
    );
    let Ok(now) = dir.changes(None, 5).await else {
        r.ensure(false, "changes: since None answers");
        return;
    };
    r.ensure(now.items.is_empty(), "changes: since None starts from now");
    r.ensure(
        now.next.is_some(),
        "changes: next is Some even on an empty page",
    );
    let Some(cursor) = now.next else { return };
    let created = dir.propose(&entry("Zzconf Zeta", None), None).await;
    r.ensure(created.is_ok(), "propose: a new entry for the change feed");
    let Ok(page) = dir.changes(Some(&cursor), 1).await else {
        r.ensure(false, "changes: continues from a cursor");
        return;
    };
    r.ensure(page.items.len() == 1, "changes: size limits the page");
    r.ensure(
        page.items
            .first()
            .is_some_and(|c| c.kind == ChangeKind::Created),
        "changes: a proposal is Created",
    );
    r.ensure(
        created
            .as_ref()
            .is_ok_and(|p| page.items.first().is_some_and(|c| c.person.id == p.id)),
        "changes: the change names the proposed reference",
    );
    let Some(next) = page.next else {
        r.ensure(false, "changes: next is Some after a full page");
        return;
    };
    let rest = dir.changes(Some(&next), 100).await;
    r.ensure(
        rest.as_ref().is_ok_and(|p| p.next.is_some()),
        "changes: next stays Some at the end",
    );
}
