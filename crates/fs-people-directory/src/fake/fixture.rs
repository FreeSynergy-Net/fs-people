//! `FakeFixture`: the fake as a `ConformanceFixture`, so the state rules run against it.
//!
//! Pattern: Strategy (the fixture side of the conformance rules, implemented over the fake's mutators).

use super::ids::uuid7_urn;
use super::{Entry, FakeDirectory, Rights};
use crate::conformance::ConformanceFixture;
use crate::person_directory_v1::{Birthday, ExcerptValue, PersonArt, PersonDirectory, PersonRef};
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

    async fn add_ai(&self, display: &str) -> Option<PersonRef> {
        let person = self.add_confirmed(PersonArt::Person, display, None).await?;
        self.directory
            .lock()
            .find_mut(&person)
            .map(|e| e.ai = true)?;
        Some(person)
    }

    async fn set_birthday(&self, person: &PersonRef, birthday: Birthday) -> bool {
        let mut g = self.directory.lock();
        let Some(e) = g.find_mut(person) else {
            return false;
        };
        e.excerpt.retain(|v| {
            !matches!(
                v,
                ExcerptValue::Birthday(_) | ExcerptValue::BirthMonthDay(_)
            )
        });
        e.excerpt.push(ExcerptValue::Birthday(birthday));
        true
    }

    async fn describe(&self, profile: &PersonRef, person: Option<&PersonRef>) -> bool {
        self.directory.set_describes(profile, person.cloned())
    }

    async fn admin_view(&self) -> Option<Box<dyn PersonDirectory>> {
        let admin = self
            .add_confirmed(PersonArt::Person, "Zzconf Admin", None)
            .await?;
        Some(Box::new(self.directory.view_as(Some(admin), Rights::all())))
    }

    async fn member_view(
        &self,
        who: &PersonRef,
        rights: &[&str],
    ) -> Option<Box<dyn PersonDirectory>> {
        let held = Rights {
            read: rights.contains(&"consent.read"),
            check: rights.contains(&"consent.check"),
            write: rights.contains(&"consent.write"),
        };
        Some(Box::new(self.directory.view_as(Some(who.clone()), held)))
    }
}
