//! Anpassung 6 im Vertrag: `ConsentRetention` (Wert), die Berechnung von `retain_until` mit RÜCKWIRKUNG (b7 17.11.,
//! Antwort 6: eine geänderte Dauer gilt auch für bestehende Nachweise) und der Einstellungs-Port mit Vorschau und
//! Bestätigung für die Verkürzung (b7 Entscheidung 11: keine harte Untergrenze, aber Zahl, Hinweis, Bestätigung).
//!
//! Pattern: Value Object (`ConsentRetention`) + Port (`ZoneOffsets`) + Repository-Teil (`RetentionSettings`; die
//! Einstellung ist KEINE Methode von `PersonDirectory`, Verbraucher brauchen sie nicht).
//!
//! `crate::retention` bleibt in der Crate als `pub use` auf diese Datei, sobald v2 v1 ersetzt (Entwurf 2.4).
//! Keine Rechtsberatung: der Standard 3 Jahre folgt der Nutzerantwort 40.

use super::{Date, Result, When};
use async_trait::async_trait;
use fs_types::{BasicValue, Utc, Zone};

/// Jahre, die der Nachweis eines widerrufenen Consents ab Jahresende des Widerrufs aufbewahrt wird.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ConsentRetention(u8);

impl ConsentRetention {
    pub const DEFAULT_YEARS: u8 = 3;
    pub const MIN_YEARS: u8 = 1;
    pub const MAX_YEARS: u8 = 10;

    /// `MIN_YEARS` bis `MAX_YEARS`; sonst `None`.
    #[must_use]
    pub fn new(years: u8) -> Option<Self> {
        (Self::MIN_YEARS..=Self::MAX_YEARS)
            .contains(&years)
            .then_some(Self(years))
    }

    #[must_use]
    pub fn years(self) -> u8 {
        self.0
    }

    /// 31.12. von (Jahr des Widerrufs + N). `When::On` ist ein Datum und zählt, wie es ist. `When::At` ist ein
    /// Zeitpunkt: das Jahr zählt in `zone` (die GESPEICHERTE Zone des Widerrufs), ohne sie in `instance` (Zone der
    /// Instanz); die Verschiebung kommt aus `offsets` (tz-Datenbank beim Anbieter). `None` nur, wenn die Zone
    /// unbekannt ist (der Anbieter nimmt dann die Instanz-Zone) oder der Zeitpunkt außerhalb des Bereichs liegt.
    #[must_use]
    pub fn after(
        self,
        when: &When,
        zone: Option<&Zone>,
        instance: &Zone,
        offsets: &dyn ZoneOffsets,
    ) -> Option<Date> {
        let year = match when {
            When::On(day) => day.year(),
            When::At(at) => {
                let off = offsets.offset_minutes(zone.unwrap_or(instance), *at)?;
                let local = Utc::from_millis(at.millis().checked_add(i64::from(off) * 60_000)?);
                local.to_rfc3339()?.get(..4)?.parse().ok()?
            }
        };
        Date::parse(&format!("{:04}-12-31", year + i32::from(self.0))).ok()
    }
}

impl Default for ConsentRetention {
    fn default() -> Self {
        Self(Self::DEFAULT_YEARS)
    }
}

/// Port: Verschiebung einer Zone gegen UTC zu einem Zeitpunkt (Sommerzeit eingerechnet).
pub trait ZoneOffsets: Send + Sync {
    /// Minuten, die die Ortszeit der Zone zu `at` vor UTC liegt; `None` = Zone unbekannt.
    fn offset_minutes(&self, zone: &Zone, at: Utc) -> Option<i32>;
}

/// Was eine Änderung der Aufbewahrung bewirkt (Vorschau, ändert nichts).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetentionPreview {
    /// Anzahl bestehender widerrufener Nachweise, deren `retain_until` sich ändert.
    pub recomputed: u32,
    /// Anzahl der Nachweise, die beim nächsten Aufräumlauf gelöscht würden (nur bei Verkürzung > 0).
    pub to_delete: u32,
    /// Hinweis "kann rechtlich noch nötig sein, keine Rechtsberatung" (MF2 `people-consent-retention-hint`) zeigen:
    /// wahr genau dann, wenn `to_delete > 0`. Das Fenster zeigt ihn vor der Bestätigung.
    pub legal_hint: bool,
}

/// Die Einstellung "Aufbewahrung nach Widerruf" (`consent_retention`, Recht `people.settings`).
#[async_trait]
pub trait RetentionSettings: Send + Sync {
    /// Die gültige Einstellung; ohne Zeile 3 Jahre.
    ///
    /// # Errors
    /// `Error::Unavailable`.
    async fn consent_retention(&self) -> Result<ConsentRetention>;
    /// Vorschau auf die Änderung auf `new`.
    ///
    /// # Errors
    /// `Forbidden{people.settings}` ohne Recht; `Unavailable`.
    async fn preview_retention(&self, new: ConsentRetention) -> Result<RetentionPreview>;
    /// Einstellung schreiben UND `retain_until` aller bestehenden widerrufenen Nachweise neu berechnen, in EINEM
    /// Schreibschritt (K25c), mit Protokolleintrag "Aufbewahrung N→M, k Nachweise". Verkürzung (`to_delete > 0`)
    /// braucht `confirmed = true`, sonst `Invalid` und nichts geschrieben (K25b). Antwort: die Vorschau, die galt.
    ///
    /// # Errors
    /// `Forbidden`, `Invalid` (Bestätigung fehlt), `Unavailable`.
    async fn set_consent_retention(
        &self,
        new: ConsentRetention,
        confirmed: bool,
    ) -> Result<RetentionPreview>;
}
