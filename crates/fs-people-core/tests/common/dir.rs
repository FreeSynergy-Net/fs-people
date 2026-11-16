//! Fixtures of the directory-store tests: people, consents, withdrawals, excerpts and snapshots.
//!
//! Pattern: Object Mother (small builders; a test states only what it varies).

use fs_objects_v5::v5::{Store, new_id};
use fs_people_core::directory_store::{self, DocKind, Snapshot};
use fs_people_directory::{
    Birthday, Consent, ConsentEntry, ConsentOrigin, Date, Excerpt, ExcerptValue, MonthDay, Name,
    ObjectId, PersonArt, PersonRef, Purpose, Reference, Revision, Utc, When, Withdrawal, Withdrawn,
};
use fs_types::BasicValue;

pub fn person(n: u8) -> PersonRef {
    PersonRef {
        id: Reference::parse(&format!(
            "urn:uuid:01927f00-0000-7000-8000-0000000000{n:02x}"
        ))
        .unwrap(),
        art: PersonArt::Person,
        abzug: None,
    }
}

pub fn day(s: &str) -> Date {
    Date::parse(s).unwrap()
}

pub fn id(n: i64) -> ObjectId {
    new_id(n).unwrap()
}

pub fn entry(who: u8) -> ConsentEntry {
    ConsentEntry {
        person: person(who),
        given_by: None,
        purpose: Purpose::Newsletter,
        scope: None,
        recipient: None,
        given: When::At(Utc::from_millis(1_700_000_000_000)),
        origin: ConsentOrigin::ByPerson,
        evidence: None,
        text: None,
    }
}

pub fn consent_of(n: i64, entry: ConsentEntry) -> Consent {
    Consent {
        id: id(n),
        rev: Revision(1),
        entry,
        recorded_by: person(9),
        recorded_at: Utc::from_millis(1_700_000_100_000),
        withdrawn: None,
        retain_until: None,
    }
}

pub fn withdrawal(at: When) -> Withdrawn {
    Withdrawn {
        withdrawal: Withdrawal {
            at,
            origin: ConsentOrigin::Form,
            evidence: None,
        },
        recorded_by: person(9),
        recorded_at: Utc::from_millis(1_700_000_200_000),
    }
}

pub async fn store() -> Store {
    directory_store::open(":memory:", 0).await.unwrap()
}

pub fn excerpt(consent: Option<ObjectId>) -> Excerpt {
    Excerpt {
        person: person(1),
        source: Reference::parse("urn:uuid:01927f00-0000-7000-8000-0000000000b1").unwrap(),
        captured: Utc::from_millis(1_700_000_300_000),
        purpose: Purpose::Newsletter,
        consent,
        values: vec![
            ExcerptValue::FullName(Name::new("Ada Lovelace", &[]).unwrap()),
            ExcerptValue::Birthday(Birthday::Date(day("1815-12-10"))),
            ExcerptValue::BirthMonthDay(MonthDay::new(12, 10).unwrap()),
        ],
        missing: Vec::new(),
    }
}

pub fn snap(n: i64, e: Excerpt) -> Snapshot {
    Snapshot {
        id: id(n),
        doc_kind: DocKind::Invoice,
        issued: day("2026-11-16"),
        keep_until: day("2036-12-31"),
        excerpt: e,
    }
}
