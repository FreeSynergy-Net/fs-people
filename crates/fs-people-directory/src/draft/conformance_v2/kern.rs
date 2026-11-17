//! Die Fassung des Anbieters für die Fälle: Personen jeder Art, Freigaben, Uhr, Fehler einspritzen, Aufräumlauf.
//!
//! Pattern: Strategy / Test fixture (jeder Anbieter sagt, wie man ihn in die Zustände der Fälle bringt; hier nur die
//! Form, ohne Anbieter: alles `unimplemented!`, bis der Kern da ist).

#![allow(dead_code)]

use crate::draft::person_directory_v2::retention::ZoneOffsets;
use crate::draft::person_directory_v2::{
    ConsentEntry, ConsentOrigin, ConsentRetention, ExcerptField, ExcerptRequest, ObjectId,
    PersonDirectory, PersonRef, Purpose, PurposeCatalog, Reference, RetentionSettings, ShareField,
    Utc, When,
};
use fs_types::BasicValue;

/// Grund für `unimplemented!`: ein ignorierter Fall darf nie still grün werden.
pub(super) const KERN: &str =
    "kern: the v2 provider (fake with settable clock / PeopleDirectory) does not exist yet";

/// Der Anbieter unter Test plus die Dinge, die der Trait nicht kann.
pub(super) struct Kern {
    pub(super) d: Box<dyn PersonDirectory>,
    pub(super) c: Box<dyn PurposeCatalog>,
}

impl Kern {
    pub(super) fn start() -> Self {
        unimplemented!("{KERN}")
    }
    pub(super) fn human(&self) -> PersonRef {
        unimplemented!("{KERN}")
    }
    pub(super) fn agent(&self) -> PersonRef {
        unimplemented!("{KERN}")
    }
    /// Die fragende Person wechseln (Eigenschaft des Anbieters).
    pub(super) fn act(&self, _who: &PersonRef) {
        unimplemented!("{KERN}")
    }
    /// Ein Anbieter, der als Programm-Konto (fs-groups, fs-messages) mit `consent.situation` fragt.
    pub(super) fn as_program(&self) -> Box<dyn PersonDirectory> {
        unimplemented!("{KERN}")
    }
    /// Die Uhr des Anbieters stellen (K10, K24, K25).
    pub(super) fn set_clock(&self, _now: Utc) {
        unimplemented!("{KERN}")
    }
    /// Instanz-Zone und Verschiebungen (tz-Datenbank oder feste Tabelle des Fakes).
    pub(super) fn zones(&self) -> &dyn ZoneOffsets {
        unimplemented!("{KERN}")
    }
    /// Eigene Kontaktkarte der Person (Anschrift, E-Mail, Telefon, Geburtstag) anlegen; liefert ihren Verweis.
    pub(super) fn own_card(&self, _owner: &PersonRef) -> PersonRef {
        unimplemented!("{KERN}")
    }
    /// Profil der anderen Person, mit Werten in allen sensiblen Feldern.
    pub(super) fn foreign_profile(&self, _of: &PersonRef) -> PersonRef {
        unimplemented!("{KERN}")
    }
    /// Freigabe (fs-access) anlegen; `expires` = Ablauf.
    pub(super) fn share(
        &self,
        _owner: &PersonRef,
        _field: ShareField,
        _to: &PersonRef,
        _expires: Option<Utc>,
    ) -> ObjectId {
        unimplemented!("{KERN}")
    }
    pub(super) fn revoke_share(&self, _share: ObjectId) {
        unimplemented!("{KERN}")
    }
    /// Der Port zur Freigabe antwortet `Unreachable` (K5).
    pub(super) fn share_port_down(&self, _down: bool) {
        unimplemented!("{KERN}")
    }
    /// Zähler: wie oft fragte der Anbieter den Port (K1: nie für eigene Einträge).
    pub(super) fn share_port_calls(&self) -> usize {
        unimplemented!("{KERN}")
    }
    /// Die Person darf `purpose.write`/`consent.write` (Verwaltung, Datenschutz) oder nicht.
    pub(super) fn grant_admin(&self, _who: &PersonRef, _yes: bool) {
        unimplemented!("{KERN}")
    }
    /// Aufräumlauf des Anbieters (löscht Nachweise nach `retain_until`, hält Widerrufe `SituationEnded` fest).
    pub(super) fn cleanup(&self) {
        unimplemented!("{KERN}")
    }
    /// Das nächste Schreiben der Einstellung scheitert mitten in der Neuberechnung (K25c).
    pub(super) fn fail_mid_recompute(&self) {
        unimplemented!("{KERN}")
    }
    pub(super) fn settings(&self) -> &dyn RetentionSettings {
        unimplemented!("{KERN}")
    }
    /// Zähler der Protokolleinträge "Aufbewahrung N→M, k Nachweise" (K25).
    pub(super) fn retention_log(&self) -> Vec<String> {
        unimplemented!("{KERN}")
    }
    /// Speicher neu öffnen (K32: Rundreise über den Speicher).
    pub(super) fn reopen(&self) {
        unimplemented!("{KERN}")
    }
}

/// Anfrage mit einem Feld und einem Zweck.
pub(super) fn req(field: ExcerptField, purpose: Purpose) -> ExcerptRequest {
    ExcerptRequest {
        fields: vec![field],
        purpose,
        scope: None,
        recipient: None,
    }
}

/// Eine Einwilligung der Person für den Zweck, ohne Laufzeit und Hinweis (die Fälle setzen, was sie brauchen).
pub(super) fn entry(
    person: &PersonRef,
    purpose: Purpose,
    scope: Option<&str>,
    given: When,
) -> ConsentEntry {
    ConsentEntry {
        person: person.clone(),
        given_by: None,
        purpose,
        scope: scope.and_then(|s| Reference::parse(s).ok()),
        recipient: None,
        given,
        origin: ConsentOrigin::ByPerson,
        evidence: None,
        text: None,
        lifetime: None,
        notice: None,
    }
}

/// Standard-Einstellung der Aufbewahrung für Fälle, die nichts anderes sagen.
pub(super) fn default_retention() -> ConsentRetention {
    ConsentRetention::default()
}
