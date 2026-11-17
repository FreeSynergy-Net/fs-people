//! Setting and lifting the sensitive marking at card level from the form.
//!
//! Pattern: Command over the card-level list `freesynergy.net:sensitive` (ADR 0029). A category is
//! added or lifted only by the person typing it; a marking on a part of the card (an e-mail, a name
//! part) is not listed here and is never lifted by this form. Lifting a category that is also marked
//! on such a part is refused and nothing is changed. The mark is local to this program: the contract
//! `person_directory` carries no sensitive state yet (it is connected with v2).

use fs_components_workbenches::people::{DraftField, DraftGroup};
use fs_people_core::Card;
use fs_people_core::card::SENSITIVE_KEY;
use serde_json::Value;

use crate::words::field;
use crate::words_edit;

/// The key of the group.
pub(super) const GROUP: &str = "sensitive";

/// The categories named at card level, in the order they stand.
pub(super) fn level_categories(card: &Card) -> Vec<String> {
    match card.extra.get(SENSITIVE_KEY) {
        Some(Value::Array(list)) => list
            .iter()
            .filter_map(Value::as_str)
            .filter(|c| !c.trim().is_empty())
            .map(str::to_owned)
            .collect(),
        Some(Value::String(one)) if !one.trim().is_empty() => vec![one.clone()],
        _ => Vec::new(),
    }
}

/// The group of the form: each category at card level, and room to add one.
pub(super) fn group_of(card: &Card) -> DraftGroup {
    let fields = level_categories(card)
        .into_iter()
        .map(|c| DraftField::new(format!("{GROUP}/{c}"), field::sensitive_group(), c))
        .collect();
    DraftGroup::new(GROUP, field::sensitive_group(), true, fields)
}

/// The group of the form for a new person: nothing marked yet.
pub(super) fn blank_group() -> DraftGroup {
    DraftGroup::new(GROUP, field::sensitive_group(), true, Vec::new())
}

/// What the form changed: categories to lift and to set.
#[derive(Default)]
pub(super) struct Changes {
    pub lifted: Vec<String>,
    pub set: Vec<String>,
}

impl Changes {
    /// Note the change of one field of the group: `id` is what its key says (empty for an added one).
    ///
    /// # Errors
    /// The sentence to show for a category that has spaces.
    pub(super) fn note(&mut self, id: &str, value: &str) -> Result<(), String> {
        let value = value.trim();
        if value.chars().any(char::is_whitespace) {
            return Err(words_edit::category());
        }
        if !id.is_empty() && !id.starts_with('+') {
            self.lifted.push(id.to_owned());
        }
        if !value.is_empty() {
            self.set.push(value.to_owned());
        }
        Ok(())
    }

    pub(super) fn is_empty(&self) -> bool {
        self.lifted.is_empty() && self.set.is_empty()
    }

    /// Put the changes on the card level list. A category that was lifted and set again stays.
    pub(super) fn apply(&self, card: &mut Card) {
        let mut items: Vec<Value> = match card.extra.get(SENSITIVE_KEY) {
            Some(Value::Array(list)) => list.clone(),
            Some(other) => vec![other.clone()],
            None => Vec::new(),
        };
        items.retain(|v| {
            v.as_str().is_none_or(|c| {
                !self.lifted.iter().any(|l| l == c) || self.set.iter().any(|s| s == c)
            })
        });
        for category in &self.set {
            if !items.iter().any(|v| v.as_str() == Some(category)) {
                items.push(Value::String(category.clone()));
            }
        }
        if items.is_empty() {
            card.extra.remove(SENSITIVE_KEY);
        } else {
            card.extra.insert(SENSITIVE_KEY.into(), Value::Array(items));
        }
    }

    /// A lifted category that the card still carries somewhere else.
    pub(super) fn still_marked(&self, card: &Card) -> Option<String> {
        let now = card.sensitive_categories();
        self.lifted
            .iter()
            .filter(|c| !self.set.contains(c))
            .find(|c| now.contains(c))
            .cloned()
    }
}
