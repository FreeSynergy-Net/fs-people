//! Tests of the words: both languages agree, German has no English left, the labels are translated.

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
        " the ", " and ", " not ", " with ", " your ", " you ", " is ", " are ", " this ", "Save",
        "Cancel", "Delete", "Edit", "New ", "Search", "Nobody", "Nothing", "Failed", "Imported",
        "Released", "Wrote", "Contact ",
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
