//! State and mutators of `FakeDirectory`.
//!
//! Pattern: Fake (in-memory state behind one lock; mutators feed the change log).

use super::store::{Entry, Status};
use crate::person_directory_v1::{ChangeKind, DirectoryChange, ObjectId, PersonRef, Utc};
use std::sync::{Mutex, MutexGuard, PoisonError};

/// Mutable state, behind one lock.
#[derive(Default)]
pub(crate) struct Inner {
    pub(crate) entries: Vec<Entry>,
    pub(crate) log: Vec<DirectoryChange>,
    /// Batches and whether they are still open.
    pub(crate) batches: Vec<(ObjectId, bool)>,
    pub(crate) tick: u64,
}

/// In-memory provider; `asker` is a property of the provider (the "I" card).
#[derive(Default)]
pub struct FakeDirectory {
    /// A foreign address book: reading works, `person.propose` is `Forbidden`.
    pub(crate) foreign: bool,
    pub(crate) asker: Option<PersonRef>,
    pub(crate) inner: Mutex<Inner>,
}

impl FakeDirectory {
    /// An empty directory with no asker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A foreign address book: reading works, writing is `Forbidden`.
    #[must_use]
    pub fn foreign_book(mut self) -> Self {
        self.foreign = true;
        self
    }

    /// Set who asks (`None` = not signed in).
    #[must_use]
    pub fn with_asker(mut self, asker: Option<PersonRef>) -> Self {
        self.asker = asker;
        self
    }

    pub(crate) fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Add a row and log `Created`; its creation time comes from the fake clock.
    #[must_use]
    pub fn with_entry(self, entry: Entry) -> Self {
        self.add(entry);
        self
    }

    /// Add a row and log `Created`.
    pub fn add(&self, mut entry: Entry) {
        let mut g = self.lock();
        entry.created = g.now();
        let change = (entry.person.clone(), ChangeKind::Created);
        g.entries.push(entry);
        g.record(change.0, change.1);
    }

    /// Delete (tombstone) a row.
    pub fn delete(&self, person: &PersonRef) -> bool {
        let mut g = self.lock();
        let at = g.now();
        g.set_status(person, Status::Deleted(at), ChangeKind::Deleted)
    }

    /// Withdraw the sharing of a row.
    pub fn unshare(&self, person: &PersonRef) -> bool {
        self.lock()
            .set_status(person, Status::NotShared, ChangeKind::NotShared)
    }

    /// Bring a deleted or unshared row back; logs `Restored`.
    pub fn restore(&self, person: &PersonRef) -> bool {
        self.lock()
            .set_status(person, Status::Active, ChangeKind::Restored)
    }

    /// Merge `from` into `into`: `from` stays valid and resolves to `into`.
    pub fn merge(&self, from: &PersonRef, into: &PersonRef) -> bool {
        let mut g = self.lock();
        let Some(e) = g.find_mut(from) else {
            return false;
        };
        e.merged_into = Some(into.clone());
        e.rev += 1;
        g.record(from.clone(), ChangeKind::Merged { into: into.clone() });
        true
    }

    /// Change the display name of a row; logs `Changed`.
    pub fn rename(&self, person: &PersonRef, display: &str) -> bool {
        let mut g = self.lock();
        let Some(e) = g.find_mut(person) else {
            return false;
        };
        display.clone_into(&mut e.display);
        e.sort_key = display.to_lowercase();
        e.rev += 1;
        g.record(person.clone(), ChangeKind::Changed);
        true
    }

    /// Close an open batch; later `propose` into it is `BatchClosed`.
    pub fn close_batch(&self, batch: ObjectId) {
        for (id, open) in &mut self.lock().batches {
            if *id == batch {
                *open = false;
            }
        }
    }
}

impl Inner {
    pub(crate) fn now(&mut self) -> Utc {
        self.tick += 1;
        Utc::from_millis(1_700_000_000_000 + i64::try_from(self.tick).unwrap_or(0) * 1000)
    }

    pub(crate) fn record(&mut self, person: PersonRef, kind: ChangeKind) {
        let at = self.now();
        self.log.push(DirectoryChange { person, kind, at });
    }

    pub(crate) fn find_mut(&mut self, person: &PersonRef) -> Option<&mut Entry> {
        self.entries.iter_mut().find(|e| e.person.id == person.id)
    }

    fn set_status(&mut self, person: &PersonRef, status: Status, kind: ChangeKind) -> bool {
        let Some(e) = self.find_mut(person) else {
            return false;
        };
        e.status = status;
        e.rev += 1;
        self.record(person.clone(), kind);
        true
    }
}
