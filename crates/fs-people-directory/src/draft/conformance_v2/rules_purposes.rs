//! K15–K22 (N6: die Zweck-Liste ist editierbar). Skelett, ignoriert bis der Kern da ist.
//!
//! Pattern: Template method (Fall = feste Schritte gegen `Kern`; Mutant = die Änderung, die den Fall rot macht).
//!
//! Mutanten: K15 umsortiert; K16 `needs_consent` aus dem Entwurf; K17 für alle gelistet; K18 `expected` ignoriert /
//! Fassung überschrieben; K19 Zurückziehen widerruft; K20 Text ändert die Regel; K21 leerer Text; K22 Lesen erlaubt.
//! Eigene Zwecke brauchen IMMER eine Einwilligung (VORLÄUFIG, b7 17.11., Frage 5).

use super::kern::{Kern, entry};
use super::pure::block_on;
use crate::draft::person_directory_v2::{
    ConsentQuery, Error, Purpose, PurposeDraft, PurposeOwner, PurposeText, Reference, Revision,
    When,
};
use fs_types::{BasicValue, Language, Text};

fn draft(owner: PurposeOwner) -> PurposeDraft {
    PurposeDraft {
        owner,
        texts: vec![PurposeText {
            language: Language::parse("de").expect("a language"),
            label: Text::parse("Mitgliederpost").expect("a text"),
            explanation: Text::parse("Wir schreiben Ihnen.").expect("a text"),
        }],
        close_match: None,
        recording: None,
    }
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k15_standard_purposes_equal_v1() {
    let k = Kern::start();
    let all = block_on(k.c.purposes()).unwrap();
    assert!(all.len() >= 9);
    assert_eq!(all[0].purpose, Purpose::Contract);
    assert!(
        !all[0].needs_consent && all[5].needs_consent,
        "needs_consent as in v1"
    );
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k16_instance_purpose_needs_the_right() {
    let k = Kern::start();
    let p = k.human();
    k.act(&p);
    k.grant_admin(&p, false);
    assert!(matches!(
        block_on(k.c.define_purpose(&draft(PurposeOwner::Instance))),
        Err(Error::Forbidden { .. })
    ));
    k.grant_admin(&p, true);
    let def = block_on(k.c.define_purpose(&draft(PurposeOwner::Instance))).unwrap();
    assert!(matches!(&def.purpose, Purpose::Other(r) if r.canonical().starts_with("urn:uuid:")));
    assert!(
        def.needs_consent,
        "own purposes always need consent (provisional)"
    );
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k17_person_purpose_is_private() {
    let k = Kern::start();
    let (a, b) = (k.human(), k.human());
    k.act(&a);
    let def = block_on(k.c.define_purpose(&draft(PurposeOwner::Person(a)))).unwrap();
    k.act(&b);
    assert!(
        block_on(k.c.purposes())
            .unwrap()
            .iter()
            .all(|d| d.purpose != def.purpose)
    );
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k18_stale_revision_is_changed_and_old_version_stays_readable() {
    let k = Kern::start();
    let p = k.human();
    k.act(&p);
    k.grant_admin(&p, true);
    let def = block_on(k.c.define_purpose(&draft(PurposeOwner::Instance))).unwrap();
    let v2 =
        block_on(k.c.revise_purpose(&def.purpose, Some(def.rev), &draft(PurposeOwner::Instance)))
            .unwrap();
    assert_ne!(v2.version, def.version);
    let old = block_on(k.c.purpose_version(&def.version))
        .unwrap()
        .expect("the agreed text version stays readable");
    assert_eq!(
        old.texts, def.texts,
        "Art. 7(1): the old text, not the current one"
    );
    let stale =
        block_on(k.c.revise_purpose(&def.purpose, Some(def.rev), &draft(PurposeOwner::Instance)));
    assert!(matches!(stale, Err(Error::Changed { current, .. }) if current == v2.rev));
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k19_retire_blocks_new_records_only() {
    let k = Kern::start();
    let p = k.human();
    k.act(&p);
    let def = block_on(k.c.define_purpose(&draft(PurposeOwner::Person(p.clone())))).unwrap();
    let e = entry(
        &p,
        def.purpose.clone(),
        None,
        When::On(fs_types::Date::parse("2026-11-17").expect("a date")),
    );
    block_on(k.d.record_consent(&e)).unwrap();
    block_on(k.c.retire_purpose(&def.purpose, def.rev)).unwrap();
    assert!(matches!(
        block_on(k.d.record_consent(&e)),
        Err(Error::PurposeRetired(_))
    ));
    let q = ConsentQuery {
        person: p,
        purpose: def.purpose,
        scope: None,
        recipient: None,
    };
    assert!(block_on(k.d.check_consent(&q)).unwrap().is_some());
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k20_standard_purpose_is_fixed_but_its_text_changes() {
    let k = Kern::start();
    let p = k.human();
    k.act(&p);
    k.grant_admin(&p, true);
    let all = block_on(k.c.purposes()).unwrap();
    let std = all[0].clone();
    assert!(matches!(
        block_on(k.c.retire_purpose(&std.purpose, std.rev)),
        Err(Error::StandardPurposeFixed)
    ));
    let new =
        block_on(k.c.revise_purpose(&std.purpose, None, &draft(PurposeOwner::Instance))).unwrap();
    assert_eq!(new.purpose, std.purpose);
    assert_eq!(new.needs_consent, std.needs_consent);
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k21_unknown_dpv_purpose_shows_placeholder_and_iri() {
    let k = Kern::start();
    let p = k.human();
    k.act(&p);
    let iri = Reference::parse("https://w3id.org/dpv#Research").expect("an IRI");
    let e = entry(
        &p,
        Purpose::Other(iri),
        None,
        When::On(fs_types::Date::parse("2026-11-17").expect("a date")),
    );
    let c = block_on(k.d.record_consent(&e)).unwrap();
    assert!(
        block_on(k.c.purposes())
            .unwrap()
            .iter()
            .all(|d| d.purpose != c.entry.purpose),
        "no row: placeholder in the UI"
    );
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k22_ai_cannot_touch_purposes() {
    let k = Kern::start();
    let ai = k.agent();
    k.act(&ai);
    assert!(matches!(
        block_on(k.c.purposes()),
        Err(Error::Forbidden { .. })
    ));
    assert!(matches!(
        block_on(k.c.define_purpose(&draft(PurposeOwner::Instance))),
        Err(Error::Forbidden { .. })
    ));
    let _ = Revision(0);
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k17b_levels_rights_and_owner_mismatch() {
    // V3: a person defines and changes own purposes and an own text of a standard purpose WITHOUT `purpose.write`;
    // `owner` unequal to the former owner is `Invalid`; `Person(other)` is `Forbidden`; `purposes()` lists one entry
    // per level (the instance text next to the own override).
    let k = Kern::start();
    let (a, b) = (k.human(), k.human());
    k.act(&a);
    k.grant_admin(&a, false);
    let own = block_on(k.c.define_purpose(&draft(PurposeOwner::Person(a.clone())))).unwrap();
    let moved =
        block_on(k.c.revise_purpose(&own.purpose, Some(own.rev), &draft(PurposeOwner::Instance)));
    assert!(matches!(moved, Err(Error::Invalid(_))));
    let foreign = block_on(k.c.define_purpose(&draft(PurposeOwner::Person(b))));
    assert!(matches!(foreign, Err(Error::Forbidden { .. })));
    let std = block_on(k.c.purposes()).unwrap()[0].purpose.clone();
    block_on(k.c.revise_purpose(&std, None, &draft(PurposeOwner::Person(a)))).unwrap();
    let levels = block_on(k.c.purposes())
        .unwrap()
        .iter()
        .filter(|d| d.purpose == std)
        .count();
    assert_eq!(levels, 2, "built-in/instance text and the own override");
}
