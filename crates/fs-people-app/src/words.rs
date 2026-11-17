//! The words of the program: `MessageFormat` 2 files in German and English (ADR 0057), picked by the
//! language of the session (`LC_ALL`, `LC_MESSAGES`, `LANG`; else English).
//!
//! Pattern: Factory over embedded `.mf2` files; one `Mf2Texts` per process. Texts with a place for a
//! value keep `{}` (one value) or `{name}` after resolving, so the callers fill them as before.

use std::sync::OnceLock;

use fs_components_mf2::{Mf2Texts, Source};
use fs_components_words::{Args, Locale, Words};
use fs_components_workbenches::people::PeopleLabels;

/// The two language files; `fs_components_mf2::check_pairs(TEXTS)` must be empty (a test asserts it).
pub const TEXTS: &[Source] = &[
    Source {
        language: "en",
        text: include_str!("texts/en.mf2"),
    },
    Source {
        language: "de",
        text: include_str!("texts/de.mf2"),
    },
];

/// The language of this session.
#[must_use]
pub fn session_locale() -> Locale {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .filter_map(|name| std::env::var(name).ok())
        .find_map(|value| Locale::from_posix(&value))
        .unwrap_or_else(Locale::english)
}

/// The words in `locale`.
#[must_use]
pub fn words(locale: &Locale) -> Mf2Texts {
    Mf2Texts::new(TEXTS, locale)
}

fn session() -> &'static Mf2Texts {
    static TEXTS_OF_SESSION: OnceLock<Mf2Texts> = OnceLock::new();
    TEXTS_OF_SESSION.get_or_init(|| words(&session_locale()))
}

/// The text `id`, in the language of the session.
pub(crate) fn t(id: &str) -> String {
    session().plain(id)
}

/// The text `id` with the value left as `{}` for the caller to fill.
pub(crate) fn tpl(id: &str) -> String {
    say(id, &Args::new().text("v", "{}"))
}

pub(crate) fn say(id: &str, args: &Args) -> String {
    session().say(&id.into(), args).text().to_owned()
}

/// The title of the window.
pub fn title() -> String {
    t("app-name")
}

/// Editing is locked: the source cannot compare and write in one step (`people/no-checked-update`).
pub(crate) fn no_checked_update() -> String {
    t("no-checked-update")
}

/// The words of the workbench, in the language of the session.
pub fn labels() -> PeopleLabels {
    labels_with(session())
}

/// The words of the workbench from `texts`.
#[must_use]
pub fn labels_with(texts: &Mf2Texts) -> PeopleLabels {
    let t = |id: &str| texts.plain(id);
    let tpl = |id: &str| {
        texts
            .say(&id.into(), &Args::new().text("v", "{}"))
            .text()
            .to_owned()
    };
    let mut l = PeopleLabels::default();
    l.search = t("search");
    l.columns = vec![t("col-name")];
    l.more = t("more");
    l.create = t("create");
    l.edit = t("edit");
    l.save = t("save");
    l.cancel = t("cancel");
    l.delete = t("delete");
    l.delete_really = t("delete-really");
    l.delete_keep = t("delete-keep");
    l.add_field = tpl("add-field");
    l.nothing_selected = t("nothing-selected");
    l.nobody = t("nobody");
    l.nobody_found = tpl("nobody-found");
    l.delete_confirm = tpl("delete-confirm");
    l.gone = t("gone");
    l.unsaved = t("unsaved");
    l.discard_confirm = t("discard-confirm");
    l.discard_keep = t("discard-keep");
    l.discard_really = t("discard-really");
    l.saved = t("saved");
    l.created = t("created");
    l.deleted = tpl("deleted");
    l.no_changes = t("no-changes");
    l.not_saved = tpl("not-saved");
    l.fix_fields = t("fix-fields");
    l.email_placeholder = t("email-placeholder");
    l.phone_placeholder = t("phone-placeholder");
    l.not_deleted = tpl("not-deleted");
    l.cannot_edit = tpl("cannot-edit");
    l.unreadable = tpl("unreadable");
    l.list_failed = tpl("list-failed");
    l.conflict_confirm = tpl("conflict-confirm");
    l
}

/// Headings and labels of what is shown about a person (functions, because the language is the session's).
pub(crate) mod field {
    use super::{Args, say, t, tpl};

