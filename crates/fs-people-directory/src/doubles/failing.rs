//! `FailingDirectory`: answers every call with one fixed `Error`, so every variant is reachable in a test.
//!
//! Pattern: Test double (Stub that fails; `each_variant` lists one per `Error` variant).

use crate::fake::sample_ref;
use crate::person_directory_v1::{
    Candidate, Capabilities, ContactAddress, DirectoryChange, Error, Excerpt, ExcerptField,
    NoExcerptReason, ObjectId, Page, PersonDirectory, PersonRef, PersonView, Place, Problem,
    ProposeEntry, Reference, Resolved, Result, Suggest, Unreachable,
};
use async_trait::async_trait;

/// A provider whose every method returns the same error; it offers no capability.
#[derive(Clone, Debug)]
pub struct FailingDirectory {
    error: Error,
}

impl FailingDirectory {
    /// Fail with `error` on every call.
    #[must_use]
    pub fn new(error: Error) -> Self {
        Self { error }
    }

    /// One provider per `Error` variant (`Unavailable` once per cause), in the order of the enum.
    #[must_use]
    pub fn each_variant() -> Vec<Self> {
        let mut errors = vec![
            Error::Unsupported {
                capability: "person.propose".to_owned(),
                reason: "this provider does not offer it".to_owned(),
            },
            Error::Forbidden {
                capability: "person.propose".to_owned(),
            },
            Error::Invalid(vec![Problem {
                field: "limit".to_owned(),
                reason: "out_of_range".to_owned(),
            }]),
            Error::BatchClosed(ObjectId([0; 16])),
        ];
        if let Some(person) = sample_ref(1) {
            for reason in [
                NoExcerptReason::Deleted,
                NoExcerptReason::NotShared,
                NoExcerptReason::Unknown,
            ] {
                errors.push(Error::NoExcerpt {
                    person: person.clone(),
                    reason,
                });
            }
        }
        for cause in [
            Unreachable::NotRunning,
            Unreachable::Timeout,
            Unreachable::Locked,
            Unreachable::Storage,
        ] {
            errors.push(Error::Unavailable(cause));
        }
        errors.into_iter().map(Self::new).collect()
    }

    fn fail<T>(&self) -> Result<T> {
        Err(self.error.clone())
    }
}

#[async_trait]
impl PersonDirectory for FailingDirectory {
    fn capabilities(&self) -> Capabilities {
        Capabilities::default()
    }
    async fn resolve(&self, _: &PersonRef) -> Result<Resolved> {
        self.fail()
    }
    async fn asker(&self) -> Result<Option<PersonRef>> {
        self.fail()
    }
    async fn resolve_many(&self, _: &[PersonRef]) -> Result<Vec<Resolved>> {
        self.fail()
    }
    async fn changes(&self, _: Option<&Place>, _: usize) -> Result<Page<DirectoryChange>> {
        self.fail()
    }
    async fn find_by_address(&self, _: &ContactAddress) -> Result<Vec<PersonView>> {
        self.fail()
    }
    async fn suggest(&self, _: &Suggest) -> Result<Vec<Candidate>> {
        self.fail()
    }
    async fn open_batch(&self, _: &Reference) -> Result<ObjectId> {
        self.fail()
    }
    async fn propose(&self, _: &ProposeEntry, _: Option<ObjectId>) -> Result<PersonRef> {
        self.fail()
    }
    async fn excerpt(&self, _: &PersonRef, _: &[ExcerptField]) -> Result<Excerpt> {
        self.fail()
    }
}
