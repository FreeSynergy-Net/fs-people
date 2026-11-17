//! A card as a form.
//!
//! Pattern: Adapter — the fields the form edits (name, nicknames, e-mails, phones, organisations,
//! titles, addresses, links, notes, birthday, sensitive categories) as neutral labelled text fields,
//! the kinds of entries from one table (`groups.rs`). A field's key names what it belongs to on the
//! card (`emails/e1`), so a save finds the very entry again and keeps everything else about it. An
//! entry that has a context says it in the label (`E-mail (work)`); a new e-mail address or phone
//! number can be given one by adding it in the group of that context (`emails.work`).

use fs_components_workbenches::people::{DraftField, DraftGroup, PersonDraft};
use fs_people_core::Card;

use super::groups::{ALL, CONTEXTS, Kind};
use super::{birthday, sensitive};
use crate::words::field;

/// The key of the name field.
pub(super) const NAME: &str = "name/full";

/// The name as the form shows it: `name.full`, else the joined parts. Not the display name, which
/// falls back on an e-mail address.
pub(super) fn written_name(card: &Card) -> String {
    let Some(name) = &card.name else {
        return String::new();
    };
    if let Some(full) = name.full.as_deref().filter(|f| !f.trim().is_empty()) {
        return full.to_string();
    }
    let parts: Vec<&str> = name
        .components
        .iter()
        .filter(|c| c.kind != "separator" && !c.value.trim().is_empty())
        .map(|c| c.value.trim())
        .collect();
    parts.join(" ")
}

/// The group of `kind`: every entry of the card, then room to add one.
fn group_of(kind: Kind, card: &Card) -> DraftGroup {
    let base = kind.base();
    let fields = kind
        .entries(card)
        .into_iter()
        .map(|e| {
            let label = if e.contexts.is_empty() {
                kind.label()
            } else {
                field::in_context(&kind.label(), &e.contexts.join(", "))
            };
            DraftField::new(format!("{base}/{}", e.id), label, e.text).with_check(kind.check())
        })
        .collect();
    DraftGroup::new(base, kind.label(), true, fields).with_check(kind.check())
}

/// The groups for a new e-mail address or phone number with a context: empty, to be added to.
fn context_groups(kind: Kind) -> Vec<DraftGroup> {
    if !kind.has_contexts() {
        return Vec::new();
    }
    CONTEXTS
        .iter()
        .map(|c| {
            let label = field::in_context(&kind.label(), c);
            DraftGroup::new(format!("{}.{c}", kind.base()), label, true, Vec::new())
                .with_check(kind.check())
        })
        .collect()
}

fn birthday_group(text: String) -> DraftGroup {
    DraftGroup::new(
        birthday::GROUP,
        field::birthday(),
        false,
        vec![DraftField::new(birthday::KEY, field::birthday(), text)],
    )
}

/// `card` as a form, marked with `revision`. The roles are the source's to add: they are not on the
/// card.
pub(super) fn draft_of(card: &Card, revision: Option<String>) -> PersonDraft {
    let mut groups = vec![DraftGroup::new(
        "name",
        field::name(),
        false,
        vec![DraftField::new(NAME, field::name(), written_name(card))],
    )];
    for kind in ALL {
        groups.push(group_of(kind, card));
        groups.extend(context_groups(kind));
    }
    groups.push(birthday_group(birthday::written(card)));
    groups.push(sensitive::group_of(card));
    PersonDraft::new(revision, groups)
}

/// The form for a new person: one empty field of each of the usual kinds.
pub(super) fn blank_draft() -> PersonDraft {
    let mut groups = vec![DraftGroup::new(
        "name",
        field::name(),
        false,
        vec![DraftField::new(NAME, field::name(), "")],
    )];
    for kind in ALL {
        let mut group =
            DraftGroup::new(kind.base(), kind.label(), true, Vec::new()).with_check(kind.check());
        if matches!(kind, Kind::Emails | Kind::Phones | Kind::Organizations) {
            let mut one = DraftField::new(format!("{}/+0", kind.base()), kind.label(), "")
                .with_check(kind.check());
            one.added = true;
            group.fields.push(one);
        }
        groups.push(group);
        groups.extend(context_groups(kind));
    }
    groups.push(birthday_group(String::new()));
    groups.push(sensitive::blank_group());
    PersonDraft::new(None, groups)
}
