//! Gemeinsame Typen: Wiederverwendung von `fs-types` (>= 0.0.22), kein Stand-in mehr. Ein Name weicht ab: `Place`
//! (Weiterblätter-Stelle) ist dort `PageCursor` (`Place` ist dort der `geo:`-Punkt). `ContactAddress` (Erreichbarkeit
//! als IRI) und `Address` (Postanschrift) tragen die Namen von `fs-types`, kein Alias.
//!
//! Pattern: Value Object (Basistypen aus fs-types; Eintritt nur über `parse`/Konstruktoren).

pub use fs_types::PageCursor as Place;
pub use fs_types::{
    Address, BasicValue, Capabilities, ContactAddress, EmailAddress, FileRef, MediaType, Name,
    ObjectId, Page, PersonArt, PersonRef, PhoneNumber, Problem, Reference, Revision, Text,
    TypeProblem, Utc,
};
