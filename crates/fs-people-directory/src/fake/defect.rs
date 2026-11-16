//! `Defect`: one deliberate rule break of the fake, to prove that the conformance rules catch it (mutation test of
//! the tests). A fake with a defect is never used as test data; `FakeDirectory::with_defect` is the only door. Both
//! are `#[doc(hidden)]`: they are test machinery of this crate (and of other fakes that copy the idea), not API.
//!
//! Pattern: Fault injection (a closed list of mutants; `tests/mutants.rs` runs every one and demands a failure).

/// A rule the fake breaks on purpose.
#[doc(hidden)]
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
    /// The AI mark only counts while the entry is shared.
    AiFlagNeedsShared,
    /// The AI mark is not looked up at the merge target of the asker.
    AiFlagNoMergeTarget,
    /// The AI mark is only looked up at the end of the merge chain, not at the asker's own id.
    AiFlagOnlyMergeTarget,
    /// `record_consent` without a login is recorded for the person themselves.
    RecordWithoutLogin,
    /// `retain_until` counts from the day the withdrawal was recorded, not from the withdrawal.
    RetainFromRecorded,
    /// Expired records are never purged.
    PurgeKeeps,
    /// A deleted person's consents are withdrawn by that person, not by the one who deletes.
    ErasedByPerson,
}

impl Defect {
    /// Every defect, for the mutation test.
    pub const ALL: [Self; 26] = [
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
        Self::AiFlagNeedsShared,
        Self::AiFlagNoMergeTarget,
        Self::AiFlagOnlyMergeTarget,
        Self::RecordWithoutLogin,
        Self::RetainFromRecorded,
        Self::PurgeKeeps,
        Self::ErasedByPerson,
    ];
}
