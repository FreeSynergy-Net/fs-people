//! The kinds of entries a card holds that the form edits as text, one table for all of them.
//!
//! Pattern: Strategy table — each [`Kind`] knows how to read its entries off a card, how to change
//! the text of one, add one and remove one. The draft (`draft.rs`) and the write-back (`apply.rs`)
//! walk the same table, so what the form shows and what a save does cannot drift apart. What an
//! entry holds besides its text (contexts, preference, label, units) is never touched by a change.

use std::collections::BTreeMap;

use fs_components_workbenches::people::FieldCheck;
use fs_people_core::Card;
use fs_people_core::card::{Address, Link, Map, Nickname, Note, Title};

use crate::words::field;

/// A kind of entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Nicknames,
    Emails,
    Phones,
    Organizations,
    Titles,
    Addresses,
    Links,
    Notes,
}

/// The order of the groups in the form.
pub(super) const ALL: [Kind; 8] = [
    Kind::Nicknames,
    Kind::Emails,
    Kind::Phones,
    Kind::Organizations,
    Kind::Titles,
    Kind::Addresses,
    Kind::Links,
    Kind::Notes,
];

/// The contexts a new e-mail address or phone number can be given (JSContact `contexts`).
pub(super) const CONTEXTS: [&str; 2] = ["work", "private"];

/// One entry as the form shows it.
pub(super) struct Entry {
    pub id: String,
    pub text: String,
    /// The contexts it holds, to say them in the label; never changed by the form.
    pub contexts: Vec<String>,
}

fn contexts_of(map: &BTreeMap<String, bool>) -> Vec<String> {
    map.iter()
        .filter(|(_, on)| **on)
        .map(|(c, _)| c.clone())
        .collect()
}

fn entries<T>(items: &Map<T>, pick: impl Fn(&T) -> (String, Vec<String>)) -> Vec<Entry> {
    items
        .iter()
        .map(|(id, item)| {
            let (text, contexts) = pick(item);
            Entry {
                id: id.clone(),
                text,
                contexts,
            }
        })
        .collect()
}

fn fresh<T>(items: &Map<T>, prefix: &str) -> String {
    (1..=u32::MAX)
        .map(|i| format!("{prefix}{i}"))
        .find(|k| !items.contains_key(k))
        .unwrap_or_default()
}

fn address_text(a: &Address) -> String {
    a.full.clone().unwrap_or_else(|| {
        let parts: Vec<&str> = a
            .components
            .iter()
            .filter(|c| c.kind != "separator")
            .map(|c| c.value.as_str())
            .collect();
        parts.join(a.default_separator.as_deref().unwrap_or(" "))
    })
}

fn context_map(context: Option<&str>) -> BTreeMap<String, bool> {
    context.map(|c| (c.to_owned(), true)).into_iter().collect()
}

