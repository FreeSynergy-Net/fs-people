//! The `PersonDirectory` implementation of the fake.
//!
//! Pattern: Fake behind the Repository trait; the rules are those written in the doc comments of `api.rs`.

use super::ids::{uuid7, uuid7_urn};
use super::store::{find_order, same_address, suggest_rank};
use super::{Entry, FakeDirectory, Inner, Status};
use crate::person_directory_v1::{
    CAPABILITIES, Candidate, Capabilities, ChangeKind, ContactAddress, DirectoryChange, EntryState,
    Error, Excerpt, ExcerptField, Matched, NoExcerptReason, ObjectId, Page, PersonArt,
    PersonDirectory, PersonRef, PersonView, Place, Problem, ProposeEntry, Reference, Resolved,
    Result, Suggest,
};
use async_trait::async_trait;
use fs_types::BasicValue;

fn forbidden<T>() -> Result<T> {
    Err(Error::Forbidden {
        capability: "person.propose".to_owned(),
    })
}

fn invalid<T>(field: &str, reason: &str) -> Result<T> {
    Err(Error::Invalid(vec![Problem {
        field: field.to_owned(),
        reason: reason.to_owned(),
    }]))
}

impl Inner {
    /// The row a reference ends at; a merge chain is followed, with a bound against cycles.
    fn target(&self, person: &PersonRef) -> Option<&Entry> {
        let mut current = person.id.clone();
        for _ in 0..8 {
            let e = self.entries.iter().find(|e| e.person.id == current)?;
            match (&e.merged_into, e.status) {
                (Some(into), Status::Active) => current = into.id.clone(),
                _ => return Some(e),
            }
        }
        None
    }

    fn resolve(&self, person: &PersonRef) -> Resolved {
        match self.target(person).map(|e| (e, e.status)) {
            None => Resolved::Unknown,
            Some((e, Status::Active)) => Resolved::Found(e.view()),
            Some((_, Status::Deleted(since))) => Resolved::Deleted { since },
            Some((_, Status::NotShared)) => Resolved::NotShared,
        }
    }

    fn matching(&self, address: &ContactAddress) -> Vec<PersonView> {
        let mut hits: Vec<(usize, &Entry)> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                e.status == Status::Active
                    && e.merged_into.is_none()
                    && e.address.as_ref().is_some_and(|a| same_address(a, address))
            })
            .collect();
        hits.sort_by(find_order);
        hits.into_iter().map(|(_, e)| e.view()).collect()
    }
}

#[async_trait]
impl PersonDirectory for FakeDirectory {
    fn capabilities(&self) -> Capabilities {
        Capabilities(
            CAPABILITIES
                .iter()
                .map(|c| ((*c).to_owned(), true))
                .collect(),
        )
    }

    async fn resolve(&self, person: &PersonRef) -> Result<Resolved> {
        Ok(self.lock().resolve(person))
    }

    async fn asker(&self) -> Result<Option<PersonRef>> {
        Ok(self.asker.clone())
    }

    async fn resolve_many(&self, people: &[PersonRef]) -> Result<Vec<Resolved>> {
        let g = self.lock();
        Ok(people.iter().map(|p| g.resolve(p)).collect())
    }

    async fn changes(&self, since: Option<&Place>, size: usize) -> Result<Page<DirectoryChange>> {
        if size == 0 {
            return invalid("size", "out_of_range");
        }
        let g = self.lock();
        let from = match since {
            None => g.log.len(),
            Some(p) => match p.0.parse::<usize>() {
                Ok(n) if n <= g.log.len() => n,
                _ => return invalid("since", "bad_cursor"),
            },
        };
        let items: Vec<DirectoryChange> = g.log.iter().skip(from).take(size).cloned().collect();
        let next = Some(Place((from + items.len()).to_string()));
        Ok(Page { items, next })
    }

    async fn find_by_address(&self, address: &ContactAddress) -> Result<Vec<PersonView>> {
        Ok(self.lock().matching(address))
    }

