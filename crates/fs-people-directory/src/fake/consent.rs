//! The consent store of the fake: record, withdraw, list and check, with the matching rule of `ConsentQuery`.
//!
//! Pattern: Fake (in-memory records behind the Repository trait; rights are a plain struct, no fs-access).

use super::ids::uuid7;
use super::ops::invalid;
use super::{Defect, FakeDirectory, Inner, Status};
use crate::ConsentRetention;
use crate::person_directory_v1::{
    Consent, ConsentEntry, ConsentOrigin, ConsentQuery, Date, Error, ObjectId, PersonRef, Purpose,
    Resolved, Result, Revision, When, Withdrawal, Withdrawn,
};

fn forbidden<T>(capability: &str) -> Result<T> {
    Err(Error::Forbidden {
        capability: capability.to_owned(),
    })
}

/// Keep the proof for the default of the setting of People (3 years from the end of the year; the fake stays at it).
fn retain_until(when: &When) -> Option<Date> {
    ConsentRetention::default().after(when)
}

fn open_or_same<T: PartialEq>(have: Option<&T>, want: Option<&T>) -> bool {
    have.is_none() || have == want
}

impl Inner {
    /// The id a reference ends at (merges followed); an unknown reference is its own end.
    pub(crate) fn target_id(&self, person: &PersonRef) -> fs_types::Reference {
        self.target(person)
            .map_or_else(|| person.id.clone(), |e| e.person.id.clone())
    }

    /// Does a record on `owner` count for a question about `query`? Same target after merges; a profile
    /// question also counts the person it describes (not the other way round, not other profiles).
    fn covers(&self, owner: &PersonRef, query: &PersonRef) -> bool {
        let have = self.target_id(owner);
        let same = if self.bad(Defect::NoMergeFollow) {
            owner.id == query.id
        } else {
            have == self.target_id(query)
        };
        if same {
            return true;
        }
        let owner_links = self.target(owner).and_then(|e| e.describes.as_ref());
        if self.bad(Defect::ProfileCoversPerson)
            && owner_links.is_some_and(|d| self.target_id(d) == self.target_id(query))
        {
            return true;
        }
        let described = self
            .target(query)
            .and_then(|e| e.describes.as_ref())
            .map(|d| self.target_id(d));
        described.is_some_and(|d| d == have)
    }

    fn fits(&self, c: &Consent, q: &ConsentQuery) -> bool {
        c.entry.purpose == q.purpose
            && (self.bad(Defect::IgnoreScope)
                || open_or_same(c.entry.scope.as_ref(), q.scope.as_ref()))
            && open_or_same(
                c.entry.recipient.as_ref().map(|p| &p.id),
                q.recipient.as_ref().map(|p| &p.id),
            )
            && self.covers(&c.entry.person, &q.person)
    }

    /// The oldest active record that covers the query.
    pub(crate) fn active_match(&self, q: &ConsentQuery) -> Option<ObjectId> {
        let hit = |c: &&Consent| c.withdrawn.is_none() && self.fits(c, q);
        self.consents.iter().find(hit).map(|c| c.id)
    }

    /// A deleted person: their active records count as withdrawn with `Erasure`.
    pub(crate) fn erase_consents(&mut self, person: &PersonRef, by: &PersonRef) {
        let at = self.now();
        let end = self.target_id(person);
        let owned: Vec<usize> = (0..self.consents.len())
            .filter(|&i| {
                self.consents[i].withdrawn.is_none()
                    && self.target_id(&self.consents[i].entry.person) == end
            })
            .collect();
        for i in owned {
            let withdrawal = Withdrawal {
                at: When::At(at),
                origin: ConsentOrigin::Erasure,
                evidence: None,
            };
            self.consents[i].retain_until = retain_until(&withdrawal.at);
            self.consents[i].withdrawn = Some(Withdrawn {
                withdrawal,
                recorded_by: by.clone(),
                recorded_at: at,
            });
            self.consents[i].rev = Revision(self.consents[i].rev.0 + 1);
        }
    }
}

impl FakeDirectory {
    /// Does the asker hold this record as their own (same target)?
    fn is_self(&self, g: &Inner, person: &PersonRef) -> bool {
        self.asker
            .as_ref()
            .is_some_and(|a| g.target_id(a) == g.target_id(person))
    }

    pub(super) fn do_consents_of(&self, person: &PersonRef) -> Result<Vec<Consent>> {
        let g = self.lock();
        if self.blocks_ai(&g) || !(self.is_self(&g, person) || self.rights.read) {
            return forbidden("consent.read");
        }
        let end = g.target_id(person);
        Ok(g.consents
            .iter()
            .filter(|c| g.target_id(&c.entry.person) == end)
            .cloned()
            .collect())
    }