    pub fn ai() -> String {
        t("f-ai")
    }
    pub fn sensitive() -> String {
        tpl("f-sensitive")
    }
    pub fn sensitive_row() -> String {
        t("f-sensitive-row")
    }
    pub fn unclear() -> String {
        t("f-unclear")
    }
    pub fn contact() -> String {
        t("f-contact")
    }
    pub fn work() -> String {
        t("f-work")
    }
    pub fn places() -> String {
        t("f-places")
    }
    pub fn about() -> String {
        t("f-about")
    }
    pub fn roles() -> String {
        t("f-roles")
    }
    pub fn kept() -> String {
        t("f-kept")
    }
    pub fn name() -> String {
        t("f-name")
    }
    pub fn email() -> String {
        t("f-email")
    }
    pub fn phone() -> String {
        t("f-phone")
    }
    pub fn organisation() -> String {
        t("f-organisation")
    }
    pub fn job() -> String {
        t("f-job")
    }
    pub fn address() -> String {
        t("f-address")
    }
    pub fn nickname() -> String {
        t("f-nickname")
    }
    pub fn link() -> String {
        t("f-link")
    }
    pub fn note() -> String {
        t("f-note")
    }
    pub fn kind() -> String {
        t("f-kind")
    }
    pub fn role() -> String {
        t("f-role")
    }
    pub fn other() -> String {
        t("f-other")
    }
    pub fn unreadable() -> String {
        tpl("f-unreadable")
    }
    /// A section the source does not offer: said, never left out (code, capability and source named).
    pub fn unsupported(capability: &str, provider: &str) -> String {
        say(
            "f-unsupported",
            &Args::new()
                .text("capability", capability)
                .text("provider", provider),
        )
    }
}

/// What is wrong with a typed contact value, as a sentence (one per code, ADR 0057).
pub(crate) fn problem(problem: fs_people_core::values::ValueProblem) -> String {
    use fs_people_core::values::ValueProblem as P;
    let id = match problem {
        P::Empty => "p-empty",
        P::MissingAt => "p-missing-at",
        P::ManyAt => "p-many-at",
        P::EmptyLocal => "p-empty-local",
        P::EmptyDomain => "p-empty-domain",
        P::LocalCharacter => "p-local-character",
        P::LocalTooLong => "p-local-too-long",
        P::BadDomain => "p-bad-domain",
        P::DomainWithoutDot => "p-domain-without-dot",
        P::NoCountryCode => "p-no-country-code",
        P::PhoneCharacter => "p-phone-character",
        P::AddressTooLong => "p-address-too-long",
        P::PhoneLength => "p-phone-length",
    };
    t(id)
}

#[cfg(test)]
mod tests {
    use super::{TEXTS, words};
    use fs_components_mf2::check_pairs;
    use fs_components_words::Locale;

    #[test]
    fn both_languages_have_the_same_ids_and_variables() {
        assert!(check_pairs(TEXTS).is_empty(), "{:?}", check_pairs(TEXTS));
    }

    #[test]
    fn the_german_labels_leave_no_english_default_behind() {
        let de = super::labels_with(&words(&Locale::parse("de").unwrap()));
        let en = super::labels_with(&words(&Locale::english()));
        let default = fs_components_workbenches::people::PeopleLabels::default();
        // English words are the shipped English file; German ones must differ from it everywhere
        // except where a word is the same in both languages (a placeholder, the name column).
        assert_eq!(en.create, default.create);
        let same = [
            (
                de.email_placeholder == en.email_placeholder,
                "email placeholder",
            ),
            (
                de.phone_placeholder == en.phone_placeholder,
                "phone placeholder",
            ),
        ];
        assert!(same.iter().all(|(s, _)| *s));
        for (g, e, what) in [
            (&de.search, &en.search, "search"),
            (&de.create, &en.create, "create"),
            (&de.edit, &en.edit, "edit"),
            (&de.save, &en.save, "save"),
            (&de.cancel, &en.cancel, "cancel"),
            (&de.delete, &en.delete, "delete"),
            (&de.delete_really, &en.delete_really, "delete really"),
            (&de.add_field, &en.add_field, "add field"),
            (
                &de.nothing_selected,
                &en.nothing_selected,
                "nothing selected",
            ),
            (&de.nobody, &en.nobody, "nobody"),
            (&de.nobody_found, &en.nobody_found, "nobody found"),
            (&de.delete_confirm, &en.delete_confirm, "delete confirm"),
            (&de.discard_confirm, &en.discard_confirm, "discard confirm"),
            (&de.saved, &en.saved, "saved"),
            (&de.not_saved, &en.not_saved, "not saved"),
            (&de.list_failed, &en.list_failed, "list failed"),
        ] {
            assert_ne!(g, e, "{what} is not translated");
        }
        assert!(
            de.add_field.contains("{}") && de.delete_confirm.contains("{}"),
            "the value stays a place"
        );
    }

    #[test]
    fn the_german_file_holds_no_english_sentence() {
        // Words that are English and not German; names and options (FreeSynergy, --export-sensitive, vCard) are not in the list.
        let english = [
            " the ", " and ", " not ", " with ", " your ", " you ", " is ", " are ", " this ",
            "Save", "Cancel", "Delete", "Edit", "New ", "Search", "Nobody", "Nothing", "Failed",
            "Imported", "Released", "Wrote", "Contact ",
        ];
        for line in TEXTS[1]
            .text
            .lines()
            .filter(|l| !l.starts_with('#') && l.contains('='))
        {
            let value = line.split_once('=').map_or("", |(_, v)| v);
            for word in english {
                assert!(!value.contains(word), "{line}: English word {word:?}");
            }
        }
    }

    #[test]
    fn german_and_english_say_different_things() {
        let en = words(&Locale::english()).plain("nav-all");
        let de = words(&Locale::parse("de").unwrap()).plain("nav-all");
        assert_eq!(en, "All contacts");
        assert_eq!(de, "Alle Kontakte");
    }
}
