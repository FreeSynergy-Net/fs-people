//! The sentences the form says about what it refuses (birthday, categories, roles).
//!
//! Pattern: Factory over the same MF2 files as `words.rs`; one function per sentence, so a caller
//! cannot spell an id wrong.

use fs_components_words::Args;

use crate::words::{say, t};

pub(crate) fn birthday() -> String {
    t("e-birthday")
}
pub(crate) fn birthday_place() -> String {
    t("e-birthday-place")
}
pub(crate) fn category() -> String {
    t("e-category")
}
pub(crate) fn category_lift() -> String {
    t("e-category-lift")
}
pub(crate) fn role_ai() -> String {
    t("e-role-ai")
}
pub(crate) fn role_unknown(role: &str) -> String {
    say("e-role-unknown", &Args::new().text("role", role))
}
pub(crate) fn role_group(role: &str) -> String {
    say("e-role-group", &Args::new().text("role", role))
}
pub(crate) fn role_personal(role: &str) -> String {
    say("e-role-personal", &Args::new().text("role", role))
}
pub(crate) fn role_partial(role: &str, why: &str) -> String {
    say(
        "e-role-partial",
        &Args::new().text("role", role).text("why", why),
    )
}
pub(crate) fn role_create() -> String {
    t("e-role-create")
}
pub(crate) fn role_empty() -> String {
    t("e-role-empty")
}
