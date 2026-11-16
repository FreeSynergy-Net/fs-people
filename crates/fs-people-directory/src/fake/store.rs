//! Rows and ordering rules of the in-memory fake.
//!
//! Pattern: Value Object (rows) with pure helper functions for the rules of `person_directory_v1`.

use crate::person_directory_v1::{
    ContactAddress, EntryState, ExcerptValue, FileRef, PersonArt, PersonRef, PersonView, Revision,
    Utc,
};
use std::cmp::Ordering;

/// What happened to a row (the fake keeps deleted and unshared rows, like a trash).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Active,
    /// Tombstone since this instant.
    Deleted(Utc),
    NotShared,
}

/// One row of the fake directory.
#[derive(Clone, Debug)]
pub struct Entry {
    pub person: PersonRef,
    pub display: String,
    /// The display text is only the address (no name: the last stage of ADR 0032); an AI sees no name then.
    pub from_address: bool,
    pub sort_key: String,
    pub address: Option<ContactAddress>,
    pub state: EntryState,
    pub ai: bool,
    pub describes: Option<PersonRef>,
    pub avatar: Option<FileRef>,
    pub status: Status,
    /// Set after a merge: the reference stays valid and resolves to `into`.
    pub merged_into: Option<PersonRef>,
    /// What `excerpt` may hand out.
    pub excerpt: Vec<ExcerptValue>,
    pub created: Utc,
    pub rev: u64,
}

impl Entry {
    /// A confirmed, active entry with a display name; the sort key is the lower-case name.
    #[must_use]
    pub fn new(person: PersonRef, display: &str) -> Self {
        Self {
            person,
            display: display.to_owned(),
            from_address: false,
            sort_key: display.to_lowercase(),
            address: None,
            state: EntryState::Confirmed,
            ai: false,
            describes: None,
            avatar: None,
            status: Status::Active,
            merged_into: None,
            excerpt: Vec::new(),
            created: Utc::from_millis(0),
            rev: 1,
        }
    }

    /// The view of this row for a human asker (`abzug` is always `None`).
    #[must_use]
    pub fn view(&self) -> PersonView {
        self.view_for(false)
    }

    /// The view for an asker that is an AI or not: an AI gets no address and no address stage in `display` and
    /// `sort_key` (empty then; the display shows a translated placeholder).
    #[must_use]
    pub fn view_for(&self, ai_asker: bool) -> PersonView {
        self.view_with(ai_asker, ai_asker)
    }

    /// The view with the address and the address-only display hidden separately (for the mutation tests).
    #[must_use]
    pub fn view_with(&self, hide_address: bool, hide_name: bool) -> PersonView {
        let hide = hide_name && self.from_address;
        PersonView {
            person: PersonRef {
                id: self.person.id.clone(),
                art: self.person.art,
                abzug: None,
            },
            display: if hide {
                String::new()
            } else {
                self.display.clone()
            },
            sort_key: if hide {
                String::new()
            } else {
                self.sort_key.clone()
            },
            address: if hide_address {
                None
            } else {
                self.address.clone()
            },
            state: self.state,
            ai: self.ai,
            describes: self.describes.clone(),
            avatar: self.avatar.clone(),
            rev: Revision(self.rev),
        }
    }
}

/// Comparison rule of `ContactAddress`: scheme and, for `mailto`, the domain without case; the rest as written.
#[must_use]
pub fn same_address(a: &ContactAddress, b: &ContactAddress) -> bool {
    normal(a) == normal(b)
}

fn normal(a: &ContactAddress) -> String {
    let text = a.canonical();
    let Some((scheme, rest)) = text.split_once(':') else {
        return text;
    };
    let scheme = scheme.to_ascii_lowercase();
    if scheme == "mailto"
        && let Some((local, domain)) = rest.rsplit_once('@')
    {
        return format!("{scheme}:{local}@{}", domain.to_ascii_lowercase());
    }
    if scheme == "tel" {
        // RFC 3966 visual separators do not count; the digits are compared (E.164).
        let digits: String = rest.chars().filter(|c| !"-.() ".contains(*c)).collect();
        return format!("{scheme}:{digits}");
    }
    format!("{scheme}:{rest}")
}

fn art_rank(art: PersonArt) -> u8 {
    match art {
        PersonArt::Person => 0,
        PersonArt::Profile => 1,
        _ => 2,
    }
}

/// Order of `find_by_address`: `Confirmed` before `Proposed`, `Person` before `Profile`, then oldest first.
#[must_use]
pub fn find_order(a: &(usize, &Entry), b: &(usize, &Entry)) -> Ordering {
    let state = |e: &Entry| u8::from(e.state == EntryState::Proposed);
    state(a.1)
        .cmp(&state(b.1))
        .then(art_rank(a.1.person.art).cmp(&art_rank(b.1.person.art)))
        .then(a.1.created.cmp(&b.1.created))
        .then(a.0.cmp(&b.0))
}

/// Rank of a suggestion for the typed text: exact, prefix, name part, address (lower is better).
#[must_use]
/// An AI asker never matches on addresses, nor on a display that is only the address.
pub fn suggest_rank(entry: &Entry, text: &str, ai_asker: bool) -> Option<(u8, bool)> {
    if text.is_empty() {
        return Some((0, true));
    }
    let needle = text.to_lowercase();
    let name = if ai_asker && entry.from_address {
        String::new()
    } else {
        entry.display.to_lowercase()
    };
    if name == needle {
        Some((0, true))
    } else if name.starts_with(&needle) {
        Some((1, true))
    } else if name.contains(&needle) {
        Some((2, true))
    } else if !ai_asker
        && entry
            .address
            .as_ref()
            .is_some_and(|a| a.canonical().to_lowercase().contains(&needle))
    {
        Some((3, false))
    } else {
        None
    }
}