impl Kind {
    /// The key of the group, and the first part of the key of each of its fields.
    pub(super) fn base(self) -> &'static str {
        match self {
            Kind::Nicknames => "nicknames",
            Kind::Emails => "emails",
            Kind::Phones => "phones",
            Kind::Organizations => "organizations",
            Kind::Titles => "titles",
            Kind::Addresses => "addresses",
            Kind::Links => "links",
            Kind::Notes => "notes",
        }
    }

    pub(super) fn of_base(base: &str) -> Option<Self> {
        ALL.into_iter().find(|k| k.base() == base)
    }

    /// Whether a new entry can be given a context (a group of its own for each).
    pub(super) fn has_contexts(self) -> bool {
        matches!(self, Kind::Emails | Kind::Phones)
    }

    pub(super) fn label(self) -> String {
        match self {
            Kind::Nicknames => field::nickname(),
            Kind::Emails => field::email(),
            Kind::Phones => field::phone(),
            Kind::Organizations => field::organisation(),
            Kind::Titles => field::job(),
            Kind::Addresses => field::address(),
            Kind::Links => field::link(),
            Kind::Notes => field::note(),
        }
    }

    pub(super) fn check(self) -> FieldCheck {
        match self {
            Kind::Emails => FieldCheck::Email,
            Kind::Phones => FieldCheck::Phone,
            _ => FieldCheck::Text,
        }
    }

    /// The entries of `card`, in the order of their ids.
    pub(super) fn entries(self, card: &Card) -> Vec<Entry> {
        match self {
            Kind::Nicknames => entries(&card.nicknames, |n| {
                (n.name.clone(), contexts_of(&n.contexts))
            }),
            Kind::Emails => entries(&card.emails, |e| {
                (e.address.clone(), contexts_of(&e.contexts))
            }),
            Kind::Phones => entries(&card.phones, |p| {
                (p.number.clone(), contexts_of(&p.contexts))
            }),
            Kind::Organizations => entries(&card.organizations, |o| {
                (o.name.clone().unwrap_or_default(), contexts_of(&o.contexts))
            }),
            Kind::Titles => entries(&card.titles, |t| (t.name.clone(), Vec::new())),
            Kind::Addresses => entries(&card.addresses, |a| {
                (address_text(a), contexts_of(&a.contexts))
            }),
            Kind::Links => entries(&card.links, |l| (l.uri.clone(), contexts_of(&l.contexts))),
            Kind::Notes => entries(&card.notes, |n| (n.note.clone(), Vec::new())),
        }
    }

    /// Change the text of the entry `id`; `false` if there is none.
    pub(super) fn set(self, card: &mut Card, id: &str, text: &str) -> bool {
        let text = text.to_owned();
        match self {
            Kind::Nicknames => card.nicknames.get_mut(id).map(|n| n.name = text),
            Kind::Emails => card.emails.get_mut(id).map(|e| e.address = text),
            Kind::Phones => card.phones.get_mut(id).map(|p| p.number = text),
            Kind::Organizations => card.organizations.get_mut(id).map(|o| o.name = Some(text)),
            Kind::Titles => card.titles.get_mut(id).map(|t| t.name = text),
            Kind::Addresses => card.addresses.get_mut(id).map(|a| {
                // The parts described the old text; left in place they would keep it alive.
                a.components.clear();
                a.is_ordered = None;
                a.default_separator = None;
                a.full = Some(text);
            }),
            Kind::Links => card.links.get_mut(id).map(|l| l.uri = text),
            Kind::Notes => card.notes.get_mut(id).map(|n| n.note = text),
        }
        .is_some()
    }

    /// Add an entry holding `text`, with `context` where the kind has them.
    pub(super) fn add(self, card: &mut Card, text: &str, context: Option<&str>) {
        let text = text.to_owned();
        let contexts = context_map(context);
        match self {
            Kind::Nicknames => {
                let id = fresh(&card.nicknames, "n");
                let item = Nickname {
                    type_: Some("Nickname".into()),
                    name: text,
                    ..Nickname::default()
                };
                card.nicknames.insert(id, item);
            }
            Kind::Emails => {
                let id = card.add_email(text);
                if let Some(e) = card.emails.get_mut(&id) {
                    e.contexts = contexts;
                }
            }
            Kind::Phones => {
                let id = card.add_phone(text);
                if let Some(p) = card.phones.get_mut(&id) {
                    p.contexts = contexts;
                }
            }
            Kind::Organizations => {
                card.add_organization(text);
            }
            Kind::Titles => {
                let id = fresh(&card.titles, "t");
                let item = Title {
                    type_: Some("Title".into()),
                    name: text,
                    ..Title::default()
                };
                card.titles.insert(id, item);
            }
            Kind::Addresses => {
                let id = fresh(&card.addresses, "a");
                let item = Address {
                    type_: Some("Address".into()),
                    full: Some(text),
                    ..Address::default()
                };
                card.addresses.insert(id, item);
            }
            Kind::Links => {
                let id = fresh(&card.links, "l");
                let item = Link {
                    type_: Some("Link".into()),
                    uri: text,
                    ..Link::default()
                };
                card.links.insert(id, item);
            }
            Kind::Notes => {
                let id = fresh(&card.notes, "n");
                let item = Note {
                    type_: Some("Note".into()),
                    note: text,
                    ..Note::default()
                };
                card.notes.insert(id, item);
            }
        }
    }

    /// Remove the entry `id`. An organisation that has units only loses its name, and a title no
    /// longer points at an organisation that went.
    pub(super) fn remove(self, card: &mut Card, id: &str) {
        match self {
            Kind::Nicknames => drop(card.nicknames.remove(id)),
            Kind::Emails => drop(card.emails.remove(id)),
            Kind::Phones => drop(card.phones.remove(id)),
            Kind::Organizations => match card.organizations.get_mut(id) {
                Some(organisation) if !organisation.units.is_empty() => organisation.name = None,
                _ => {
                    card.organizations.remove(id);
                    for title in card.titles.values_mut() {
                        if title.organization_id.as_deref() == Some(id) {
                            title.organization_id = None;
                        }
                    }
                }
            },
            Kind::Titles => drop(card.titles.remove(id)),
            Kind::Addresses => drop(card.addresses.remove(id)),
            Kind::Links => drop(card.links.remove(id)),
            Kind::Notes => drop(card.notes.remove(id)),
        }
    }
}
