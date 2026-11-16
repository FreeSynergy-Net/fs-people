//! Conformance rules of `person_directory_v1`, runnable against any provider (fake, real, remote).
//!
//! Pattern: Template method (a fixed list of rules, each checked against the `PersonDirectory` under test).
//!
//! `run_conformance` seeds its own data through `propose`, so it needs a provider with `person.propose` and
//! works on an empty or a filled directory (all seeds carry the marker `Zzconf`). A provider without
//! `person.propose` runs `run_unsupported` instead.

mod read;
mod write;

use crate::person_directory_v1::{CAPABILITIES, Error, PersonDirectory, PersonRef, ProposeEntry};
use fs_types::{BasicValue, ContactAddress};

/// Collects the failed rules; empty at the end means conformant.
#[derive(Default)]
pub struct Report {
    pub failures: Vec<String>,
}

impl Report {
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

/// Run every rule against `dir`. Returns the failed rules (empty = conformant).
pub async fn run_conformance(dir: &dyn PersonDirectory) -> Vec<String> {
    let mut r = Report::default();
    for cap in CAPABILITIES {
        let on = dir.capabilities().0.iter().any(|(c, on)| c == cap && *on);
        r.ensure(on, &format!("capabilities: {cap} is offered"));
    }
    let Some(seeds) = write::seed(dir, &mut r).await else {
        return r.failures;
    };
    write::propose_rules(dir, &seeds, &mut r).await;
    read::resolve_rules(dir, &seeds, &mut r).await;
    read::suggest_rules(dir, &seeds, &mut r).await;
    read::find_rules(dir, &seeds, &mut r).await;
    read::excerpt_rules(dir, &seeds, &mut r).await;
    write::changes_rules(dir, &mut r).await;
    r.ensure(dir.asker().await.is_ok(), "asker: answers without an error");
    r.failures
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
