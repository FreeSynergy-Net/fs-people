//! Stub provider for the tests: read-only, every other method says `Unsupported` explicitly (no default methods).
//!
//! Pattern: Test double (explicit Null Object: `not_possible` for what it does not offer).

use async_trait::async_trait;
use fs_people_directory::{
    Candidate, Capabilities, Consent, ConsentEntry, ConsentQuery, ContactAddress, DirectoryChange,
    Excerpt, ExcerptRequest, ObjectId, Page, PersonDirectory, PersonRef, PersonView, Place,
    ProposeEntry, Reference, Resolved, Result, Suggest, Withdrawal, not_possible,
};

/// A read-only provider: it answers `Unsupported` for proposals, excerpts and consents.
pub struct ReadOnly;

#[async_trait]
impl PersonDirectory for ReadOnly {
    fn capabilities(&self) -> Capabilities {
        Capabilities(vec![
            ("person.read".to_owned(), true),
            ("person.propose".to_owned(), false),
            ("person.excerpt".to_owned(), false),
            ("consent.read".to_owned(), false),
            ("consent.check".to_owned(), false),
            ("consent.write".to_owned(), false),
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
    async fn open_batch(&self, _: &Reference) -> Result<ObjectId> {
        not_possible("person.propose")
    }
    async fn propose(&self, _: &ProposeEntry, _: Option<ObjectId>) -> Result<PersonRef> {
        not_possible("person.propose")
    }
    async fn excerpt(&self, _: &PersonRef, _: &ExcerptRequest) -> Result<Excerpt> {
        not_possible("person.excerpt")
    }
    async fn consents_of(&self, _: &PersonRef) -> Result<Vec<Consent>> {
        not_possible("consent.read")
    }
    async fn check_consent(&self, _: &ConsentQuery) -> Result<Option<ObjectId>> {
        not_possible("consent.check")
    }
    async fn record_consent(&self, _: &ConsentEntry) -> Result<Consent> {
        not_possible("consent.write")
    }
    async fn withdraw_consent(&self, _: ObjectId, _: &Withdrawal) -> Result<Consent> {
        not_possible("consent.write")
    }
}
