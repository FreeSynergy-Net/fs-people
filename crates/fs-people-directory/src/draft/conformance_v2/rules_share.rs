//! K1–K8 (N2: sensible Felder bei `OwnUse`, Freigabe als Port `FieldShares`). Skelett, ignoriert bis der Kern da ist.
//!
//! Pattern: Template method (Fall = feste Schritte gegen `Kern`; Mutant = die Änderung, die den Fall rot macht).
//!
//! Mutanten: K1 Port auch für eigene Einträge; K2 "ohne Jahr ist frei"; K3 Stufe ignoriert; K4 Zwischenspeicher der
//! Freigabe; K5 Fehler des Ports = freigegeben; K6 KI sieht den Grund; K7 Freigabe auch für Vertrag verlangt;
//! K8 Orakel = Anbieter-Prüfung nachgebaut (verboten).

use super::kern::{Kern, req};
use crate::draft::person_directory_v2::{
    Birthday, BirthdayPrecision, ExcerptField, ExcerptValue, Purpose, ShareField, WithheldWhy,
};

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k01_own_card_needs_no_share() {
    let k = Kern::start();
    let me = k.human();
    k.act(&me);
    let card = k.own_card(&me);
    let ex = futures_block(k.d.excerpt(&card, &req(ExcerptField::BirthMonthDay, Purpose::OwnUse)))
        .unwrap();
    assert!(matches!(
        ex.values.as_slice(),
        [ExcerptValue::BirthMonthDay(_)]
    ));
    assert_eq!(k.share_port_calls(), 0, "no share asked for an own entry");
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k02_foreign_profile_without_share_is_withheld() {
    let k = Kern::start();
    let (me, other) = (k.human(), k.human());
    let profile = k.foreign_profile(&other);
    k.act(&me);
    let mut r = req(ExcerptField::BirthMonthDay, Purpose::OwnUse);
    r.fields.push(ExcerptField::FullName);
    let ex = futures_block(k.d.excerpt(&profile, &r)).unwrap();
    assert_eq!(ex.withheld.len(), 1);
    assert_eq!(ex.withheld[0].why, WithheldWhy::NotShared);
    assert!(
        ex.values
            .iter()
            .any(|v| matches!(v, ExcerptValue::FullName(_))),
        "the name comes"
    );
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k03_share_level_month_day_only() {
    let k = Kern::start();
    let (me, other) = (k.human(), k.human());
    let profile = k.foreign_profile(&other);
    let share = k.share(
        &other,
        ShareField::Birthday(BirthdayPrecision::MonthDay),
        &me,
        None,
    );
    k.act(&me);
    let mut r = req(ExcerptField::Birthday, Purpose::OwnUse);
    r.fields.push(ExcerptField::BirthMonthDay);
    let ex = futures_block(k.d.excerpt(&profile, &r)).unwrap();
    assert!(
        ex.values
            .iter()
            .all(|v| !matches!(v, ExcerptValue::Birthday(Birthday::Date(_))))
    );
    assert_eq!(ex.shares.len(), 1);
    assert_eq!(ex.shares[0].share, share);
    assert_eq!(ex.withheld[0].field, ExcerptField::Birthday);
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k04_expired_or_revoked_share_is_not_shared() {
    let k = Kern::start();
    let (me, other) = (k.human(), k.human());
    let profile = k.foreign_profile(&other);
    let share = k.share(&other, ShareField::Email, &me, None);
    k.act(&me);
    let shared =
        futures_block(k.d.excerpt(&profile, &req(ExcerptField::Email, Purpose::OwnUse))).unwrap();
    assert_eq!(shared.withheld, Vec::new());
    k.revoke_share(share);
    let ex =
        futures_block(k.d.excerpt(&profile, &req(ExcerptField::Email, Purpose::OwnUse))).unwrap();
    assert_eq!(
        ex.withheld[0].why,
        WithheldWhy::NotShared,
        "no cache of the share"
    );
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k05_unreachable_port_is_unverifiable() {
    let k = Kern::start();
    let (me, other) = (k.human(), k.human());
    let profile = k.foreign_profile(&other);
    k.act(&me);
    k.share_port_down(true);
    let ex =
        futures_block(k.d.excerpt(&profile, &req(ExcerptField::Phone, Purpose::OwnUse))).unwrap();
    assert_eq!(ex.withheld[0].why, WithheldWhy::Unverifiable);
    assert!(ex.values.is_empty(), "fail-closed: no value");
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k06_ai_sees_no_reason() {
    let k = Kern::start();
    let (ai, other) = (k.agent(), k.human());
    let profile = k.foreign_profile(&other);
    k.share(&other, ShareField::Postal, &ai, None);
    k.act(&ai);
    let ex =
        futures_block(k.d.excerpt(&profile, &req(ExcerptField::Postal, Purpose::AiAssistance)))
            .unwrap();
    assert!(
        ex.missing.contains(&ExcerptField::Postal),
        "traceless in `missing`"
    );
    assert_eq!(ex.withheld, Vec::new());
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k07_contract_needs_no_share() {
    let k = Kern::start();
    let (me, other) = (k.human(), k.human());
    let profile = k.foreign_profile(&other);
    k.act(&me);
    let ex = futures_block(k.d.excerpt(&profile, &req(ExcerptField::Postal, Purpose::Contract)))
        .unwrap();
    assert!(
        matches!(ex.values.as_slice(), [ExcerptValue::Postal(_)]),
        "as in v1"
    );
}

/// Eigenes Orakel (Mengenmodell): Einträge mit Autor, Freigaben (Person, Feld, Stufe, Ablauf). Es kennt die Regel nur
/// als Mengenzugehörigkeit und ruft den Anbieter nie auf.
struct Model {
    authored: Vec<(u8, u8)>,                        // (autor, eintrag)
    shares: Vec<(u8, ShareField, u8, Option<i64>)>, // (besitzer, feld, betrachter, ablauf ms)
}

impl Model {
    /// Erlaubt, wenn der Eintrag vom Betrachter stammt oder eine ungleich abgelaufene Freigabe der ausreichenden Stufe da ist.
    fn allowed(&self, viewer: u8, owner: u8, entry: u8, field: ShareField, now: i64) -> bool {
        self.authored.contains(&(viewer, entry))
            || self.shares.iter().any(|(o, f, v, exp)| {
                *o == owner && *v == viewer && covers(*f, field) && exp.is_none_or(|e| now < e)
            })
    }
}

fn covers(have: ShareField, want: ShareField) -> bool {
    match (have, want) {
        (ShareField::Birthday(BirthdayPrecision::Full), ShareField::Birthday(_)) => true,
        (a, b) => a == b,
    }
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k08_property_every_delivered_value_has_a_reason() {
    // Zufallsfolgen (fester Startwert); je Folge Anbieter und Orakel gleich füttern, dann jede Antwort vergleichen.
    let m = Model {
        authored: vec![(1, 1)],
        shares: vec![(2, ShareField::Email, 1, None)],
    };
    assert!(m.allowed(1, 9, 1, ShareField::Phone, 0), "own entry");
    assert!(m.allowed(1, 2, 5, ShareField::Email, 0), "shared");
    assert!(!m.allowed(1, 2, 5, ShareField::Phone, 0), "not shared");
    let _ = Kern::start();
}

fn futures_block<T>(f: impl std::future::Future<Output = T>) -> T {
    super::pure::block_on(f)
}
