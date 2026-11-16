//! `excerpt` of the fake: purpose binding, the consent behind it, the AI rule and the birthday fields.
//!
//! Pattern: Fake (the rules are those written in the doc comment of `PersonDirectory::excerpt`).

use super::{Defect, FakeDirectory, Status};
use crate::person_directory_v1::{
    Birthday, ConsentQuery, Error, Excerpt, ExcerptField, ExcerptRequest, ExcerptValue, MonthDay,
    NoExcerptReason, PersonArt, PersonRef, Problem, Purpose, Result,
};

/// The value of `field` from what a row stores; the birthday fields convert (with year, without year).
fn pick(values: &[ExcerptValue], field: ExcerptField) -> Option<ExcerptValue> {
    let stored = |f| values.iter().find(|v| v.field() == f);
    let month_day = || match (
        stored(ExcerptField::BirthMonthDay),
        stored(ExcerptField::Birthday),
    ) {
        (Some(ExcerptValue::BirthMonthDay(m)), _)
        | (_, Some(ExcerptValue::Birthday(Birthday::MonthDay(m)))) => Some(*m),
        (_, Some(ExcerptValue::Birthday(Birthday::Date(d)))) => {
            MonthDay::new(d.month(), d.day()).ok()
        }
        _ => None,
    };
    match field {
        ExcerptField::BirthMonthDay => month_day().map(ExcerptValue::BirthMonthDay),
        ExcerptField::Birthday => match stored(ExcerptField::Birthday) {
            Some(v) => Some(v.clone()),
            None => month_day().map(|m| ExcerptValue::Birthday(Birthday::MonthDay(m))),
        },
        other => stored(other).cloned(),
    }
}

impl FakeDirectory {
    pub(super) fn do_excerpt(&self, person: &PersonRef, req: &ExcerptRequest) -> Result<Excerpt> {
        let mut g = self.lock();
        let ai = self.asker_is_ai(&g);
        if ai && req.purpose != Purpose::AiAssistance && !g.bad(Defect::AiPurposeLeak) {
            return Err(Error::Forbidden {
                capability: "person.excerpt".to_owned(),
            });
        }
        if req.fields.is_empty() {
            return Err(Error::Invalid(vec![Problem {
                field: "fields".to_owned(),
                reason: "empty".to_owned(),
            }]));
        }
        let captured = g.now();
        let refuse = |reason| {
            Err(Error::NoExcerpt {
                person: person.clone(),
                reason,
            })
        };
        let Some(e) = g.target(person) else {
            return refuse(NoExcerptReason::Unknown);
        };
        match e.status {
            Status::Deleted(_) => return refuse(NoExcerptReason::Deleted),
            Status::NotShared => return refuse(NoExcerptReason::NotShared),
            Status::Active => {}
        }
        let mut consent = None;
        let checked = req.purpose.needs_consent() && !g.bad(Defect::IgnoreConsentInExcerpt);
        if checked && matches!(e.person.art, PersonArt::Person | PersonArt::Profile) {
            let query = ConsentQuery {
                person: person.clone(),
                purpose: req.purpose.clone(),
                scope: req.scope.clone(),
                recipient: req.recipient.clone(),
            };
            match g.active_match(&query) {
                Some(id) => consent = Some(id),
                None => return refuse(NoExcerptReason::NoConsent),
            }
        }
        let mut values: Vec<ExcerptValue> = Vec::new();
        let mut missing = Vec::new();
        for f in &req.fields {
            if values.iter().any(|v| v.field() == *f) || missing.contains(f) {
                continue;
            }
            match pick(&e.excerpt, *f)
                .filter(|_| !(ai && f.sensitive() && !g.bad(Defect::AiSeesSensitive)))
            {
                Some(v) => values.push(v),
                None => missing.push(*f),
            }
        }
        Ok(Excerpt {
            person: e.view().person,
            source: e.person.id.clone(),
            captured,
            purpose: req.purpose.clone(),
            consent,
            values,
            missing,
        })
    }
}
