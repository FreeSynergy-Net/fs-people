//! Der Null-Anbieter: JEDE der 15 Methoden von `PersonDirectory` v2 (und der 5 von `PurposeCatalog`) antwortet ausdrücklich `not_possible`. Er prüft in
//! der Probe, dass der Trait als `dyn` benutzbar ist und keine Methode eine Standard-Implementierung hat.
//!
//! Pattern: Null Object (ADR 0043/0045).

use super::PurposeCatalog;
use super::{
    CAPABILITIES, Candidate, Capabilities, Consent, ConsentEntry, ConsentQuery, ContactAddress,
    DirectoryChange, Error, Excerpt, ExcerptRequest, ObjectId, Page, PersonDirectory, PersonRef,
    PersonView, Place, ProposeEntry, Purpose, PurposeDef, PurposeDraft, Reference, Resolved,
    Result, Revision, Suggest, When, Withdrawal, not_possible,
};
use async_trait::async_trait;
use fs_types::Zone;

/// Anbieter ohne jede Fähigkeit.
#[derive(Clone, Copy, Debug, Default)]
pub struct NullDirectory;

#[async_trait]
impl PersonDirectory for NullDirectory {
    fn capabilities(&self) -> Capabilities {
        Capabilities(
            CAPABILITIES
                .iter()
                .map(|c| ((*c).to_owned(), false))
                .collect(),
        )
    }
    async fn resolve(&self, _: &PersonRef) -> Result<Resolved> {
        not_possible("person.read")
    }
    async fn asker(&self) -> Result<Option<PersonRef>> {
        not_possible("person.read")
    }
    async fn resolve_many(&self, _: &[PersonRef]) -> Result<Vec<Resolved>> {
        not_possible("person.read")
    }
    async fn changes(&self, _: Option<&Place>, _: usize) -> Result<Page<DirectoryChange>> {
        not_possible("person.read")
    }
    async fn find_by_address(&self, _: &ContactAddress) -> Result<Vec<PersonView>> {
        not_possible("person.find")
    }
    async fn suggest(&self, _: &Suggest) -> Result<Vec<Candidate>> {
        not_possible("person.pick")
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
    async fn end_situation(
        &self,
        _: &Reference,
        _: When,
        _: Option<&Zone>,
        _: Option<&Reference>,
    ) -> Result<Vec<ObjectId>> {
        not_possible("consent.situation")
    }
}

/// Null-Anbieter des Zweck-Katalogs: jede Methode `not_possible`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NullCatalog;

#[async_trait]
impl PurposeCatalog for NullCatalog {
    async fn purposes(&self) -> Result<Vec<PurposeDef>> {
        not_possible("purpose.read")
    }
    async fn purpose_version(&self, _: &Reference) -> Result<Option<PurposeDef>> {
        not_possible("purpose.read")
    }
    async fn define_purpose(&self, _: &PurposeDraft) -> Result<PurposeDef> {
        not_possible("purpose.write")
    }
    async fn revise_purpose(
        &self,
        _: &Purpose,
        _: Option<Revision>,
        _: &PurposeDraft,
    ) -> Result<PurposeDef> {
        not_possible("purpose.write")
    }
    async fn retire_purpose(&self, _: &Purpose, _: Revision) -> Result<PurposeDef> {
        not_possible("purpose.write")
    }
}

/// Der Fehler, den jede Methode des Null-Anbieters liefert (für Tests).
#[must_use]
pub fn unsupported(capability: &str) -> Error {
    Error::Unsupported {
        capability: capability.to_owned(),
        reason: "this provider does not offer it".to_owned(),
    }
}
