//! K23–K29 (Anpassung 6 RÜCKWIRKEND, b7 17.11. Antwort 6 und Entscheidung 11, `retain_until` in der gespeicherten Zone).
//! Die Fälle gegen den Anbieter sind ignoriert; die reine Rechnung (K23, K24, K28) läuft in `pure`.
//!
//! Pattern: Template method (Fall = feste Schritte gegen `Kern`; Mutant = die Änderung, die den Fall rot macht).
//!
//! Mutanten: K23 +3 Jahre ab Datum; K24 nur UTC-Jahr; K25 nur neue Widerrufe; K25b Verkürzung ohne Löschung, ohne
//! Vorschau, ohne Hinweis oder ohne Bestätigungspflicht; K25c Teil-Neuberechnung; K26 eigene Rechnung je Weg;
//! K27 weich gelöscht; K28 Orakel ruft `after` / merkt N je Widerruf; K29 nur `person` darf widerrufen.

use super::kern::{Kern, entry};
use super::pure::block_on;
use crate::draft::person_directory_v2::{
    ConsentOrigin, ConsentRetention, Error, Lifetime, Purpose, When, Withdrawal,
};
use fs_types::{BasicValue, Date, Utc};

fn on(d: &str) -> When {
    When::On(Date::parse(d).expect("a date"))
}

fn withdrawal(at: When) -> Withdrawal {
    Withdrawal {
        at,
        zone: None,
        origin: ConsentOrigin::ByPerson,
        evidence: None,
    }
}

fn n(years: u8) -> ConsentRetention {
    ConsentRetention::new(years).expect("1 to 10")
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k23_withdrawal_on_a_date_keeps_until_year_end_plus_n() {
    let k = Kern::start();
    let p = k.human();
    k.act(&p);
    let c = block_on(k.d.record_consent(&entry(&p, Purpose::Newsletter, None, on("2026-01-02"))))
        .unwrap();
    let w = block_on(k.d.withdraw_consent(c.id, &withdrawal(on("2026-05-10")))).unwrap();
    assert_eq!(
        w.retain_until,
        Some(Date::parse("2029-12-31").expect("a date"))
    );
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k25_retroactive_new_period_applies_to_existing_proofs() {
    let k = Kern::start();
    let p = k.human();
    k.act(&p);
    let c = block_on(k.d.record_consent(&entry(&p, Purpose::Newsletter, None, on("2026-01-02"))))
        .unwrap();
    block_on(k.d.withdraw_consent(c.id, &withdrawal(on("2026-05-10")))).unwrap();
    block_on(k.settings().set_consent_retention(n(5), false)).unwrap();
    let all = block_on(k.d.consents_of(&p)).unwrap();
    assert_eq!(
        all[0].retain_until,
        Some(Date::parse("2031-12-31").expect("a date"))
    );
    assert_eq!(k.retention_log().len(), 1, "log: retention 3 -> 5, 1 proof");
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k25b_shortening_is_a_deletion_with_count_hint_and_confirmation() {
    let k = Kern::start();
    let p = k.human();
    k.act(&p);
    block_on(k.settings().set_consent_retention(n(10), true)).unwrap();
    let c = block_on(k.d.record_consent(&entry(&p, Purpose::Newsletter, None, on("2020-01-02"))))
        .unwrap();
    block_on(k.d.withdraw_consent(c.id, &withdrawal(on("2020-06-01")))).unwrap();
    let pre = block_on(k.settings().preview_retention(n(3))).unwrap();
    assert_eq!(pre.to_delete, 1, "the preview names the count");
    assert!(
        pre.legal_hint,
        "the hint: may still be legally needed, not legal advice"
    );
    assert!(
        matches!(
            block_on(k.settings().set_consent_retention(n(3), false)),
            Err(Error::Invalid(_))
        ),
        "confirmation required"
    );
    assert_eq!(
        block_on(k.d.consents_of(&p)).unwrap().len(),
        1,
        "nothing written without it"
    );
    block_on(k.settings().set_consent_retention(n(3), true)).unwrap();
    k.cleanup();
    assert!(
        block_on(k.d.consents_of(&p)).unwrap().is_empty(),
        "after the cleanup run it is really gone"
    );
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k25c_a_failing_recompute_leaves_the_old_state() {
    let k = Kern::start();
    let p = k.human();
    k.act(&p);
    let c = block_on(k.d.record_consent(&entry(&p, Purpose::Newsletter, None, on("2026-01-02"))))
        .unwrap();
    block_on(k.d.withdraw_consent(c.id, &withdrawal(on("2026-05-10")))).unwrap();
    k.fail_mid_recompute();
    assert!(block_on(k.settings().set_consent_retention(n(5), false)).is_err());
    assert_eq!(
        block_on(k.settings().consent_retention()).unwrap(),
        ConsentRetention::default()
    );
    assert_eq!(
        block_on(k.d.consents_of(&p)).unwrap()[0].retain_until,
        Some(Date::parse("2029-12-31").expect("a date"))
    );
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k26_erasure_and_end_situation_use_the_same_calculation() {
    let k = Kern::start();
    let p = k.human();
    k.act(&p);
    let mut e = entry(&p, Purpose::AiAssistance, None, on("2026-01-02"));
    e.scope = Some(
        super::kern::entry(
            &p,
            Purpose::Newsletter,
            Some("urn:uuid:00000000-0000-7000-8000-000000000001"),
            on("2026-01-02"),
        )
        .scope
        .expect("scope"),
    );
    e.lifetime = Some(Lifetime::WhileScope);
    block_on(k.d.record_consent(&e)).unwrap();
    let scope = e.scope.clone().expect("scope");
    block_on(k.as_program().end_situation(&scope, on("2026-05-10"), None)).unwrap();
    let ended = block_on(k.d.consents_of(&p)).unwrap();
    assert_eq!(
        ended[0].retain_until,
        Some(Date::parse("2029-12-31").expect("a date")),
        "same as a plain withdrawal (K23)"
    );
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k27_after_retain_until_the_proof_is_really_gone() {
    let k = Kern::start();
    let p = k.human();
    k.act(&p);
    let c = block_on(k.d.record_consent(&entry(&p, Purpose::Newsletter, None, on("2020-01-02"))))
        .unwrap();
    block_on(k.d.withdraw_consent(c.id, &withdrawal(on("2020-06-01")))).unwrap();
    k.set_clock(Utc::parse("2024-01-01T00:00:00Z").expect("a time"));
    k.cleanup();
    assert!(
        block_on(k.d.consents_of(&p)).unwrap().is_empty(),
        "not in consents_of, not in the history"
    );
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k29_given_by_may_withdraw_without_consent_write() {
    let k = Kern::start();
    let (child, parent) = (k.human(), k.human());
    k.act(&parent);
    let mut e = entry(&child, Purpose::Newsletter, None, on("2026-01-02"));
    e.given_by = Some(parent.clone());
    let c = block_on(k.d.record_consent(&e)).unwrap();
    k.grant_admin(&parent, false);
    assert!(block_on(k.d.withdraw_consent(c.id, &withdrawal(on("2026-05-10")))).is_ok());
}
