//! Was OHNE Anbieter läuft: die byte-gleichen v1-Dateien, die Fähigkeitsliste, der Null-Anbieter als `dyn`, K23/K24 und
//! K28 (Property-Test mit eigener Kalendertabelle) gegen `ConsentRetention::after`, `Days(0)`-freie Form der Laufzeit.
//!
//! Pattern: Test fixture (ein Fake nur für die Zeitzone: feste Verschiebungen; die Rechnung selbst ist die echte).

use crate::draft::person_directory_v2::{
    CAPABILITIES, ConsentRetention, Error, NullDirectory, PersonDirectory, When, ZoneOffsets,
};
use fs_types::{BasicValue, Date, Utc, Zone};

/// Führt eine Zukunft mit einer Einzel-Thread-Laufzeit aus (nur Tests).
pub(super) fn block_on<T>(f: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("a runtime")
        .block_on(f)
}

/// Feste Verschiebungen: UTC, Berlin (Winter), Kiritimati (+14), Pago Pago (-11). Unbekannt: `None`.
struct Fixed;

impl ZoneOffsets for Fixed {
    fn offset_minutes(&self, zone: &Zone, _at: Utc) -> Option<i32> {
        match zone.id() {
            "UTC" => Some(0),
            "Europe/Berlin" => Some(60),
            "Pacific/Kiritimati" => Some(14 * 60),
            "Pacific/Pago_Pago" => Some(-11 * 60),
            _ => None,
        }
    }
}

fn zone(id: &str) -> Zone {
    Zone::iana(id).expect("a zone")
}

fn n(years: u8) -> ConsentRetention {
    ConsentRetention::new(years).expect("1 to 10")
}

fn date(text: &str) -> Date {
    Date::parse(text).expect("a date")
}

#[test]
fn byte_equal_files_are_the_v1_files() {
    let pairs: [(&[u8], &[u8]); 2] = [
        (
            include_bytes!("../person_directory_v2/shared.rs"),
            include_bytes!("../../person_directory_v1/shared.rs"),
        ),
        (
            include_bytes!("../person_directory_v2/view.rs"),
            include_bytes!("../../person_directory_v1/view.rs"),
        ),
    ];
    for (v2, v1) in pairs {
        assert_eq!(v2, v1);
    }
}

#[test]
fn capabilities_are_the_eight_of_v1_plus_three() {
    assert_eq!(CAPABILITIES.len(), 11);
    assert_eq!(&CAPABILITIES[..8], crate::person_directory_v1::CAPABILITIES);
    assert_eq!(
        &CAPABILITIES[8..],
        ["consent.situation", "purpose.read", "purpose.write"]
    );
}

#[test]
fn the_null_provider_refuses_every_capability() {
    let d: Box<dyn PersonDirectory> = Box::new(NullDirectory);
    assert!(d.capabilities().0.iter().all(|(_, on)| !on));
    let r = block_on(d.purposes());
    assert!(
        matches!(r, Err(Error::Unsupported { ref capability, .. }) if capability == "purpose.read")
    );
    let r = block_on(d.asker());
    assert!(matches!(r, Err(Error::Unsupported { .. })));
    assert_eq!(r.unwrap_err().code(), "unsupported");
}

#[test]
fn k23_date_counts_as_it_is() {
    let got = n(3).after(&When::On(date("2026-05-10")), None, &zone("UTC"), &Fixed);
    assert_eq!(got, Some(date("2029-12-31")));
}

#[test]
fn k24_the_year_is_taken_in_the_stored_zone() {
    let at = When::At(Utc::parse("2026-12-31T23:30:00Z").expect("a time"));
    let instance = zone("Europe/Berlin");
    let year = |z: Option<&Zone>| n(3).after(&at, z, &instance, &Fixed);
    assert_eq!(
        year(Some(&zone("Pacific/Pago_Pago"))),
        Some(date("2029-12-31")),
        "still 2026 there"
    );
    assert_eq!(
        year(Some(&zone("Pacific/Kiritimati"))),
        Some(date("2030-12-31")),
        "already 2027 there"
    );
    assert_eq!(
        year(None),
        Some(date("2030-12-31")),
        "no stored zone: the instance zone (Berlin, 00:30 in 2027)"
    );
    assert_eq!(year(Some(&zone("UTC"))), Some(date("2029-12-31")));
    assert_eq!(
        year(Some(&zone("Nowhere/Land"))),
        None,
        "unknown zone: the provider falls back to the instance zone"
    );
}

/// Eigenes Orakel für K28: eine Kalendertabelle Jahr -> "31.12. (Jahr + N)" als Text; ruft `after` nie auf und merkt
/// sich N nie je Widerruf: es kennt nur (Widerrufsjahr, AKTUELLES N).
fn table(year: i32, years: u8) -> String {
    format!("{:04}-12-31", year + i32::from(years))
}

#[test]
fn k28_property_retain_until_depends_on_year_and_current_n_only() {
    let instance = zone("UTC");
    let mut seed: u64 = 0x2026_1117;
    let mut next = move |m: u64| {
        seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (seed >> 33) % m
    };
    let mut current = ConsentRetention::default();
    let mut withdrawals: Vec<(i32, When)> = Vec::new();
    for _ in 0..400 {
        if next(4) == 0 {
            current = n(u8::try_from(1 + next(10)).expect("small"));
        } else {
            let year = 1970 + i32::try_from(next(231)).expect("small");
            let when = if next(2) == 0 {
                When::On(date(&format!("{year:04}-06-15")))
            } else {
                When::At(Utc::parse(&format!("{year:04}-06-15T12:00:00Z")).expect("a time"))
            };
            withdrawals.push((year, when));
        }
        for (year, when) in &withdrawals {
            let got = current
                .after(when, None, &instance, &Fixed)
                .map(|d| d.canonical());
            assert_eq!(got.as_deref(), Some(table(*year, current.years()).as_str()));
        }
    }
}
