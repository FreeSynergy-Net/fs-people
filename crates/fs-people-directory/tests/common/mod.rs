//! Stub provider for the tests: only the required methods, the optional ones keep their Null Object default.
//!
//! Pattern: Test double (Null Object defaults and an unavailable provider).

use async_trait::async_trait;
use fs_people_directory::{
    Candidate, Capabilities, ContactAddress, DirectoryChange, Page, PersonDirectory, PersonRef,
    PersonView, Place, Resolved, Result, Suggest,
};

/// A provider with only the required methods: the optional ones keep their Null Object default.
pub struct ReadOnly;

#[async_trait]
impl PersonDirectory for ReadOnly {
    fn capabilities(&self) -> Capabilities {
        Capabilities(vec![
            ("person.read".to_owned(), true),
            ("person.propose".to_owned(), false),
        ])
    }
    async fn resolve(&self, _: &PersonRef) -> Result<Resolved> {
        Ok(Resolved::Unknown)
    }
    async fn asker(&self) -> Result<Option<PersonRef>> {
        Ok(None)
    }
    async fn resolve_many(&self, p: &[PersonRef]) -> Result<Vec<Resolved>> {
        Ok(p.iter().map(|_| Resolved::Unknown).collect())
    }
    async fn changes(&self, _: Option<&Place>, _: usize) -> Result<Page<DirectoryChange>> {
        Ok(Page {
            items: vec![],
            next: Some(Place("0".into())),
        })
    }
    async fn find_by_address(&self, _: &ContactAddress) -> Result<Vec<PersonView>> {
        Ok(vec![])
    }
    async fn suggest(&self, _: &Suggest) -> Result<Vec<Candidate>> {
        Ok(vec![])
    }
}
