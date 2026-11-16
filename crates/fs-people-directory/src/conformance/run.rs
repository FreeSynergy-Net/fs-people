//! The runners and shared helpers of the conformance rules.
//!
//! Pattern: Template method (a fixed list of rules, each checked against the provider under test).

use super::{ConformanceFixture, extra, fixture_rules, read, write};
use crate::person_directory_v1::{CAPABILITIES, Error, PersonDirectory, PersonRef, ProposeEntry};
use fs_types::{BasicValue, ContactAddress};

/// Collects the failed rules; empty at the end means conformant.
#[derive(Default)]
pub struct Report {
    /// Rules that failed.
    pub failures: Vec<String>,
    /// Rules a fixture could not set up (it has no such mutator); not a failure, but not proven either.
    pub skipped: Vec<String>,
}

impl Report {
    pub(crate) fn skip(&mut self, rule: &str) {
        self.skipped.push(rule.to_owned());
    }

    pub(crate) fn ensure(&mut self, ok: bool, rule: &str) {
        if !ok {
            self.failures.push(rule.to_owned());
        }
    }

    pub(crate) fn is_err<T>(
        &mut self,
        result: &Result<T, Error>,
        rule: &str,
        want: fn(&Error) -> bool,
    ) {
        self.ensure(result.as_ref().err().is_some_and(want), rule);
    }
}

/// The seeded entries every rule works on.
pub(crate) struct Seeds {
    pub a: PersonRef,
    pub a_address: ContactAddress,
    pub b: PersonRef,
    pub c: PersonRef,
}

pub(crate) fn address(text: &str) -> Option<ContactAddress> {
    ContactAddress::parse(text).ok()
}

pub(crate) fn entry(hint: &str, addr: Option<&str>) -> ProposeEntry {
    ProposeEntry {
        display_hint: hint.to_owned(),
        address: addr.and_then(address),
        origin: None,
    }
}

/// The rules that need only the trait: data is seeded through `propose`.
pub(crate) async fn base(dir: &dyn PersonDirectory, r: &mut Report) -> Option<Seeds> {
    for cap in CAPABILITIES {
        let on = dir.capabilities().0.iter().any(|(c, on)| c == cap && *on);
        r.ensure(on, &format!("capabilities: {cap} is offered"));
    }
    let seeds = write::seed(dir, r).await?;
    write::propose_rules(dir, &seeds, r).await;
    read::resolve_rules(dir, &seeds, r).await;
    read::suggest_rules(dir, &seeds, r).await;
    read::find_rules(dir, &seeds, r).await;
    read::excerpt_rules(dir, &seeds, r).await;
    extra::rules(dir, &seeds, r).await;
    write::changes_rules(dir, r).await;
    r.ensure(dir.asker().await.is_ok(), "asker: answers without an error");
    Some(seeds)
}

/// Run every rule that needs only the trait against `dir` (data is seeded through `propose`, so the provider
/// needs `person.propose`). Returns the failed rules (empty = conformant). Rules that need `Deleted`, `NotShared`,
/// `Merged`, `Restored` or `Forbidden` states are in [`run_fixture_conformance`].
pub async fn run_conformance(dir: &dyn PersonDirectory) -> Vec<String> {
    let mut r = Report::default();
    base(dir, &mut r).await;
    r.failures
}

/// All rules: those of [`run_conformance`] plus the state rules, with the states set up by the fixture.
pub async fn run_fixture_conformance(fixture: &dyn ConformanceFixture) -> Report {
    let mut r = Report::default();
    if let Some(seeds) = base(fixture.directory(), &mut r).await {
        fixture_rules::rules(fixture, &seeds, &mut r).await;
    }
    r
}

/// Rules for a provider WITHOUT `person.propose` / `person.excerpt`: the optional methods say `Unsupported`
/// (Null Object), and `capabilities` says the same.
pub async fn run_unsupported(dir: &dyn PersonDirectory) -> Vec<String> {
    let mut r = Report::default();
    let is_unsupported = |e: &Error| matches!(e, Error::Unsupported { .. });
    let caps = dir.capabilities();
    let off = |name: &str| caps.0.iter().any(|(c, on)| c == name && !*on);
    let origin = fs_types::Reference::parse("urn:uuid:00000000-0000-7000-8000-00000000ffff");
    if let Ok(origin) = origin {
        let opened = dir.open_batch(&origin).await;
        r.is_err(&opened, "open_batch: Unsupported", is_unsupported);
    }
    let proposed = dir.propose(&entry("x", None), None).await;
    r.is_err(&proposed, "propose: Unsupported", is_unsupported);
    r.ensure(off("person.propose"), "capabilities: person.propose is off");
    r.failures
}
