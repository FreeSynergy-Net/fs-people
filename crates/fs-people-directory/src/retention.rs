//! The retention of the proof of a withdrawn consent: how many years after the end of the year of the withdrawal it
//! is kept (`Consent::retain_until`). The setting `consent_retention` of People holds the number; this type is the
//! value and the one calculation, so the real provider and the fake agree.
//!
//! Pattern: Value Object (the years, checked on construction; `after` is the only calculation).
//!
//! Not legal advice: the default of 3 years follows the user's answer (16.11., question 40) and the usual limitation
//! period of civil claims; whether a longer period is needed for a given operator is the operator's decision.

use fs_types::BasicValue;

use crate::person_directory_v1::{Date, When};

/// Years the proof of a withdrawn consent is kept, counted from the end of the year of the withdrawal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ConsentRetention(u8);

impl ConsentRetention {
    /// The default: 3 years from the end of the year (user answer 40).
    pub const DEFAULT_YEARS: u8 = 3;
    /// The least: 1 year (a shorter proof is worth nothing against a claim within the year).
    pub const MIN_YEARS: u8 = 1;
    /// The most: 10 years (longer is hoarding personal data, which the same law forbids).
    pub const MAX_YEARS: u8 = 10;

    /// A period in years, `MIN_YEARS` to `MAX_YEARS`; `None` outside that range.
    #[must_use]
    pub fn new(years: u8) -> Option<Self> {
        (Self::MIN_YEARS..=Self::MAX_YEARS)
            .contains(&years)
            .then_some(Self(years))
    }

    /// The years.
    #[must_use]
    pub fn years(self) -> u8 {
        self.0
    }

    /// The day until which the proof of a withdrawal at `when` is kept: 31 December, `years` after the year of
    /// `when` (`When::On` is a day, `When::At` an instant in UTC). `None` only for an instant out of range.
    #[must_use]
    pub fn after(self, when: &When) -> Option<Date> {
        let year = match when {
            When::On(day) => day.year(),
            When::At(at) => at.to_rfc3339()?.get(..4)?.parse().ok()?,
        };
        Date::parse(&format!("{:04}-12-31", year + i32::from(self.0))).ok()
    }
}

impl Default for ConsentRetention {
    fn default() -> Self {
        Self(Self::DEFAULT_YEARS)
    }
}
