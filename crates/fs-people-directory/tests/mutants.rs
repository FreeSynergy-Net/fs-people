//! Mutation test of the conformance rules: every deliberate defect of the fake must be caught by the rule written
//! for it, not by accident.
//!
//! Pattern: Contract test (the rules are only worth what they catch; a clean fake passes, each mutant fails).

use fs_people_directory::conformance::run_fixture_conformance;
use fs_people_directory::{Defect, FakeDirectory, FakeFixture};

/// Each defect with a part of the name of the rule that must fail.
const EXPECTED: [(Defect, &str); 19] = [
    (Defect::IgnoreScope, "another scope does not match"),
    (Defect::ProfileCoversPerson, "never covers the person"),
    (Defect::NoMergeFollow, "consents move with a merge"),
    (Defect::DropDuplicate, "gets its own entry"),
    (
        Defect::WithdrawNotIdempotent,
        "second withdrawal returns the first",
    ),
    (Defect::WithdrawDeletes, "never deletes the record"),
    (
        Defect::WithdrawForbiddenFirst,
        "UnknownConsent, not Forbidden",
    ),
    (Defect::EraseKeepsActive, "withdrawn with Erasure"),
    (
        Defect::IgnoreConsentInExcerpt,
        "Photo without a consent is NoConsent",
    ),
    (Defect::RecordWithoutRight, "for others needs consent.write"),
    (
        Defect::AiScopeOptional,
        "AiAssistance without scope is Invalid",
    ),
    (Defect::AiPurposeLeak, "than AiAssistance is Forbidden"),
    (Defect::AiSeesSensitive, "sensitive fields are in missing"),
    (Defect::AiMayFind, "find_by_address is Forbidden"),
    (
        Defect::AiProposeAddress,
        "propose with an address is Forbidden",
    ),
    (Defect::AiSeesAddress, "PersonView.address is None"),
    (Defect::AiSeesAddressDisplay, "no address stage"),
    (Defect::AiSuggestAddress, "never searches addresses"),
    (Defect::AiMayConsent, "check_consent is Forbidden"),
];

#[tokio::test]
async fn a_clean_fake_passes_every_rule() {
    let report = run_fixture_conformance(&FakeFixture::new()).await;
    assert_eq!(report.failures, Vec::<String>::new());
    assert_eq!(report.skipped, Vec::<String>::new());
}

#[tokio::test]
async fn every_mutant_is_caught_by_its_rule() {
    assert_eq!(EXPECTED.len(), Defect::ALL.len());
    let mut missed = Vec::new();
    for (defect, rule) in EXPECTED {
        let fixture = FakeFixture::over(FakeDirectory::new().with_defect(defect));
        let report = run_fixture_conformance(&fixture).await;
        if !report.failures.iter().any(|f| f.contains(rule)) {
            missed.push(defect);
        }
    }
    assert_eq!(
        missed,
        Vec::<Defect>::new(),
        "mutants the intended rule did not catch"
    );
}
