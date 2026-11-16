//! Rules for what an AI sees (views, addresses, suggestions) and for the birthday fields.
//!
//! Pattern: Template method (rules as plain checks; the AI and its view come from the `ConformanceFixture`).

use super::consent_rules::forbidden;
use super::{ConformanceFixture, Report, entry as proposal, purposed};
use crate::person_directory_v1::{
    Birthday, ExcerptField, ExcerptValue, Matched, MonthDay, PersonArt, PersonDirectory, PersonRef,
    Purpose, Resolved, Suggest,
};
use fs_types::{BasicValue, ContactAddress, Date};

pub(super) async fn view_rules(
    admin: &dyn PersonDirectory,
    bot: &dyn PersonDirectory,
    t: &PersonRef,
    addr: &str,
    r: &mut Report,
) {
    let Ok(wanted) = ContactAddress::parse(addr) else {
        return;
    };
    r.is_err(
        &bot.find_by_address(&wanted).await,
        "AI: find_by_address is Forbidden",
        forbidden,
    );
    let with = proposal("Zzconf Ai Hint", Some("mailto:ai.hint.zzconf@example.org"));
    r.is_err(
        &bot.propose(&with, None).await,
        "AI: propose with an address is Forbidden",
        forbidden,
    );
    let without = proposal("Zzconf Ai Plain", None);
    r.ensure(
        bot.propose(&without, None).await.is_ok(),
        "AI: propose without an address is allowed",
    );
    let seen = bot.resolve(t).await;
    r.ensure(
        matches!(&seen, Ok(Resolved::Found(v)) if v.address.is_none()),
        "AI: PersonView.address is None",
    );
    r.ensure(
        matches!(bot.resolve_many(std::slice::from_ref(t)).await.as_deref(), Ok([Resolved::Found(v)]) if v.address.is_none()),
        "AI: resolve_many shows no address either",
    );
    let human = admin.resolve(t).await;
    r.ensure(
        matches!(&human, Ok(Resolved::Found(v)) if v.address.is_some()),
        "resolve: a human sees the address (control)",
    );
    nameless_rules(admin, bot, r).await;
    let q = Suggest {
        text: "ai.target.zzconf".to_owned(),
        arten: vec![],
        limit: 50,
        include_proposed: true,
    };
    let bots = bot.suggest(&q).await.unwrap_or_default();
    r.ensure(
        bots.iter()
            .all(|c| c.matched != Matched::Address && c.view.person.id != t.id),
        "AI: suggest never searches addresses, Matched::Address never occurs",
    );
    let humans = admin.suggest(&q).await.unwrap_or_default();
    r.ensure(
        humans
            .iter()
            .any(|c| c.view.person.id == t.id && c.matched == Matched::Address),
        "suggest: a human finds by address (control)",
    );
}

async fn nameless_rules(admin: &dyn PersonDirectory, bot: &dyn PersonDirectory, r: &mut Report) {
    let hint = proposal("", Some("mailto:nameless.zzconf@example.org"));
    let Ok(n) = admin.propose(&hint, None).await else {
        return;
    };
    let ai = bot.resolve(&n).await;
    r.ensure(
        matches!(&ai, Ok(Resolved::Found(v)) if v.display.is_empty() && v.sort_key.is_empty()),
        "AI: display and sort_key have no address stage (empty, the display shows a placeholder)",
    );
    let human = admin.resolve(&n).await;
    r.ensure(
        matches!(&human, Ok(Resolved::Found(v)) if v.display.contains("nameless")),
        "resolve: a human gets the address as the last stage of the display (control)",
    );
}

pub(super) async fn birthday_rules(
    fx: &dyn ConformanceFixture,
    admin: &dyn PersonDirectory,
    t: &PersonRef,
    r: &mut Report,
) {
    let fields = [
        ExcerptField::Birthday,
        ExcerptField::BirthMonthDay,
        ExcerptField::Birthday,
    ];
    match admin.excerpt(t, &purposed(&fields, Purpose::OwnUse)).await {
        Ok(x) => {
            let day = MonthDay::new(7, 4).ok();
            let full = Date::parse("1990-07-04").ok().map(Birthday::Date);
            r.ensure(
                x.values.len() == 2,
                "excerpt: a field asked twice counts once",
            );
            r.ensure(
                x.values
                    .iter()
                    .any(|v| matches!(v, ExcerptValue::Birthday(b) if Some(*b) == full)),
                "excerpt: Birthday with a known year is a Date",
            );
            r.ensure(
                x.values
                    .iter()
                    .any(|v| matches!(v, ExcerptValue::BirthMonthDay(m) if Some(*m) == day)),
                "excerpt: BirthMonthDay is always without a year",
            );
        }
        Err(_) => r.ensure(
            false,
            "excerpt: birthday fields for an entry with a birthday",
        ),
    }
    let Some(leap) = fx
        .add_confirmed(PersonArt::Person, "Zzconf Leap", None)
        .await
    else {
        return;
    };
    let Ok(feb) = MonthDay::new(2, 29) else {
        return;
    };
    if !fx.set_birthday(&leap, Birthday::MonthDay(feb)).await {
        return;
    }
    let ask = purposed(
        &[
            ExcerptField::Birthday,
            ExcerptField::BirthMonthDay,
            ExcerptField::Phone,
        ],
        Purpose::OwnUse,
    );
    match admin.excerpt(&leap, &ask).await {
        Ok(x) => {
            r.ensure(
                x.values.iter().any(
                    |v| matches!(v, ExcerptValue::Birthday(Birthday::MonthDay(m)) if *m == feb),
                ),
                "excerpt: a birthday without a year is a MonthDay, 29 February included",
            );
            r.ensure(
                x.missing == vec![ExcerptField::Phone],
                "excerpt: only the unknown field is missing",
            );
        }
        Err(_) => r.ensure(false, "excerpt: a birthday without a year"),
    }
}
