//! The roles of a person as a group of the form, and what a save does with them.
//!
//! Pattern: Command — a role is not part of the card (ADR 0029): it is an assignment of its own, so
//! the form's role fields become `assign_role` and `unassign_role` of the `RoleBook`, run after the
//! card is written and checked before it (the role exists, reaches as far as the text says, is not
//! the AI role). A field reads `manager @ <group address>` for a role within a group and `name` for a
//! personal one. A role that fails after the card was written is said as such: the card is saved.

use fs_components_workbenches::SourceError;
use fs_components_workbenches::people::{DraftField, DraftGroup, PersonDraft};
use fs_objects::ObjectId;
use fs_people_core::{ROLE_BASE, RoleAssignment, RoleBook, RoleScope, is_ai_role};
use serde_json::Map;
use tokio::runtime::Handle;

use crate::words::field;
use crate::words_edit as say;

/// The key of the group.
pub(super) const GROUP: &str = "roles";

/// The short name of a role: what the form shows and takes.
fn short(role: &str) -> &str {
    role.strip_prefix(ROLE_BASE).unwrap_or(role)
}

fn written(assignment: &RoleAssignment) -> String {
    match &assignment.scope {
        Some(group) => format!("{} @ {group}", short(&assignment.role)),
        None => short(&assignment.role).to_owned(),
    }
}

/// The group of the form: what `person` holds, except the AI marking (set by the program that holds
/// the AI, shown at the top of the person).
pub(super) fn group_of(held: &[RoleAssignment]) -> DraftGroup {
    let fields = held
        .iter()
        .filter(|a| !is_ai_role(&a.role))
        .map(|a| DraftField::new(format!("{GROUP}/{}", a.id), field::role_field(), written(a)))
        .collect();
    DraftGroup::new(GROUP, field::role_field(), true, fields)
}

/// A role and where it is held, as typed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Held {
    pub role: String,
    pub group: Option<String>,
}

/// Read `manager @ urn:group:1` or `lead`.
pub(super) fn parse(text: &str) -> Result<Held, SourceError> {
    let (role, group) = match text.split_once('@') {
        Some((role, group)) => (role.trim(), Some(group.trim())),
        None => (text.trim(), None),
    };
    if role.is_empty() || group.is_some_and(str::is_empty) {
        return Err(SourceError::failed(say::role_empty()));
    }
    Ok(Held {
        role: short(role).to_owned(),
        group: group.map(str::to_owned),
    })
}

/// One thing to do to the roles.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Change {
    Give(Held),
    End(ObjectId),
    /// End `ObjectId`, then give the new one.
    Replace(ObjectId, Held),
}

/// What the form changed about the roles.
///
/// # Errors
/// [`SourceError`] for a text that is no role, and for the AI role.
pub(super) fn plan(draft: &PersonDraft) -> Result<Vec<Change>, SourceError> {
    let mut changes = Vec::new();
    for f in draft.fields().filter(|f| f.changed()) {
        let Some(("roles", id)) = f.key.split_once('/') else {
            continue;
        };
        let text = f.value.trim();
        let old = (!f.added).then(|| id.parse::<ObjectId>().ok()).flatten();
        let new = if text.is_empty() {
            None
        } else {
            let held = parse(text)?;
            if is_ai_role(&held.role) {
                return Err(SourceError::failed(say::role_ai()));
            }
            Some(held)
        };
        match (old, new) {
            (Some(old), Some(new)) => changes.push(Change::Replace(old, new)),
            (Some(old), None) => changes.push(Change::End(old)),
            (None, Some(new)) => changes.push(Change::Give(new)),
            (None, None) => {}
        }
    }
    Ok(changes)
}

fn given(changes: &[Change]) -> impl Iterator<Item = &Held> {
    changes.iter().filter_map(|c| match c {
        Change::Give(h) | Change::Replace(_, h) => Some(h),
        Change::End(_) => None,
    })
}

/// Everything that can be known before anything is written: the role exists, and the group is named
/// where the role needs one (and is not where it has none).
///
/// # Errors
/// [`SourceError`] with the sentence for the first role that cannot be given.
pub(super) fn check(
    changes: &[Change],
    roles: &dyn RoleBook,
    runtime: &Handle,
) -> Result<(), SourceError> {
    for held in given(changes) {
        let found = runtime
            .block_on(roles.role(&held.role))
            .map_err(|e| SourceError::failed(e.to_string()))?
            .ok_or_else(|| SourceError::failed(say::role_unknown(&held.role)))?;
        match (found.scope, &held.group) {
            (RoleScope::Group, None) => {
                return Err(SourceError::failed(say::role_group(&held.role)));
            }
            (RoleScope::Personal, Some(_)) => {
                return Err(SourceError::failed(say::role_personal(&held.role)));
            }
            _ => {}
        }
    }
    Ok(())
}

/// Do the changes. The first that fails stops the rest and is said as a role that was not changed
/// while the card is saved.
///
/// # Errors
/// [`SourceError`] naming the role.
pub(super) fn run(
    changes: &[Change],
    person: ObjectId,
    roles: &dyn RoleBook,
    runtime: &Handle,
) -> Result<(), SourceError> {
    let partial = |role: &str, e: &dyn std::fmt::Display| {
        SourceError::failed(say::role_partial(role, &e.to_string()))
    };
    for change in changes {
        let give = |held: &Held| {
            // Give first: a role that cannot be given leaves the one it should replace in place.
            runtime
                .block_on(roles.assign_role(person, &held.role, held.group.as_deref(), Map::new()))
                .map_err(|e| partial(&held.role, &e))
        };
        match change {
            Change::Give(held) => {
                give(held)?;
            }
            Change::End(id) => end(*id, roles, runtime)?,
            Change::Replace(old, held) => {
                let new = give(held)?;
                if new.id != *old {
                    end(*old, roles, runtime)?;
                }
            }
        }
    }
    Ok(())
}

fn end(id: ObjectId, roles: &dyn RoleBook, runtime: &Handle) -> Result<(), SourceError> {
    runtime
        .block_on(roles.unassign_role(id))
        .map_err(|e| SourceError::failed(say::role_partial(&id.to_string(), &e.to_string())))
}
