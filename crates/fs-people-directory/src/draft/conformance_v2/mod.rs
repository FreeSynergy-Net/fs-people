//! Konformitätsfälle K1–K33 von `person_directory_v2` (Entwurf Abschnitt 4 plus b7-Entscheidungen 11/12): je Regel ein
//! Fall und der Mutant, der ihn rot macht. Skelett der Probe, kein Anbieter.
//!
//! Pattern: Template method (feste Fälle gegen einen `PersonDirectory`-Anbieter; `kern::Kern` sagt, wie man ihn in die
//! Zustände der Fälle bringt).
//!
//! Was LÄUFT: `pure` (byte-gleiche v1-Dateien, Fähigkeitsliste, Null-Anbieter als `dyn`, K23, K24 in drei Zonen, K28 als
//! Property-Test mit eigenem Orakel gegen `ConsentRetention::after`). Was `#[ignore = "kern: ..."]` trägt, braucht den
//! v2-Anbieter (Fake mit einstellbarer Uhr oder `PeopleDirectory`, Bahn Kern): `Kern::start` ist `unimplemented!`, ein
//! ignorierter Fall wird dadurch nie still grün. Ist der Anbieter da, fällt das `ignore` weg und jeder Fall muss rot
//! werden, wenn man seinen Mutanten einbaut (Mutant-Lauf der Probe-Folge).
//!
//! Zuordnung: `rules_share` K1–K8, `rules_situation` K9–K14, `rules_purposes` K15–K22, `rules_retention` K23–K29,
//! `rules_notice` K30–K33. Orakel-Regel: K8 und K28 haben ein EIGENES Orakel (Mengenmodell, Kalendertabelle); ein Orakel,
//! das die Anbieter-Prüfung nachbaut, ist verboten.

// The cases are flat lists of steps on a few persons of ONE case; short names are the persons.
#![allow(clippy::many_single_char_names, clippy::similar_names)]

mod kern;
mod pure;
mod rules_notice;
mod rules_purposes;
mod rules_retention;
mod rules_share;
mod rules_situation;
