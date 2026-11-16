//! The typed tables of the People directory in fs-objects v5 (ADR 0086: primitive column types, no JSON, no blob).
//!
//! Pattern: Value Object (const table specifications; the registry and the `CREATE TABLE` text come from them).
//!
//! `people_consent` is the record of the locked trait (`Consent`: entry, recorder, withdrawal, `retain_until`),
//! `people_snapshot` the frozen excerpt (`person_snapshot` of `docs/v5-entwurf.md` section 3; a domain table
//! because the store's own table of that name does not exist in fs-objects v0.0.3), `people_birthday` the typed
//! birthday of a person (a date with year or a month and a day without).

use fs_objects_v5::v5::spec::Logical::{Date, Enum, Int, Iri, Ref, Text, Utc};
use fs_objects_v5::v5::tables::ART;
use fs_objects_v5::v5::{Col, Table};

/// The words of `Purpose` (the stored form; `Other` carries its IRI in `purpose_other`).
pub const PURPOSES: &[&str] = &[
    "contract",
    "legal_obligation",
    "own_use",
    "photo",
    "publication",
    "newsletter",
    "ai_assistance",
    "disclosure_to_third_party",
    "other",
];
/// The words of `ConsentOrigin`; `erasure` is only valid for a withdrawal.
pub const ORIGINS: &[&str] = &["by_person", "form", "import", "oral", "erasure"];
/// The kinds of document a snapshot is issued for (draft section 3, `doc_kind`; an open set grows by a later step).
pub const DOC_KINDS: &[&str] = &[
    "booking",
    "invoice",
    "receipt",
    "ticket",
    "membership",
    "other",
];

/// The consents and their withdrawals (DSGVO Art. 7, ISO/IEC TS 27560 fields). A withdrawal never deletes the row.
pub const CONSENT: Table = Table {
    name: "people_consent",
    cols: &[
        Col::req("id", Ref),
        Col::req("rev", Int),
        Col::req("person_id", Iri),
        Col::req("person_art", Enum(ART)),
        Col::opt("person_abzug", Ref),
        Col::opt("given_by_id", Iri),
        Col::opt("given_by_art", Enum(ART)),
        Col::opt("given_by_abzug", Ref),
        Col::req("purpose", Enum(PURPOSES)),
        Col::opt("purpose_other", Iri),
        Col::opt("scope", Iri),
        Col::opt("recipient_id", Iri),
        Col::opt("recipient_art", Enum(ART)),
        Col::opt("recipient_abzug", Ref),
        Col::opt("given_at", Utc),
        Col::opt("given_on", Date),
        Col::req("origin", Enum(ORIGINS)),
        Col::opt("evidence", Iri),
        Col::opt("text", Iri),
        Col::req("recorded_by_id", Iri),
        Col::req("recorded_by_art", Enum(ART)),
        Col::opt("recorded_by_abzug", Ref),
        Col::req("recorded_at", Utc),
        Col::opt("withdrawn_at", Utc),
        Col::opt("withdrawn_on", Date),
        Col::opt("withdrawn_origin", Enum(ORIGINS)),
        Col::opt("withdrawn_evidence", Iri),
        Col::opt("withdrawn_by_id", Iri),
        Col::opt("withdrawn_by_art", Enum(ART)),
        Col::opt("withdrawn_by_abzug", Ref),
        Col::opt("withdrawn_recorded_at", Utc),
        Col::opt("retain_until", Date),
    ],
    constraints: &[
        "PRIMARY KEY (id)",
        "CHECK (rev >= 1)",
        "CHECK ((given_by_id IS NULL) = (given_by_art IS NULL))",
        "CHECK ((recipient_id IS NULL) = (recipient_art IS NULL))",
        "CHECK ((purpose = 'other') = (purpose_other IS NOT NULL))",
        "CHECK (origin <> 'erasure')",
        "CHECK ((given_at IS NULL) <> (given_on IS NULL))",
        "CHECK (((withdrawn_at IS NULL) <> (withdrawn_on IS NULL)) = (withdrawn_origin IS NOT NULL))",
        "CHECK ((withdrawn_recorded_at IS NULL) = (withdrawn_origin IS NULL))",
        "CHECK ((retain_until IS NULL) = (withdrawn_origin IS NULL))",
        "CHECK ((withdrawn_by_id IS NULL) = (withdrawn_origin IS NULL) AND (withdrawn_by_art IS NULL) = (withdrawn_origin IS NULL))",
    ],
    indexes: &[
        "CREATE INDEX IF NOT EXISTS people_consent_person ON people_consent (person_id, purpose)",
    ],
};

/// The frozen excerpt of a person for one purpose (ADR 0085 section 3): insert-only, there is no update path.
/// `consent` names the consent that carries it (no foreign key: the snapshot outlives the consent's proof).
pub const SNAPSHOT: Table = Table {
    name: "people_snapshot",
    cols: &[
        Col::req("id", Ref),
        Col::req("person_id", Iri),
        Col::req("person_art", Enum(ART)),
        Col::req("captured", Utc),
        Col::req("source", Iri),
        Col::req("doc_kind", Enum(DOC_KINDS)),
        Col::req("issued", Date),
        Col::req("keep_until", Date),
        Col::req("purpose", Enum(PURPOSES)),
        Col::opt("purpose_other", Iri),
        Col::opt("consent", Ref),
        Col::opt("full_name", Text),
        Col::opt("given_name", Text),
        Col::opt("surname", Text),
        Col::opt("organization", Text),
        Col::opt("postal_full", Text),
        Col::opt("postal_country", Text),
        Col::opt("email", Text),
        Col::opt("phone", Text),
        Col::opt("birthday_date", Date),
        Col::opt("birthday_month", Int),
        Col::opt("birthday_day", Int),
        Col::opt("birth_month", Int),
        Col::opt("birth_day", Int),
    ],
    constraints: &[
        "PRIMARY KEY (id)",
        "CHECK (keep_until >= issued)",
        "CHECK ((purpose = 'other') = (purpose_other IS NOT NULL))",
        "CHECK ((birthday_month IS NULL) = (birthday_day IS NULL))",
        "CHECK (birthday_date IS NULL OR birthday_month IS NULL)",
        "CHECK (birthday_month IS NULL OR (birthday_month BETWEEN 1 AND 12 AND birthday_day BETWEEN 1 AND 31))",
        "CHECK ((birth_month IS NULL) = (birth_day IS NULL))",
        "CHECK (birth_month IS NULL OR (birth_month BETWEEN 1 AND 12 AND birth_day BETWEEN 1 AND 31))",
    ],
    indexes: &[
        "CREATE INDEX IF NOT EXISTS people_snapshot_person ON people_snapshot (person_id, captured)",
    ],
};

/// The birthday of a person: one row per person, a date with year or a month and a day.
pub const BIRTHDAY: Table = Table {
    name: "people_birthday",
    cols: &[
        Col::req("person_id", Iri),
        Col::req("person_art", Enum(ART)),
        Col::opt("date", Date),
        Col::opt("month", Int),
        Col::opt("day", Int),
        Col::req("set_at", Utc),
    ],
    constraints: &[
        "PRIMARY KEY (person_id)",
        "CHECK ((month IS NULL) = (day IS NULL))",
        "CHECK ((date IS NULL) <> (month IS NULL))",
        "CHECK (month IS NULL OR (month BETWEEN 1 AND 12 AND day BETWEEN 1 AND 31))",
    ],
    indexes: &[],
};
