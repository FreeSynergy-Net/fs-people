//! `CompositeDirectory`: asks its members in order; the first answer that is not "unknown" counts.
//!
//! Pattern: Composite (a chain of `PersonDirectory` members is itself a `PersonDirectory`) with Chain of
//! Responsibility for reading; writing (`open_batch`, `propose`) goes to the first member with `person.propose`.

use crate::person_directory_v1::{
    Candidate, Capabilities, ContactAddress, DirectoryChange, Error, Excerpt, ExcerptField,
    NoExcerptReason, ObjectId, Page, PersonDirectory, PersonRef, PersonView, Place, ProposeEntry,
    Reference, Resolved, Result, Suggest, Unreachable, not_possible,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};

/// A chain of providers. `Unknown` and `Unavailable` pass the question on; every other answer ends it.
pub struct CompositeDirectory {
    members: Vec<Box<dyn PersonDirectory>>,
    /// Which member issued a batch id; the id counts only there.
    batches: Mutex<HashMap<ObjectId, usize>>,
}

impl CompositeDirectory {
    /// A chain over `members`, asked in this order.
    #[must_use]
    pub fn new(members: Vec<Box<dyn PersonDirectory>>) -> Self {
        Self {
            members,
            batches: Mutex::new(HashMap::new()),
        }
    }

    fn proposer(&self) -> Option<usize> {
        self.members.iter().position(|m| {
            m.capabilities()
                .0
                .iter()
                .any(|(c, on)| c == "person.propose" && *on)
        })
    }
}

fn unavailable(seen: Option<Unreachable>) -> Error {
    Error::Unavailable(seen.unwrap_or(Unreachable::NotRunning))
}

#[async_trait]
impl PersonDirectory for CompositeDirectory {
    fn capabilities(&self) -> Capabilities {
        let mut all: Vec<(String, bool)> = Vec::new();
        for m in &self.members {
            for (name, on) in m.capabilities().0 {
                match all.iter_mut().find(|(n, _)| *n == name) {
                    Some(slot) => slot.1 |= on,
                    None => all.push((name, on)),
                }
            }
        }
        Capabilities(all)
    }

    async fn resolve(&self, person: &PersonRef) -> Result<Resolved> {
        let mut seen = None;
        for m in &self.members {
            match m.resolve(person).await {
                Ok(Resolved::Unknown) => {}
                Err(Error::Unavailable(u)) => seen = Some(u),
                other => return other,
            }
        }
        seen.map_or(Ok(Resolved::Unknown), |u| Err(unavailable(Some(u))))
    }

    async fn asker(&self) -> Result<Option<PersonRef>> {
        let mut seen = None;
        for m in &self.members {
            match m.asker().await {
                Ok(None) => {}
                Err(Error::Unavailable(u)) => seen = Some(u),
                other => return other,
            }
        }
        seen.map_or(Ok(None), |u| Err(unavailable(Some(u))))
    }

    async fn resolve_many(&self, people: &[PersonRef]) -> Result<Vec<Resolved>> {
        let mut out = Vec::with_capacity(people.len());
        for p in people {
            out.push(self.resolve(p).await?);
        }
        Ok(out)
    }

    /// Change cursors are opaque and belong to one provider, so the feed of the first reachable member is
    /// handed through (a merged feed needs a cursor of its own; see DESIGN.md).
    async fn changes(&self, since: Option<&Place>, size: usize) -> Result<Page<DirectoryChange>> {
        let mut seen = None;
        for m in &self.members {
            match m.changes(since, size).await {
                Err(Error::Unavailable(u)) => seen = Some(u),
                other => return other,
            }
        }
        Err(unavailable(seen))
    }

    async fn find_by_address(&self, address: &ContactAddress) -> Result<Vec<PersonView>> {
        let mut out: Vec<PersonView> = Vec::new();
        let mut seen = None;
        let mut answered = false;
        for m in &self.members {
            match m.find_by_address(address).await {
                Ok(list) => {
                    answered = true;
                    for v in list {
                        if !out.iter().any(|o| o.person.id == v.person.id) {
                            out.push(v);
                        }
                    }
                }
                Err(Error::Unavailable(u)) => seen = Some(u),
                Err(e) => return Err(e),
            }
        }
        if answered {
            Ok(out)
        } else {
            Err(unavailable(seen))
        }
    }

    async fn suggest(&self, query: &Suggest) -> Result<Vec<Candidate>> {
        let mut out: Vec<Candidate> = Vec::new();
        let mut seen = None;
        let mut answered = false;
        for m in &self.members {
            match m.suggest(query).await {
                Ok(list) => {
                    answered = true;
                    for c in list {
                        if !out.iter().any(|o| o.view.person.id == c.view.person.id) {
                            out.push(c);
                        }
                    }
                }
                Err(Error::Unavailable(u)) => seen = Some(u),
                Err(e) => return Err(e),
            }
        }
        if !answered {
            return Err(unavailable(seen));
        }
        out.truncate(usize::from(query.limit));
        Ok(out)
    }

    async fn open_batch(&self, origin: &Reference) -> Result<ObjectId> {
        let Some(i) = self.proposer() else {
            return not_possible("person.propose");
        };
        let id = self.members[i].open_batch(origin).await?;
        self.batches
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(id, i);
        Ok(id)
    }

    async fn propose(&self, entry: &ProposeEntry, batch: Option<ObjectId>) -> Result<PersonRef> {
        let issuer = batch.map(|b| {
            let map = self.batches.lock().unwrap_or_else(PoisonError::into_inner);
            (b, map.get(&b).copied())
        });
        let target = match issuer {
            Some((b, None)) => return Err(Error::BatchClosed(b)),
            Some((_, Some(i))) => i,
            None => match self.proposer() {
                Some(i) => i,
                None => return not_possible("person.propose"),
            },
        };
        self.members[target].propose(entry, batch).await
    }

    async fn excerpt(&self, person: &PersonRef, fields: &[ExcerptField]) -> Result<Excerpt> {
        let mut seen = None;
        for m in &self.members {
            match m.excerpt(person, fields).await {
                Err(
                    Error::NoExcerpt {
                        reason: NoExcerptReason::Unknown,
                        ..
                    }
                    | Error::Unsupported { .. },
                ) => {}
                Err(Error::Unavailable(u)) => seen = Some(u),
                other => return other,
            }
        }
        match seen {
            Some(u) => Err(unavailable(Some(u))),
            None => Err(Error::NoExcerpt {
                person: person.clone(),
                reason: NoExcerptReason::Unknown,
            }),
        }
    }
}