    pub(super) fn do_check(&self, query: &ConsentQuery) -> Result<Option<ObjectId>> {
        let g = self.lock();
        if self.blocks_ai(&g) || !self.rights.check {
            return forbidden("consent.check");
        }
        Ok(g.active_match(query))
    }

    pub(super) fn do_record(&self, entry: &ConsentEntry) -> Result<Consent> {
        let mut g = self.lock();
        let login = self.asker.clone().or_else(|| {
            g.bad(Defect::RecordWithoutLogin)
                .then(|| entry.person.clone())
        });
        let Some(by) = login.filter(|_| !self.blocks_ai(&g)) else {
            return forbidden("consent.write");
        };
        if !(self.is_self(&g, &entry.person)
            || self.rights.write
            || g.bad(Defect::RecordWithoutRight))
        {
            return forbidden("consent.write");
        }
        if !matches!(g.resolve(&entry.person, false), Resolved::Found(_)) {
            return invalid("person", "not_found");
        }
        if entry.purpose == Purpose::AiAssistance
            && entry.scope.is_none()
            && !g.bad(Defect::AiScopeOptional)
        {
            return invalid("scope", "required");
        }
        if g.bad(Defect::DropDuplicate) {
            let same = ConsentQuery {
                person: entry.person.clone(),
                purpose: entry.purpose.clone(),
                scope: entry.scope.clone(),
                recipient: entry.recipient.clone(),
            };
            if let Some(c) = g
                .active_match(&same)
                .and_then(|id| g.consents.iter().find(|c| c.id == id))
            {
                return Ok(c.clone());
            }
        }
        let at = g.now();
        let consent = Consent {
            id: ObjectId(uuid7(at.millis()).to_be_bytes()),
            rev: Revision(1),
            entry: entry.clone(),
            recorded_by: by,
            recorded_at: at,
            withdrawn: None,
            retain_until: None,
        };
        g.consents.push(consent.clone());
        Ok(consent)
    }

    pub(super) fn do_withdraw(&self, id: ObjectId, w: &Withdrawal) -> Result<Consent> {
        let mut g = self.lock();
        let Some(by) = self.asker.clone().filter(|_| !self.blocks_ai(&g)) else {
            return forbidden("consent.write");
        };
        let Some(pos) = g.consents.iter().position(|c| c.id == id) else {
            return Err(Error::UnknownConsent(id));
        };
        let own = self.is_self(&g, &g.consents[pos].entry.person);
        // Not visible (neither own nor readable) looks like unknown; only a visible one without right is Forbidden.
        if !(own || self.rights.read) {
            return if g.bad(Defect::WithdrawForbiddenFirst) {
                forbidden("consent.write")
            } else {
                Err(Error::UnknownConsent(id))
            };
        }
        if !(own || self.rights.write) {
            return forbidden("consent.write");
        }
        if g.consents[pos].withdrawn.is_some() && !g.bad(Defect::WithdrawNotIdempotent) {
            return Ok(g.consents[pos].clone());
        }
        let at = g.now();
        let from = if g.bad(Defect::RetainFromRecorded) {
            When::At(at)
        } else {
            w.at
        };
        let c = &mut g.consents[pos];
        c.withdrawn = Some(Withdrawn {
            withdrawal: w.clone(),
            recorded_by: by,
            recorded_at: at,
        });
        c.retain_until = retain_until(&from);
        c.rev = Revision(c.rev.0 + 1);
        let done = c.clone();
        if g.bad(Defect::WithdrawDeletes) {
            g.consents.remove(pos);
        }
        Ok(done)
    }

    /// An AI may not use the consent methods (unless the mutation test lets it).
    fn blocks_ai(&self, g: &Inner) -> bool {
        self.asker_is_ai(g) && !g.bad(Defect::AiMayConsent)
    }

    /// Is the asker an AI? The mark sticks to the entry whatever its state (shared, unshared, deleted) and it is
    /// looked up at the asker's own id and at the end of its merge chain.
    pub(super) fn asker_is_ai(&self, g: &Inner) -> bool {
        self.asker.as_ref().is_some_and(|a| {
            let end = g.target_id(a);
            let merged = !g.bad(Defect::AiFlagNoMergeTarget);
            let own = !g.bad(Defect::AiFlagOnlyMergeTarget);
            g.entries.iter().any(|e| {
                e.ai && ((own && e.person.id == a.id) || (merged && e.person.id == end))
                    && !(g.bad(Defect::AiFlagNeedsShared) && e.status == Status::NotShared)
            })
        })
    }
}
