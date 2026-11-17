//! A form written back onto a card.
//!
//! Pattern: Command — only the fields that differ from what the form started with are applied, each
//! to the entry its key names. Everything else on the card stays untouched: unknown properties,
//! contexts, preferences, labels, addresses, notes, name parts of a name that was not changed.
//!
//! Changes come first, additions second, removals last, so an id freed by this edit is never given
//! to something added by it. An emptied field removes that value, and a title no longer points at an
//! organisation that went. A changed name replaces the name's parts as well: left in place they
//! would keep the old name in sorting, search and vCard. An organisation that has units only loses
//! its name when emptied. The sensitive categories of the card never shrink (`marks.rs`).

use fs_components_workbenches::SourceError;
use fs_components_workbenches::people::{DraftField, PersonDraft};
use fs_people_core::Card;
use fs_people_core::card::{Name, SENSITIVE_KEY};

use super::draft::NAME;
use super::groups::{CONTEXTS, Kind};
use super::marks::Marks;
use super::sensitive::{self, Changes};
use super::{birthday, role_edit};
use crate::words_edit as say;

/// What a name may lose together with its text: it only describes that text.
const NAME_ONLY: [&str; 3] = ["sortAs", "phoneticScript", "phoneticSystem"];

/// Put what changed in `draft` onto `card`. Roles are not on the card: the fields of the group
/// `roles` are the adapter's to carry out (`role_edit.rs`) and are skipped here.
///
/// # Errors
/// [`SourceError`] for a field this form does not know (nothing may be dropped silently), for a
/// birthday that is no day, for a category that has spaces or one that is lifted (not possible yet), for a name
/// that holds more than the form shows when it is emptied, and for a card that would be left with
/// nothing to call the person by. The rule is the same for a new card and a stored one.
pub fn apply_draft(card: &mut Card, draft: &PersonDraft) -> Result<(), SourceError> {
    let marks = Marks::of(card);
    let mut removals = Vec::new();
    let mut additions = Vec::new();
    let mut categories = Changes::default();
    for field in draft.fields().filter(|f| f.changed()) {
        let value = field.value.trim();
        if field.key == NAME {
            set_name(card, value)?;
            continue;
        }
        if field.key == birthday::KEY {
            if !value.is_empty() && birthday::parse(value).is_none() {
                return Err(SourceError::failed(say::birthday()));
            }
            if !birthday::set(card, value) {
                return Err(SourceError::failed(say::birthday_place()));
            }
            continue;
        }
        let Some((group, id)) = field.key.split_once('/') else {
            return Err(unknown(field));
        };
        match group {
            role_edit::GROUP => continue,
            sensitive::GROUP => {
                categories.note(id, value).map_err(SourceError::failed)?;
                continue;
            }
            _ => {}
        }
        let (kind, context) = kind_of(group).ok_or_else(|| unknown(field))?;
        if value.is_empty() {
            removals.push((kind, id));
        } else if !kind.set(card, id, value) {
            additions.push((kind, value, context));
        }
    }
    for (kind, value, context) in additions {
        kind.add(card, value, context);
    }
    for (kind, id) in removals {
        kind.remove(card, id);
    }
    if !categories.lifted.is_empty() {
        return Err(SourceError::failed(say::category_lift()));
    }
    categories.apply(card);
    marks.restore(card);
    if has_something_to_call_it(card) {
        Ok(())
    } else {
        Err(SourceError::failed(
            "enter at least a name, an e-mail, a phone or an organisation",
        ))
    }
}

/// The kind a group key names, and the context of a group like `emails.work`.
fn kind_of(group: &str) -> Option<(Kind, Option<&str>)> {
    let (base, context) = match group.split_once('.') {
        Some((base, context)) => (base, Some(context)),
        None => (group, None),
    };
    let kind = Kind::of_base(base)?;
    match context {
        Some(c) if kind.has_contexts() && CONTEXTS.contains(&c) => Some((kind, Some(c))),
        Some(_) => None,
        None => Some((kind, None)),
    }
}

fn unknown(field: &DraftField) -> SourceError {
    SourceError::failed(format!("the form has an unknown field \"{}\"", field.key))
}

fn has_something_to_call_it(card: &Card) -> bool {
    card.name.is_some()
        || !card.nicknames.is_empty()
        || !card.emails.is_empty()
        || !card.phones.is_empty()
        || card
            .organizations
            .values()
            .any(|o| o.name.as_deref().is_some_and(|n| !n.trim().is_empty()))
}

fn set_name(card: &mut Card, value: &str) -> Result<(), SourceError> {
    if value.is_empty() {
        let kept: Vec<&str> = card
            .name
            .iter()
            .flat_map(|n| n.extra.keys())
            .map(String::as_str)
            .filter(|key| !NAME_ONLY.contains(key) && *key != SENSITIVE_KEY)
            .collect();
        if !kept.is_empty() {
            return Err(SourceError::failed(format!(
                "this name holds more than the form shows ({}), so it can be replaced but not emptied",
                kept.join(", ")
            )));
        }
        card.name = None;
        return Ok(());
    }
    let name = card.name.get_or_insert_with(|| Name {
        type_: Some("Name".into()),
        ..Name::default()
    });
    name.components.clear();
    name.is_ordered = None;
    name.default_separator = None;
    name.full = Some(value.to_string());
    Ok(())
}
