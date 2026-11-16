//! `sample()`: the shared test data of the consumers (calendar adapter, tasks, groups): twelve entries that cover
//! every state of `person_directory_v1`.
//!
//! Pattern: Object Mother (one well-known directory, entries addressed by number through `sample_ref`).
//!
//! | n | art | what |
//! | --- | --- | --- |
//! | 1 | Person | Amara Zolenko, confirmed, `mailto:amara@example.org`, the asker, full excerpt |
//! | 2 | Profile | "Amara Z.", linked to 1 (`describes`), same address as 1 |
//! | 3 | Profile | "A. Zolenko", unlinked, same address as 1 (two profiles with one address) |
//! | 4 | Person | "Assistent Sol", `ai` set |
//! | 5 | Group | "Quillon Team" (members live in fs-groups) |
//! | 6 | Organization | "Tamarind Works Ltd" |
//! | 7 | Person | "Brannoc Deleted", deleted (tombstone) |
//! | 8 | Person | "Ilvy Hidden", no longer shared |
//! | 9 | Profile | "Soo-jin M.", merged into 10 |
//! | 10 | Person | "Soo-jin Marchetti", `tel:+49305550123` |
//! | 11 | Profile | "Import Tariq", `Proposed`, `mailto:tariq@example.org` |
//! | 12 | Person | "Tariq Lindqvist", confirmed |

use super::{Entry, FakeDirectory};
use crate::person_directory_v1::{EntryState, ExcerptValue, PersonArt, PersonRef};
use fs_types::{BasicValue, ContactAddress, EmailAddress, Name, Reference, Text};

/// The reference of entry `n` (1 to 12) of [`sample`], with the art of that entry; `None` for any other number.
#[must_use]
pub fn sample_ref(n: u8) -> Option<PersonRef> {
    let art = match n {
        2 | 3 | 9 | 11 => PersonArt::Profile,
        5 => PersonArt::Group,
        6 => PersonArt::Organization,
        1 | 4 | 7 | 8 | 10 | 12 => PersonArt::Person,
        _ => return None,
    };
    let id = Reference::parse(&format!("urn:uuid:00000000-0000-7000-8000-{n:012x}")).ok()?;
    Some(PersonRef {
        id,
        art,
        abzug: None,
    })
}

fn row(n: u8, display: &str, address: Option<&str>) -> Option<Entry> {
    let mut e = Entry::new(sample_ref(n)?, display);
    e.address = address.and_then(|a| ContactAddress::parse(a).ok());
    Some(e)
}

fn amara() -> Option<Entry> {
    let mut e = row(1, "Amara Zolenko", Some("mailto:amara@example.org"))?;
    if let Ok(name) = Name::new("Amara Zolenko", &[]) {
        e.excerpt.push(ExcerptValue::FullName(name));
    }
    if let Ok(t) = Text::parse("Zolenko") {
        e.excerpt.push(ExcerptValue::Surname(t));
    }
    if let Ok(mail) = EmailAddress::parse("amara@example.org") {
        e.excerpt.push(ExcerptValue::Email(mail));
    }
    Some(e)
}

/// A directory with the twelve entries of the table above; the asker is entry 1. Entries 7, 8 and 9 are put into
/// their state after creation, so the change log holds `Deleted`, `NotShared` and `Merged`.
#[must_use]
pub fn sample() -> FakeDirectory {
    let d = FakeDirectory::new().with_asker(sample_ref(1));
    let linked = row(2, "Amara Z.", Some("mailto:amara@example.org")).map(|mut e| {
        e.describes = sample_ref(1);
        e
    });
    let ai = row(4, "Assistent Sol", None).map(|mut e| {
        e.ai = true;
        e
    });
    let proposed = row(11, "Import Tariq", Some("mailto:tariq@example.org")).map(|mut e| {
        e.state = EntryState::Proposed;
        e
    });
    let rows = [
        amara(),
        linked,
        row(3, "A. Zolenko", Some("mailto:amara@example.org")),
        ai,
        row(5, "Quillon Team", None),
        row(6, "Tamarind Works Ltd", None),
        row(7, "Brannoc Deleted", None),
        row(8, "Ilvy Hidden", None),
        row(9, "Soo-jin M.", None),
        row(10, "Soo-jin Marchetti", Some("tel:+49305550123")),
        proposed,
        row(12, "Tariq Lindqvist", None),
    ];
    for e in rows.into_iter().flatten() {
        d.add(e);
    }
    if let (Some(deleted), Some(hidden), Some(from), Some(into)) =
        (sample_ref(7), sample_ref(8), sample_ref(9), sample_ref(10))
    {
        d.delete(&deleted);
        d.unshare(&hidden);
        d.merge(&from, &into);
    }
    d
}
