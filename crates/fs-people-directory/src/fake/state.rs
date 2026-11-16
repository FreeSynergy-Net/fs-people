//! State and mutators of `FakeDirectory`.
//!
//! Pattern: Fake (in-memory state behind one lock; mutators feed the change log).

use super::Defect;
use super::store::{Entry, Status};
use crate::person_directory_v1::{ChangeKind, Consent, DirectoryChange, ObjectId, PersonRef, Utc};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// Which `consent.*` rights the asker holds (the fake has no fs-access; the asker's own records never need one).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rights {
    /// `consent.read`: read the records of others.
    pub read: bool,
    /// `consent.check`: ask `check_consent` (a program account's right).
    pub check: bool,
    /// `consent.write`: record and withdraw for others.
    pub write: bool,
}

impl Rights {
    /// All three rights (the default of the fake: an administrator).
    #[must_use]
    pub fn all() -> Self {
        Self {
            read: true,
            check: true,
            write: true,
        }
    }

    /// No right: an ordinary member, who may only handle their own records.
    #[must_use]
    pub fn none() -> Self {
        Self {
            read: false,
            check: false,
            write: false,
        }
    }
}

impl Default for Rights {
    fn default() -> Self {
        Self::all()
    }
}

/// Mutable state, behind one lock.
#[derive(Default)]
pub(crate) struct Inner {
    pub(crate) entries: Vec<Entry>,
    pub(crate) log: Vec<DirectoryChange>,
    /// Batches and whether they are still open.
    pub(crate) batches: Vec<(ObjectId, bool)>,
    pub(crate) tick: u64,
    /// Consent records in the order they were recorded (oldest first).
    pub(crate) consents: Vec<Consent>,
    /// A deliberate rule break (mutation test of the conformance rules).
    pub(crate) defect: Option<Defect>,
}

/// In-memory provider; `asker` is a property of the provider (the "I" card).
#[derive(Default)]
pub struct FakeDirectory {
    /// A foreign address book: reading works, `person.propose` is `Forbidden`.
    pub(crate) foreign: bool,
    pub(crate) asker: Option<PersonRef>,
    pub(crate) rights: Rights,
    /// Shared with the views made by [`FakeDirectory::view_as`].
    pub(crate) inner: Arc<Mutex<Inner>>,
}

// The mutators report whether the row existed; a caller may ignore that (the state is shared behind an `Arc`, so the
// lint does not see the interior mutability).
#[allow(clippy::must_use_candidate)]
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

    /// Break one rule on purpose (see [`Defect`]); the views of this fake share it.
    #[doc(hidden)]
    #[must_use]
    pub fn with_defect(self, defect: Defect) -> Self {
        self.lock().defect = Some(defect);
        self
    }

    /// Set the `consent.*` rights of the asker (default: all three).
    #[must_use]
    pub fn with_rights(mut self, rights: Rights) -> Self {
        self.rights = rights;
        self
    }

    /// The same data asked by someone else, with their own rights: a second door to the shared state. An asker
    /// whose entry has `ai` set is an AI (the AI rules apply).
    #[must_use]
    pub fn view_as(&self, asker: Option<PersonRef>, rights: Rights) -> Self {
        Self {
            foreign: self.foreign,
            asker,
            rights,
            inner: Arc::clone(&self.inner),
        }
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
        let done = g.set_status(person, Status::Deleted(at), ChangeKind::Deleted);
        if done && !g.bad(Defect::EraseKeepsActive) {
            // The contract: the one who deletes records the withdrawal (the person only if nobody asks).
            let by = self
                .asker
                .clone()
                .filter(|_| !g.bad(Defect::ErasedByPerson))
                .unwrap_or_else(|| person.clone());
            g.erase_consents(person, &by);
        }
        done
    }

    /// The retention time is over: delete the withdrawn records whose `retain_until` is before `today`, for good.
    /// The consents are the only history the fake keeps, so they are gone from `consents_of` afterwards.
    pub fn purge_expired(&self, today: crate::person_directory_v1::Date) -> usize {
        let mut g = self.lock();
        if g.bad(Defect::PurgeKeeps) {
            return 0;
        }
        let before = g.consents.len();
        g.consents
            .retain(|c| c.retain_until.is_none_or(|d| d >= today));
        before - g.consents.len()
    }

    /// Link a profile to a person (`describes`), or cut the link with `None`; logs `Changed`.
    pub fn set_describes(&self, profile: &PersonRef, person: Option<PersonRef>) -> bool {
        let mut g = self.lock();
        let Some(e) = g.find_mut(profile) else {
            return false;
        };
        e.describes = person;
        e.rev += 1;
        g.record(profile.clone(), ChangeKind::Changed);
        true
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
    /// Is this deliberate defect switched on?
    pub(crate) fn bad(&self, defect: Defect) -> bool {
        self.defect == Some(defect)
    }

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
