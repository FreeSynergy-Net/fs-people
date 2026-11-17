//! K9–K14 (N3: KI-Zustimmung nur für die Situation: `Lifetime`, `end_situation`, `SituationEnded`). Skelett.
//!
//! Pattern: Template method (Fall = feste Schritte gegen `Kern`; Mutant = die Änderung, die den Fall rot macht).
//!
//! Mutanten: K9 Laufzeit optional; K10 `<=` statt `<`; K11 alle Zwecke / alle Räume; K12 Recht nicht geprüft;
//! K13 alter Eintrag wird reaktiviert; K14 nur `Erasure` geprüft.

use super::kern::{Kern, entry};
use super::pure::block_on;
use crate::draft::person_directory_v2::{
    ConsentOrigin, ConsentQuery, Error, Lifetime, Purpose, RecordingKeep, RecordingKind,
    RecordingNotice, Utc, When,
};
use fs_types::{BasicValue, Reference};

fn at(ms: i64) -> When {
    When::At(Utc::from_millis(ms))
}

fn scope(n: u8) -> Reference {
    Reference::parse(&format!(
        "urn:uuid:00000000-0000-7000-8000-0000000000{n:02x}"
    ))
    .expect("a valid IRI")
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k09_ai_needs_lifetime_and_only_ai_may_have_one() {
    let k = Kern::start();
    let p = k.human();
    k.act(&p);
    let ai = entry(
        &p,
        Purpose::AiAssistance,
        Some("urn:uuid:00000000-0000-7000-8000-000000000001"),
        at(0),
    );
    assert!(matches!(
        block_on(k.d.record_consent(&ai)),
        Err(Error::Invalid(_))
    ));
    let mut photo = entry(&p, Purpose::Photo, None, at(0));
    photo.lifetime = Some(Lifetime::WhileScope);
    photo.notice = Some(RecordingNotice {
        kind: RecordingKind::Photo,
        default_retention: RecordingKeep::Days(30),
        text: scope(2),
    });
    assert!(matches!(
        block_on(k.d.record_consent(&photo)),
        Err(Error::Invalid(_))
    ));
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k10_until_is_half_open() {
    let k = Kern::start();
    let p = k.human();
    k.act(&p);
    let t = 1_000_000;
    let mut e = entry(
        &p,
        Purpose::AiAssistance,
        Some("urn:uuid:00000000-0000-7000-8000-000000000001"),
        at(0),
    );
    e.lifetime = Some(Lifetime::Until(Utc::from_millis(t)));
    block_on(k.d.record_consent(&e)).unwrap();
    let q = ConsentQuery {
        person: p,
        purpose: Purpose::AiAssistance,
        scope: e.scope.clone(),
        recipient: None,
    };
    k.set_clock(Utc::from_millis(t - 1));
    assert!(block_on(k.d.check_consent(&q)).unwrap().is_some());
    k.set_clock(Utc::from_millis(t));
    assert!(block_on(k.d.check_consent(&q)).unwrap().is_none());
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k11_end_situation_revokes_only_ai_in_that_scope() {
    let k = Kern::start();
    let p = k.human();
    k.act(&p);
    let (s, s2) = (scope(1), scope(2));
    for sc in [&s, &s, &s2] {
        let mut e = entry(&p, Purpose::AiAssistance, None, at(0));
        e.scope = Some(sc.clone());
        e.lifetime = Some(Lifetime::WhileScope);
        block_on(k.d.record_consent(&e)).unwrap();
    }
    let mut photo = entry(&p, Purpose::Photo, None, at(0));
    photo.scope = Some(s.clone());
    photo.notice = Some(RecordingNotice {
        kind: RecordingKind::Photo,
        default_retention: RecordingKeep::Days(30),
        text: scope(3),
    });
    block_on(k.d.record_consent(&photo)).unwrap();
    let program = k.as_program();
    let ids = block_on(program.end_situation(&s, at(1), None, None)).unwrap();
    assert_eq!(ids.len(), 2);
    let all = block_on(k.d.consents_of(&p)).unwrap();
    let ended = all
        .iter()
        .filter(|c| {
            c.withdrawn
                .as_ref()
                .is_some_and(|w| w.withdrawal.origin == ConsentOrigin::SituationEnded)
        })
        .count();
    assert_eq!(ended, 2, "the photo in s and the AI in s2 stay active");
    assert!(
        block_on(program.end_situation(&s, at(2), None, None))
            .unwrap()
            .is_empty(),
        "idempotent"
    );
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k12_end_situation_needs_the_right() {
    let k = Kern::start();
    let (human, ai) = (k.human(), k.agent());
    k.act(&human);
    assert!(matches!(
        block_on(k.d.end_situation(&scope(1), at(1), None, None)),
        Err(Error::Forbidden { .. })
    ));
    k.act(&ai);
    assert!(matches!(
        block_on(k.d.end_situation(&scope(1), at(1), None, None)),
        Err(Error::Forbidden { .. })
    ));
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k13_new_consent_after_the_end_is_a_new_entry() {
    let k = Kern::start();
    let p = k.human();
    k.act(&p);
    let mut e = entry(&p, Purpose::AiAssistance, None, at(0));
    e.scope = Some(scope(1));
    e.lifetime = Some(Lifetime::WhileScope);
    let old = block_on(k.d.record_consent(&e)).unwrap();
    block_on(k.as_program().end_situation(&scope(1), at(1), None, None)).unwrap();
    let new = block_on(k.d.record_consent(&e)).unwrap();
    assert_ne!(old.id, new.id);
    let all = block_on(k.d.consents_of(&p)).unwrap();
    assert!(
        all.iter()
            .find(|c| c.id == old.id)
            .unwrap()
            .withdrawn
            .is_some(),
        "the old one stays withdrawn"
    );
    assert!(
        all.iter()
            .find(|c| c.id == new.id)
            .unwrap()
            .withdrawn
            .is_none()
    );
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k14_withdrawal_origins_cannot_be_recorded() {
    let k = Kern::start();
    let p = k.human();
    k.act(&p);
    for origin in [ConsentOrigin::SituationEnded, ConsentOrigin::Erasure] {
        let mut e = entry(&p, Purpose::Newsletter, None, at(0));
        e.origin = origin;
        assert!(matches!(
            block_on(k.d.record_consent(&e)),
            Err(Error::Invalid(_))
        ));
    }
}
