//! `ConformanceFixture`: how a provider under test is put into the states the rules need.
//!
//! Pattern: Strategy / Test fixture (the rules are written once; each provider says how to create a deleted,
//! unshared or merged entry, so the same rules run against the fake and later against `PeopleRepo`).

use crate::person_directory_v1::{Birthday, PersonArt, PersonDirectory, PersonRef};
use async_trait::async_trait;

/// The provider under test plus the mutators behind it. A mutator that the provider cannot offer keeps its
/// default (`None` / `false`); the rules that need it are reported as skipped, not as passed.
#[async_trait]
pub trait ConformanceFixture: Send + Sync {
    /// The provider under test.
    fn directory(&self) -> &dyn PersonDirectory;

    /// Create a confirmed entry (with a full-name excerpt) and return its reference. `display` carries the marker
    /// `Zzconf`.
    async fn add_confirmed(
        &self,
        _art: PersonArt,
        _display: &str,
        _address: Option<&str>,
    ) -> Option<PersonRef> {
        None
    }

    /// Delete (tombstone) an entry.
    async fn delete(&self, _person: &PersonRef) -> bool {
        false
    }

    /// Withdraw the sharing of an entry.
    async fn unshare(&self, _person: &PersonRef) -> bool {
        false
    }

    /// Bring a deleted or unshared entry back (the change feed says `Restored`).
    async fn restore(&self, _person: &PersonRef) -> bool {
        false
    }

    /// Merge `from` into `into`.
    async fn merge(&self, _from: &PersonRef, _into: &PersonRef) -> bool {
        false
    }

    /// Change the display name of an entry.
    async fn rename(&self, _person: &PersonRef, _display: &str) -> bool {
        false
    }

    /// A view of a foreign address book: reading works, `person.propose` is `Forbidden`.
    async fn foreign_book(&self) -> Option<Box<dyn PersonDirectory>> {
        None
    }

    /// Create a confirmed entry with the AI mark (`PersonView::ai`); `display` carries the marker `Zzconf`.
    async fn add_ai(&self, _display: &str) -> Option<PersonRef> {
        None
    }

    /// Give an entry a birthday (with or without year) that `excerpt` hands out.
    async fn set_birthday(&self, _person: &PersonRef, _birthday: Birthday) -> bool {
        false
    }

    /// Link a profile to a person (`describes`); `None` cuts the link.
    async fn describe(&self, _profile: &PersonRef, _person: Option<&PersonRef>) -> bool {
        false
    }

    /// The same data asked by a signed-in administrator: all `consent.*` rights (read, check, write).
    async fn admin_view(&self) -> Option<Box<dyn PersonDirectory>> {
        None
    }

    /// The same data asked by `who` holding exactly the named rights of `consent.read`, `consent.check`,
    /// `consent.write` (none: an ordinary member; an AI if `who` was made with `add_ai`).
    async fn member_view(
        &self,
        _who: &PersonRef,
        _rights: &[&str],
    ) -> Option<Box<dyn PersonDirectory>> {
        None
    }
}
