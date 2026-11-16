//! The People directory in fs-objects v5: typed tables for consents, the frozen snapshot, the birthday and the
//! setting `consent_retention`. No JSON, no blob (ADR 0086); every value has a column of its own type.
//!
//! Pattern: Facade (declarations and the one `Domain`) over repository functions on a unit of work (`Conn`): the
//! functions are free of a trait implementation on purpose. They store what the locked trait `person_directory_v1`
//! names; the implementation of that trait over them (`PeopleRepo`, point 6) is a later step.
//!
//! The domain prefix is `people`: a v5 store file belongs to one program, so this store is the People store of the
//! directory, next to (not inside) the v4 store of the cards. There is no old data to migrate: consents, snapshots
//! and birthdays are new with the delta; the cards stay in the v4 store until their own lift (point 6).

mod birthday;
pub mod consent;
mod doc_kind;
mod domain;
mod person_birthday;
mod retention;
pub mod snapshot;
mod tables;
mod words;

pub use doc_kind::DocKind;
pub use domain::{DOMAIN, open};
pub use person_birthday::{birthday_of, set_birthday};
pub use retention::{KEY_CONSENT_RETENTION, consent_retention, set_consent_retention};
pub use snapshot::Snapshot;
pub use tables::{BIRTHDAY, CONSENT, DOC_KINDS, ORIGINS, PURPOSES, SNAPSHOT};
