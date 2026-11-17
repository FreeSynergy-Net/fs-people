//! The program as screens of the shell: the navigation on the left (all contacts, groups), the
//! contacts as the workbench (list and detail), the groups as a plain pointer to FreeSynergy Groups.
//!
//! Pattern: Adapter — the workbench and one page of blocks as `Surface`s of the application shell; the
//! shell draws the navigation because there is more than one screen.

use std::sync::Arc;

use fs_components::blinc::core::BlincContextState;
use fs_components_app::{AppError, AppWidget, Element, Menu, MenuEntry, Screen, Surface};
use fs_components_workbenches::people::{PeopleSource, PeopleWidget};

use crate::words::{labels, t, title};

/// The people workbench as a surface.
pub struct PeopleScreen(pub PeopleWidget);

impl Surface for PeopleScreen {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        Box::new(self.0.build(width, height))
    }

    fn styles(&self) -> String {
        [self.0.styles(), MORE_AIR.to_owned()].concat()
    }
}

/// The style sheet: the workbench's, with more air and larger type than its defaults (theme variables only).
const MORE_AIR: &str = r"
.fs-people-page { padding: var(--space-6); gap: var(--space-5); }
.fs-people-title-text { font-size: var(--text-3xl); }
.fs-people-section-title { font-size: var(--text-base); }
.fs-people-label { width: 140px; }
.fs-people-field { gap: var(--space-4); font-size: var(--text-lg); }
";

/// The shell around the workbench over `source`.
///
/// # Errors
/// [`AppError`] if the shell cannot be made.
pub fn app_widget(source: Arc<dyn PeopleSource>) -> Result<AppWidget, AppError> {
    let widget = PeopleWidget::new(source, labels());
    // One screen: the shell draws no navigation, so there are two areas, the list and the detail. Groups are not kept here
    // (FreeSynergy Groups keeps them): the menu says so instead of a page of its own.
    let screens = vec![Screen::new(
        "all",
        t("nav-all"),
        Arc::new(PeopleScreen(widget.clone())),
    )];
    let app = AppWidget::new(title(), screens)?;
    app.set_menus(menus(), handler(&widget))?;
    Ok(app)
}

/// The menu "Contacts": what the workbench can do now, and what waits for a later run (locked, with the reason).
fn menus() -> Vec<Menu> {
    let later = t("menu-later");
    vec![Menu::new(
        t("menu-contacts"),
        vec![
            MenuEntry::new("new", t("create")),
            MenuEntry::new("edit", t("edit")),
            MenuEntry::new("delete", t("delete")),
            MenuEntry::separator(),
            MenuEntry::new("groups", t("menu-groups")).disabled_because(t("groups-text")),
            MenuEntry::new("import", t("menu-import")).disabled_because(later.clone()),
            MenuEntry::new("export", t("menu-export")).disabled_because(later),
        ],
    )]
}

/// A chosen menu line acts on the workbench exactly as its button does (the state decides what is possible),
/// then the window is drawn again.
fn handler(widget: &PeopleWidget) -> impl Fn(&str) + Send + Sync + use<> {
    let widget = widget.clone();
    move |id| {
        widget.with_state(|state| match id {
            "new" => {
                let _ = state.begin_create();
            }
            "edit" => state.begin_edit(),
            "delete" => state.ask_delete(),
            _ => {}
        });
        if BlincContextState::is_initialized() {
            BlincContextState::get().request_rebuild();
        }
    }
}
