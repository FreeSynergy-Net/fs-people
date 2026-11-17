//! K30–K33 (Aufnahmedauer mitteilen: `RecordingNotice`, b7 17.11.; Pflichtfeld der Einwilligung, im Nachweis gespeichert).
//! Skelett, ignoriert bis der Kern da ist.
//!
//! Pattern: Template method (Fall = feste Schritte gegen `Kern`; Mutant = die Änderung, die den Fall rot macht).
//!
//! Mutanten: K30 Hinweis optional; K31 nur Aufnahmezwecke speichern; K32 Dauer/Fassung beim Speichern verloren;
//! K33 `Days(0)` erlaubt.

use super::kern::{Kern, entry};
use super::pure::block_on;
use crate::draft::person_directory_v2::{
    Error, Purpose, RecordingKeep, RecordingKind, RecordingNotice, When,
};
use fs_types::{BasicValue, Date, Reference};

fn day() -> When {
    When::On(Date::parse("2026-11-17").expect("a date"))
}

fn notice(kind: RecordingKind, keep: RecordingKeep) -> RecordingNotice {
    RecordingNotice {
        kind,
        default_retention: keep,
        text: Reference::parse(
            "urn:sha256:0000000000000000000000000000000000000000000000000000000000000000",
        )
        .expect("an IRI"),
    }
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k30_photo_and_recording_purposes_need_a_matching_notice() {
    let k = Kern::start();
    let p = k.human();
    k.act(&p);
    let no_notice = entry(&p, Purpose::Photo, None, day());
    assert!(matches!(
        block_on(k.d.record_consent(&no_notice)),
        Err(Error::Invalid(_))
    ));
    let mut wrong_kind = entry(&p, Purpose::Photo, None, day());
    wrong_kind.notice = Some(notice(RecordingKind::Audio, RecordingKeep::Days(30)));
    assert!(matches!(
        block_on(k.d.record_consent(&wrong_kind)),
        Err(Error::Invalid(_))
    ));
    // third case: an own purpose with `recording` and no notice (defined with `define_purpose`, see K16)
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k31_notice_on_a_newsletter_is_voluntary_and_kept() {
    let k = Kern::start();
    let p = k.human();
    k.act(&p);
    let mut e = entry(&p, Purpose::Newsletter, None, day());
    e.notice = Some(notice(
        RecordingKind::Transcript,
        RecordingKeep::UntilSituationEnds,
    ));
    let c = block_on(k.d.record_consent(&e)).unwrap();
    assert_eq!(c.entry.notice, e.notice);
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k32_notice_round_trips_through_the_store() {
    let k = Kern::start();
    let p = k.human();
    k.act(&p);
    for keep in [
        RecordingKeep::Days(30),
        RecordingKeep::UntilSituationEnds,
        RecordingKeep::UntilDeletedByHand,
    ] {
        let mut e = entry(&p, Purpose::Photo, None, day());
        e.notice = Some(notice(RecordingKind::Photo, keep));
        block_on(k.d.record_consent(&e)).unwrap();
    }
    k.reopen();
    let all = block_on(k.d.consents_of(&p)).unwrap();
    let keeps: Vec<_> = all
        .iter()
        .map(|c| c.entry.notice.as_ref().expect("notice").default_retention)
        .collect();
    assert_eq!(
        keeps,
        [
            RecordingKeep::Days(30),
            RecordingKeep::UntilSituationEnds,
            RecordingKeep::UntilDeletedByHand
        ]
    );
}

#[test]
#[ignore = "kern: no v2 provider yet"]
fn k33_zero_days_is_invalid() {
    let k = Kern::start();
    let p = k.human();
    k.act(&p);
    let mut e = entry(&p, Purpose::Photo, None, day());
    e.notice = Some(notice(RecordingKind::Photo, RecordingKeep::Days(0)));
    assert!(matches!(
        block_on(k.d.record_consent(&e)),
        Err(Error::Invalid(_))
    ));
}
