//! `ConformanceFixture`: how a provider under test is put into the states the rules need.
//!
//! Pattern: Strategy / Test fixture (the rules are written once; each provider says how to create a deleted,
//! unshared or merged entry, so the same rules run against the fake and later against `PeopleRepo`).

use crate::person_directory_v1::{PersonArt, PersonDirectory, PersonRef};
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
}
