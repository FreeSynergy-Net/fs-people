//! `CompositeDirectory`: asks its members in order; the first answer that is not "unknown" counts.
//!
//! Pattern: Composite (a chain of `PersonDirectory` members is itself a `PersonDirectory`) with Chain of
//! Responsibility for reading; writing (`open_batch`, `propose`) goes to the first member with `person.propose`.

use crate::person_directory_v1::{
    Candidate, Capabilities, ContactAddress, DirectoryChange, Error, Excerpt, ExcerptField,
    NoExcerptReason, ObjectId, Page, PersonDirectory, PersonRef, PersonView, Place, Problem,
    ProposeEntry, Reference, Resolved, Result, Suggest, Unreachable, not_possible,
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

    /// One `resolve_many` per member for the entries that are still `Unknown`; the rest goes on to the next member.
    /// A member that is down while entries stay `Unknown` makes the whole answer `Unavailable`, like `resolve`.
    async fn resolve_many(&self, people: &[PersonRef]) -> Result<Vec<Resolved>> {
        let mut out: Vec<Option<Resolved>> = vec![None; people.len()];
        let mut pending: Vec<usize> = (0..people.len()).collect();
        let mut seen = None;
        for m in &self.members {
            if pending.is_empty() {
                break;
            }
            let ask: Vec<PersonRef> = pending.iter().map(|&i| people[i].clone()).collect();
            match m.resolve_many(&ask).await {
                Ok(answers) if answers.len() == ask.len() => {
                    let mut still = Vec::new();
                    for (&i, answer) in pending.iter().zip(answers) {
                        match answer {
                            Resolved::Unknown => still.push(i),
                            other => out[i] = Some(other),
                        }
                    }
                    pending = still;
                }
                // A wrong length breaks the contract of the member: treat it as a broken store.
                Ok(_) => seen = Some(Unreachable::Storage),
                Err(Error::Unavailable(u)) => seen = Some(u),
                Err(e) => return Err(e),
            }
        }
        if !pending.is_empty() && seen.is_some() {
            return Err(unavailable(seen));
        }
        Ok(out
            .into_iter()
            .map(|r| r.unwrap_or(Resolved::Unknown))
            .collect())
    }

    /// Change cursors are opaque and belong to one provider, so the feed of the first reachable member is
    /// handed through (a merged feed needs a cursor of its own; see DESIGN.md).
    async fn changes(&self, since: Option<&Place>, size: usize) -> Result<Page<DirectoryChange>> {
        let mut seen = None;
        for m in &self.members {
            match m.changes(since, size).await {
                Err(Error::Unavailable(u)) => seen = Some(u),
                // A member that cannot do it passes on, as with `excerpt`.
                Err(Error::Unsupported { .. }) => {}
                other => return other,
            }
        }
        match seen {
            Some(u) => Err(unavailable(Some(u))),
            None => not_possible("person.read"),
        }
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

    /// The lists of the members are merged by position: every first suggestion in member order, then every
    /// second one, and so on (a candidate carries no rank of its own to compare across providers); duplicates
    /// (same reference) keep their first place. `limit` is checked here, so a chain of members that are all down
    /// still says `Invalid` for a bad `limit`.
    async fn suggest(&self, query: &Suggest) -> Result<Vec<Candidate>> {
        if query.limit == 0 || query.limit > 50 {
            return Err(Error::Invalid(vec![Problem {
                field: "limit".to_owned(),
                reason: "out_of_range".to_owned(),
            }]));
        }
        let mut lists: Vec<Vec<Candidate>> = Vec::new();
        let mut seen = None;
        for m in &self.members {
            match m.suggest(query).await {
                Ok(list) => lists.push(list),
                Err(Error::Unavailable(u)) => seen = Some(u),
                Err(e) => return Err(e),
            }
        }
        if lists.is_empty() {
            return Err(unavailable(seen));
        }
        let mut out: Vec<Candidate> = Vec::new();
        let longest = lists.iter().map(Vec::len).max().unwrap_or(0);
        for rank in 0..longest {
            for list in &lists {
                if let Some(c) = list.get(rank)
                    && !out.iter().any(|o| o.view.person.id == c.view.person.id)
                {
                    out.push(c.clone());
                }
            }
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
