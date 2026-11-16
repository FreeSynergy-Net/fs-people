//! `Defect`: one deliberate rule break of the fake, to prove that the conformance rules catch it (mutation test of
//! the tests). A fake with a defect is never used as test data; `FakeDirectory::with_defect` is the only door.
//!
//! Pattern: Fault injection (a closed list of mutants; `tests/mutants.rs` runs every one and demands a failure).

/// A rule the fake breaks on purpose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Defect {
    /// The matching rule ignores `scope`.
    IgnoreScope,
    /// A consent on a profile also covers the person it describes.
    ProfileCoversPerson,
    /// Merged references are compared as written, not by their target.
    NoMergeFollow,
    /// `record_consent` returns the matching active record instead of a new one.
    DropDuplicate,
    /// A second withdrawal replaces the first.
    WithdrawNotIdempotent,
    /// A withdrawal removes the record.
    WithdrawDeletes,
    /// An invisible record gives `Forbidden` instead of `UnknownConsent`.
    WithdrawForbiddenFirst,
    /// Deleting a person leaves their consents active.
    EraseKeepsActive,
    /// `excerpt` does not look for the consent.
    IgnoreConsentInExcerpt,
    /// `record_consent` for others needs no right.
    RecordWithoutRight,
    /// `AiAssistance` without `scope` is accepted.
    AiScopeOptional,
    /// An AI asking `excerpt` with another purpose gets the normal answer.
    AiPurposeLeak,
    /// An AI sees sensitive fields in an excerpt.
    AiSeesSensitive,
    /// An AI may call `find_by_address`.
    AiMayFind,
    /// An AI may `propose` with an address.
    AiProposeAddress,
    /// An AI sees the address in `PersonView`.
    AiSeesAddress,
    /// An AI sees the address as `display`.
    AiSeesAddressDisplay,
    /// `suggest` searches addresses for an AI.
    AiSuggestAddress,
    /// An AI may use the consent methods.
    AiMayConsent,
}

impl Defect {
    /// Every defect, for the mutation test.
    pub const ALL: [Self; 19] = [
        Self::IgnoreScope,
        Self::ProfileCoversPerson,
        Self::NoMergeFollow,
        Self::DropDuplicate,
        Self::WithdrawNotIdempotent,
        Self::WithdrawDeletes,
        Self::WithdrawForbiddenFirst,
        Self::EraseKeepsActive,
        Self::IgnoreConsentInExcerpt,
        Self::RecordWithoutRight,
        Self::AiScopeOptional,
        Self::AiPurposeLeak,
        Self::AiSeesSensitive,
        Self::AiMayFind,
        Self::AiProposeAddress,
        Self::AiSeesAddress,
        Self::AiSeesAddressDisplay,
        Self::AiSuggestAddress,
        Self::AiMayConsent,
    ];
}
