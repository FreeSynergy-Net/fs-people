//! Rules for an AI that asks (ADR 0054, 0076: a person with the AI mark) and for the birthday fields.
//!
//! Pattern: Template method (rules as plain checks; the AI and its view come from the `ConformanceFixture`).

use super::consent_rules::{entry, forbidden, scope, withdrawal};
use super::{ConformanceFixture, Report, purposed};
use crate::person_directory_v1::{
    Birthday, Error, ExcerptField, ExcerptValue, NoExcerptReason, PersonArt, PersonDirectory,
    PersonRef, Purpose,
};
use fs_types::{BasicValue, Date, ObjectId};

const SENSITIVE: [ExcerptField; 5] = [
    ExcerptField::Birthday,
    ExcerptField::BirthMonthDay,
    ExcerptField::Email,
    ExcerptField::Postal,
    ExcerptField::Phone,
];

pub(super) async fn rules(fx: &dyn ConformanceFixture, r: &mut Report) {
    let Some(admin) = fx.admin_view().await else {
        return;
    };
    let Some(ai) = fx.add_ai("Zzconf Robot").await else {
        r.skip("AI: fixture cannot add_ai");
        return;
    };
    let all = ["consent.read", "consent.check", "consent.write"];
    let Some(bot) = fx.member_view(&ai, &all).await else {
        r.skip("AI: fixture has no member view");
        return;
    };
    let addr = "mailto:ai.target.zzconf@example.org";
    let target = fx
        .add_confirmed(PersonArt::Person, "Zzconf Ai Target", Some(addr))
        .await;
    let Some(t) = target else { return };
    let born = Date::parse("1990-07-04").map(Birthday::Date);
    if !fx
        .set_birthday(
            &t,
            born.unwrap_or(Birthday::Date(
                Date::parse("2000-01-01").unwrap_or_else(|_| unreachable!()),
            )),
        )
        .await
    {
        r.skip("AI: fixture cannot set a birthday");
        return;
    }
    consent_rules(&*admin, &*bot, &t, r).await;
    excerpt_rules(&*admin, &*bot, &t, r).await;
    super::ai_views::view_rules(&*admin, &*bot, &t, addr, r).await;
    super::ai_views::birthday_rules(fx, &*admin, &t, r).await;
}

async fn consent_rules(
    admin: &dyn PersonDirectory,
    bot: &dyn PersonDirectory,
    t: &PersonRef,
    r: &mut Report,
) {
    let Ok(c) = admin.record_consent(&entry(t, Purpose::Newsletter)).await else {
        return;
    };
    let q = super::consent_rules::query(t, Purpose::Newsletter);
    r.is_err(
        &bot.consents_of(t).await,
        "AI: consents_of is Forbidden",
        forbidden,
    );
    r.is_err(
        &bot.check_consent(&q).await,
        "AI: check_consent is Forbidden",
        forbidden,
    );
    r.is_err(
        &bot.record_consent(&entry(t, Purpose::Photo)).await,
        "AI: record_consent is Forbidden",
        forbidden,
    );
    r.is_err(
        &bot.withdraw_consent(c.id, &withdrawal()).await,
        "AI: withdraw_consent is Forbidden",
        forbidden,
    );
    let unknown = bot
        .withdraw_consent(ObjectId([0xCD; 16]), &withdrawal())
        .await;
    r.is_err(
        &unknown,
        "AI: withdraw_consent of an unknown id is Forbidden, not UnknownConsent",
        forbidden,
    );
}

async fn excerpt_rules(
    admin: &dyn PersonDirectory,
    bot: &dyn PersonDirectory,
    t: &PersonRef,
    r: &mut Report,
) {
    let own = purposed(&[ExcerptField::FullName], Purpose::OwnUse);
    r.is_err(
        &bot.excerpt(t, &own).await,
        "AI: excerpt for another purpose than AiAssistance is Forbidden",
        forbidden,
    );
    let empty = purposed(&[], Purpose::Photo);
    r.is_err(
        &bot.excerpt(t, &empty).await,
        "AI: the purpose check comes before the field check",
        forbidden,
    );
    let nobody = PersonRef {
        id: scope(0xbeef).unwrap_or_else(|| t.id.clone()),
        art: PersonArt::Person,
        abzug: None,
    };
    let gone = bot.excerpt(&nobody, &own).await;
    r.is_err(
        &gone,
        "AI: the purpose check comes before the lookup (no leak through Unknown)",
        forbidden,
    );
    let mut ask = purposed(
        &[
            ExcerptField::FullName,
            SENSITIVE[0],
            SENSITIVE[1],
            SENSITIVE[2],
            SENSITIVE[3],
            SENSITIVE[4],
        ],
        Purpose::AiAssistance,
    );
    ask.scope = scope(9);
    let without = bot.excerpt(t, &ask).await;
    r.is_err(
        &without,
        "AI: AiAssistance without the consent of the person is NoConsent",
        |e| {
            matches!(
                e,
                Error::NoExcerpt {
                    reason: NoExcerptReason::NoConsent,
                    ..
                }
            )
        },
    );
    let mut room = entry(t, Purpose::AiAssistance);
    room.scope = scope(9);
    if admin.record_consent(&room).await.is_err() {
        return;
    }
    match bot.excerpt(t, &ask).await {
        Ok(x) => {
            r.ensure(
                x.values.len() == 1 && matches!(x.values[0], ExcerptValue::FullName(_)),
                "AI: an excerpt has only non-sensitive values",
            );
            r.ensure(
                SENSITIVE.iter().all(|f| x.missing.contains(f)),
                "AI: sensitive fields are in missing, without a trace",
            );
        }
        Err(_) => r.ensure(
            false,
            "AI: AiAssistance with a consent for the room gives an excerpt",
        ),
    }
    let human = admin.excerpt(t, &ask).await;
    r.ensure(
        human.is_ok_and(|x| {
            x.values.iter().any(|v| v.field() == ExcerptField::Birthday)
                && x.values
                    .iter()
                    .any(|v| v.field() == ExcerptField::BirthMonthDay)
        }),
        "excerpt: a human gets the birthday (control for the AI rule)",
    );
}
