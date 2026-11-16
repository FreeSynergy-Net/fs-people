//! The kind of document a snapshot is issued for (`people_snapshot.doc_kind`).
//!
//! Pattern: Value Object (a closed word list with its stored form; the list grows by a later step of the domain).

use fs_objects_v5::v5::Result;
use fs_objects_v5::v5::codec::invalid;

/// What a snapshot was issued for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DocKind {
    /// A booking (appointment, room, ticket reservation).
    Booking,
    /// An invoice.
    Invoice,
    /// A receipt.
    Receipt,
    /// A ticket.
    Ticket,
    /// A membership.
    Membership,
    /// Anything else.
    Other,
}

const WORDS: [(DocKind, &str); 6] = [
    (DocKind::Booking, "booking"),
    (DocKind::Invoice, "invoice"),
    (DocKind::Receipt, "receipt"),
    (DocKind::Ticket, "ticket"),
    (DocKind::Membership, "membership"),
    (DocKind::Other, "other"),
];

impl DocKind {
    /// The stored word.
    #[must_use]
    pub fn word(self) -> &'static str {
        WORDS
            .iter()
            .find(|(k, _)| *k == self)
            .map_or("other", |(_, w)| *w)
    }

    /// The kind of a stored word.
    ///
    /// # Errors
    /// `Invalid` for a word that is none.
    pub fn of(word: &str) -> Result<Self> {
        WORDS
            .iter()
            .find(|(_, w)| *w == word)
            .map(|(k, _)| *k)
            .ok_or_else(|| invalid("people_snapshot.doc_kind", "unknown word"))
    }
}