    async fn suggest(&self, query: &Suggest) -> Result<Vec<Candidate>> {
        if query.limit == 0 || query.limit > 50 {
            return invalid("limit", "out_of_range");
        }
        let g = self.lock();
        let mut hits: Vec<(u8, &Entry, bool)> = g
            .entries
            .iter()
            .filter(|e| e.status == Status::Active && e.merged_into.is_none())
            .filter(|e| query.arten.is_empty() || query.arten.contains(&e.person.art))
            .filter(|e| query.include_proposed || e.state == EntryState::Confirmed)
            .filter_map(|e| suggest_rank(e, query.text.trim()).map(|(r, name)| (r, e, name)))
            .collect();
        hits.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.sort_key.cmp(&b.1.sort_key)));
        Ok(hits
            .into_iter()
            .take(usize::from(query.limit))
            .map(|(_, e, by_name)| Candidate {
                view: e.view(),
                matched: if !by_name {
                    Matched::Address
                } else if e.person.art == PersonArt::Organization {
                    Matched::Organization
                } else {
                    Matched::Name
                },
            })
            .collect())
    }

    async fn open_batch(&self, _origin: &Reference) -> Result<ObjectId> {
        if self.foreign {
            return forbidden();
        }
        let mut g = self.lock();
        let ms = g.now().millis();
        let id = ObjectId(uuid7(ms).to_be_bytes());
        g.batches.push((id, true));
        Ok(id)
    }

    async fn propose(&self, entry: &ProposeEntry, batch: Option<ObjectId>) -> Result<PersonRef> {
        if self.foreign {
            return forbidden();
        }
        let hint = entry.display_hint.trim();
        if hint.is_empty() && entry.address.is_none() {
            return invalid("display_hint", "empty");
        }
        let mut g = self.lock();
        if let Some(b) = batch
            && !g.batches.iter().any(|(id, open)| *id == b && *open)
        {
            return Err(Error::BatchClosed(b));
        }
        if let Some(address) = &entry.address
            && let Some(first) = g.matching(address).into_iter().next()
        {
            return Ok(first.person);
        }
        let ms = g.now().millis();
        let id = Reference::parse(&uuid7_urn(ms))?;
        let person = PersonRef {
            id,
            art: PersonArt::Profile,
            abzug: None,
        };
        let shown = if hint.is_empty() {
            entry
                .address
                .as_ref()
                .map(ContactAddress::canonical)
                .unwrap_or_default()
        } else {
            hint.to_owned()
        };
        let mut row = Entry::new(person.clone(), &shown);
        row.state = EntryState::Proposed;
        row.address.clone_from(&entry.address);
        row.created = g.now();
        g.entries.push(row);
        g.record(person.clone(), ChangeKind::Created);
        Ok(person)
    }

    async fn excerpt(&self, person: &PersonRef, fields: &[ExcerptField]) -> Result<Excerpt> {
        if fields.is_empty() {
            return invalid("fields", "empty");
        }
        let mut g = self.lock();
        let captured = g.now();
        let refuse = |reason| {
            Err(Error::NoExcerpt {
                person: person.clone(),
                reason,
            })
        };
        let Some(e) = g.target(person) else {
            return refuse(NoExcerptReason::Unknown);
        };
        match e.status {
            Status::Deleted(_) => return refuse(NoExcerptReason::Deleted),
            Status::NotShared => return refuse(NoExcerptReason::NotShared),
            Status::Active => {}
        }
        let mut values = Vec::new();
        let mut missing = Vec::new();
        for f in fields {
            if values.iter().any(|v: &crate::ExcerptValue| v.field() == *f) || missing.contains(f) {
                continue;
            }
            match e.excerpt.iter().find(|v| v.field() == *f) {
                Some(v) => values.push(v.clone()),
                None => missing.push(*f),
            }
        }
        Ok(Excerpt {
            person: e.view().person,
            source: e.person.id.clone(),
            captured,
            values,
            missing,
        })
    }
}
