//! `FakeFixture`: the fake as a `ConformanceFixture`, so the state rules run against it.
//!
//! Pattern: Strategy (the fixture side of the conformance rules, implemented over the fake's mutators).

use super::ids::uuid7_urn;
use super::{Entry, FakeDirectory};
use crate::conformance::ConformanceFixture;
use crate::person_directory_v1::{ExcerptValue, PersonArt, PersonDirectory, PersonRef};
use async_trait::async_trait;
use fs_types::{BasicValue, ContactAddress, Name, Reference};

/// Owns a [`FakeDirectory`] and exposes its mutators to the conformance rules.
#[derive(Default)]
pub struct FakeFixture {
    pub directory: FakeDirectory,
}

impl FakeFixture {
    /// A fixture over an empty fake.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A fixture over `directory` (for example [`super::sample`]).
    #[must_use]
    pub fn over(directory: FakeDirectory) -> Self {
        Self { directory }
    }
}

#[async_trait]
impl ConformanceFixture for FakeFixture {
    fn directory(&self) -> &dyn PersonDirectory {
        &self.directory
    }

    async fn add_confirmed(
        &self,
        art: PersonArt,
        display: &str,
        address: Option<&str>,
    ) -> Option<PersonRef> {
        let ms = self.directory.lock().now().millis();
        let person = PersonRef {
            id: Reference::parse(&uuid7_urn(ms)).ok()?,
            art,
            abzug: None,
        };
        let mut entry = Entry::new(person.clone(), display);
        entry.address = address.and_then(|a| ContactAddress::parse(a).ok());
        entry
            .excerpt
            .push(ExcerptValue::FullName(Name::new(display, &[]).ok()?));
        self.directory.add(entry);
        Some(person)
    }

    async fn delete(&self, person: &PersonRef) -> bool {
        self.directory.delete(person)
    }

    async fn unshare(&self, person: &PersonRef) -> bool {
        self.directory.unshare(person)
    }

    async fn restore(&self, person: &PersonRef) -> bool {
        self.directory.restore(person)
    }

    async fn merge(&self, from: &PersonRef, into: &PersonRef) -> bool {
        self.directory.merge(from, into)
    }

    async fn rename(&self, person: &PersonRef, display: &str) -> bool {
        self.directory.rename(person, display)
    }

    async fn foreign_book(&self) -> Option<Box<dyn PersonDirectory>> {
        Some(Box::new(FakeDirectory::new().foreign_book()))
    }
}
