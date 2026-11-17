//! B5: writing is compare-and-set or it does not happen. A source without `update_checked` locks
//! editing with the reason `people/no-checked-update` and writes nothing (no fall back to a plain
//! `update`); two windows saving from the same revision: exactly one wins, the other gets a conflict.
//!
//! Pattern: Integration test with a counting double of `People` (delegates reads, counts writes);
//! the second test runs two real threads over the real core. Own oracle: the write counter.
//! Mutant: the old fall back to `update` on `Unsupported` in `PeopleAdapter::save`: the counter is 1
//! and the first test is red.

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use common::{World, person};
use fs_components_workbenches::ErrorKind;
use fs_components_workbenches::people::{PeopleSource, PersonDraft};
use fs_interfaces::Capabilities;
use fs_objects::{ObjectId, StoredObject};
use fs_people_app::{NoRoles, PeopleAdapter};
use fs_people_core::{Card, ListFilter, People, PeopleRepo, Result};

/// Reads go to the real core; `update` counts; `update_checked` is the default (`Unsupported`).
struct NoCheckedUpdate {
    inner: Arc<PeopleRepo>,
    writes: AtomicUsize,
}

#[async_trait]
impl People for NoCheckedUpdate {
    fn capabilities(&self) -> Capabilities {
        self.inner.capabilities()
    }
    async fn get(&self, id: ObjectId) -> Result<Option<StoredObject>> {
        self.inner.get(id).await
    }
    async fn list(&self, filter: &ListFilter) -> Result<Vec<StoredObject>> {
        People::list(&*self.inner, filter).await
    }
    async fn update(&self, id: ObjectId, card: Card) -> Result<StoredObject> {
        self.writes.fetch_add(1, Ordering::SeqCst);
        People::update(&*self.inner, id, card).await
    }
}

fn change_mail(draft: &mut PersonDraft, from: &str, to: &str) {
    let field = draft
        .groups
        .iter_mut()
        .flat_map(|g| g.fields.iter_mut())
        .find(|f| f.value == from)
        .expect("the e-mail is in the form");
    field.value = to.into();
}

#[test]
fn a_source_without_a_checked_update_locks_editing_and_writes_nothing() {
    let w = World::in_memory();
    let key = w.add(person("Amara", "amara@x.org"));
    let double = Arc::new(NoCheckedUpdate {
        inner: w.repo.clone(),
        writes: AtomicUsize::new(0),
    });
    let adapter = PeopleAdapter::new(
        double.clone(),
        Arc::new(NoRoles),
        w.runtime.handle().clone(),
    );
    let mut draft = adapter.draft(&key).unwrap();
    change_mail(&mut draft, "amara@x.org", "changed@x.org");
    let refused = adapter.save(&key, &draft).unwrap_err();
    assert_eq!(refused.kind, ErrorKind::Unsupported, "{refused:?}");
    assert!(
        refused.message.contains("people/no-checked-update"),
        "{refused:?}"
    );
    assert_eq!(
        double.writes.load(Ordering::SeqCst),
        0,
        "nothing was written"
    );
    assert_eq!(
        w.card(&key).emails.values().next().unwrap().address,
        "amara@x.org"
    );
}

#[test]
fn two_windows_saving_from_one_revision_exactly_one_wins() {
    for round in 0..20 {
        let w = Arc::new(World::in_memory());
        let key = w.add(person("Amara", "amara@x.org"));
        let (mut a, mut b) = (
            w.adapter.draft(&key).unwrap(),
            w.adapter.draft(&key).unwrap(),
        );
        change_mail(&mut a, "amara@x.org", "a@x.org");
        change_mail(&mut b, "amara@x.org", "b@x.org");
        let results: Vec<_> = std::thread::scope(|s| {
            let ha = s.spawn(|| w.adapter.save(&key, &a));
            let hb = s.spawn(|| w.adapter.save(&key, &b));
            vec![ha.join().unwrap(), hb.join().unwrap()]
        });
        let won = results.iter().filter(|r| r.is_ok()).count();
        let lost: Vec<_> = results.iter().filter_map(|r| r.as_ref().err()).collect();
        assert_eq!(won, 1, "round {round}: {results:?}");
        assert_eq!(lost.len(), 1);
        assert_eq!(lost[0].kind, ErrorKind::Conflict, "round {round}: {lost:?}");
    }
}
