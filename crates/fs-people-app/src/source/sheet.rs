//! A card as neutral sections of labelled fields, to be read.
//!
//! Pattern: Adapter — the JSContact card and its roles in the workbench's terms. Marks (an AI,
//! sensitive data) stand out at the top (ADR 0029, 0054). What the card holds that is not shown is
//! named, so nobody thinks it is gone.

use fs_components_workbenches::people::{PersonField, PersonSection, PersonSheet};
use fs_objects::StoredObject;
use fs_people_core::{Card, Error, RoleBook, is_ai_role};
use tokio::runtime::Handle;

use super::marks::is_marked;
use crate::words::field;

/// The e-mail to show in a row: the preferred one, else the first.
pub(super) fn first_email(card: &Card) -> String {
    card.emails
        .values()
        .min_by_key(|e| e.pref.unwrap_or(u32::MAX))
        .map(|e| e.address.clone())
        .unwrap_or_default()
}

/// The sheet of `card`, stored as `object`.
pub(super) fn sheet_of(
    card: &Card,
    object: &StoredObject,
    roles: &dyn RoleBook,
    runtime: &Handle,
) -> PersonSheet {
    let (role_section, is_ai) = role_section(object, roles, runtime);
    let sections = [
        contact(card),
        work(card),
        places(card),
        about(card),
        role_section,
        kept(card),
    ];
    let mut sheet = PersonSheet::new(
        card.display_name(),
        sections
            .into_iter()
            .filter(|s| !s.fields.is_empty())
            .collect(),
    );
    if is_ai {
        sheet.marks.push(field::ai());
    }
    sheet.marks.extend(sensitive_mark(card, object));
    if let Some(photo) = photo_of(card) {
        sheet = sheet.with_image(photo);
    }
    sheet
}

/// The picture of the card: its first `media` entry of kind `photo` (RFC 9553 §2.6.4) whose `uri` is
/// a `data:image/` URI. A web address is **not** fetched: opening a person must not make the program
/// call out (and tell a server who is looked at); a local file is not read for the same reason.
fn photo_of(card: &Card) -> Option<String> {
    card.media
        .values()
        .filter(|m| m.kind.as_deref() == Some("photo"))
        .map(|m| m.uri.as_str())
        .find(|uri| uri.starts_with("data:image/"))
        .map(str::to_string)
}

/// The marking of someone sensitive, with the categories; `None` for everyone else (ADR 0029).
pub(super) fn sensitive_mark(card: &Card, object: &StoredObject) -> Option<String> {
    is_marked(object).then(|| {
        let categories = card.sensitive_categories().join(", ");
        let categories = if categories.is_empty() {
            field::unclear()
        } else {
            categories
        };
        field::sensitive().replacen("{}", &categories, 1)
    })
}

fn contact(card: &Card) -> PersonSection {
    let mails = card
        .emails
        .values()
        .map(|e| PersonField::new(field::email(), &e.address));
    let phones = card
        .phones
        .values()
        .map(|p| PersonField::new(field::phone(), &p.number));
    let links = card
        .links
        .values()
        .map(|l| PersonField::new(field::link(), &l.uri));
    PersonSection::new(field::contact(), mails.chain(phones).chain(links).collect())
}

fn work(card: &Card) -> PersonSection {
    let organisations = card
        .organizations
        .values()
        .filter_map(|o| o.name.as_ref())
        .map(|n| PersonField::new(field::organisation(), n));
    let titles = card
        .titles
        .values()
        .map(|t| PersonField::new(field::job(), &t.name));
    PersonSection::new(field::work(), organisations.chain(titles).collect())
}

fn places(card: &Card) -> PersonSection {
    let lines = card.addresses.values().map(|a| {
        let joined = a
            .components
            .iter()
            .filter(|c| c.kind != "separator")
            .map(|c| c.value.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        PersonField::new(field::address(), a.full.clone().unwrap_or(joined))
    });
    PersonSection::new(field::places(), lines.collect())
}

fn about(card: &Card) -> PersonSection {
    let kind = (card.kind() != "individual").then(|| PersonField::new(field::kind(), card.kind()));
    let nicknames = card
        .nicknames
        .values()
        .map(|n| PersonField::new(field::nickname(), &n.name));
    let notes = card
        .notes
        .values()
        .map(|n| PersonField::new(field::note(), &n.note));
    PersonSection::new(
        field::about(),
        kind.into_iter().chain(nicknames).chain(notes).collect(),
    )
}

/// The roles, read only, and whether one of them marks an AI. A provider without roles shows the
/// section with the reason (`unsupported`, the capability); a failure is shown, not hidden.
fn role_section(
    object: &StoredObject,
    roles: &dyn RoleBook,
    runtime: &Handle,
) -> (PersonSection, bool) {
    let held = match runtime.block_on(roles.roles_of(object.envelope.id)) {
        Ok(held) => held,
        // Never silent: a source without roles says so, with the code and what it cannot do.
        Err(Error::Unsupported(u)) => {
            let why = field::unsupported(u.capability(), "people");
            let fields = vec![PersonField::new(field::role(), why)];
            return (PersonSection::new(field::roles(), fields), false);
        }
        Err(e) => {
            let why = field::unreadable().replacen("{}", &e.to_string(), 1);
            let fields = vec![PersonField::new(field::role(), why)];
            return (PersonSection::new(field::roles(), fields), false);
        }
    };
    let is_ai = held.iter().any(|a| is_ai_role(&a.role));
    let fields = held
        .iter()
        .map(|assignment| {
            let name = runtime
                .block_on(roles.role(&assignment.role))
                .ok()
                .flatten()
                .and_then(|d| {
                    d.names
                        .get("en")
                        .or_else(|| d.names.values().next())
                        .cloned()
                })
                .unwrap_or_else(|| assignment.role.clone());
            PersonField::new(field::role(), name)
        })
        .collect();
    (PersonSection::new(field::roles(), fields), is_ai)
}

/// The names of what the card holds beyond what is shown above.
fn kept(card: &Card) -> PersonSection {
    // `vCard` is the marker of the kept original (ADR 0030), not something the person typed.
    let mut names: Vec<&str> = card
        .extra
        .keys()
        .map(String::as_str)
        .filter(|name| *name != "vCard")
        .collect();
    // Typed since the whole of RFC 9553 is, but not shown in the window (see DESIGN.md).
    let unshown = [
        ("language", card.language.is_some()),
        ("prodId", card.prod_id.is_some()),
        ("speakAs", card.speak_as.is_some()),
        ("onlineServices", !card.online_services.is_empty()),
        ("preferredLanguages", !card.preferred_languages.is_empty()),
        ("calendars", !card.calendars.is_empty()),
        ("schedulingAddresses", !card.scheduling_addresses.is_empty()),
        ("cryptoKeys", !card.crypto_keys.is_empty()),
        ("directories", !card.directories.is_empty()),
        ("media", !card.media.is_empty()),
        ("localizations", !card.localizations.is_empty()),
        ("anniversaries", !card.anniversaries.is_empty()),
        ("personalInfo", !card.personal_info.is_empty()),
    ];
    names.extend(unshown.iter().filter(|(_, held)| *held).map(|(n, _)| *n));
    names.sort_unstable();
    if !card.members.is_empty() {
        names.push("members");
    }
    if !card.related_to.is_empty() {
        names.push("relatedTo");
    }
    if !card.keywords.is_empty() {
        names.push("keywords");
    }
    let fields = if names.is_empty() {
        Vec::new()
    } else {
        vec![PersonField::new(field::other(), names.join(", "))]
    };
    PersonSection::new(field::kept(), fields)
}
