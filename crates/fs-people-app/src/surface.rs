//! The program as screens of the shell: the navigation on the left (all contacts, groups), the
//! contacts as the workbench (list and detail), the groups as a plain pointer to FreeSynergy Groups.
//!
//! Pattern: Adapter — the workbench and one page of blocks as `Surface`s of the application shell; the
//! shell draws the navigation because there is more than one screen.

use std::sync::Arc;

use fs_components::blinc::layout::div::div;
use fs_components_app::{AppError, AppWidget, Element, Screen, Surface};
use fs_components_widgets::states::EmptyState;
use fs_components_widgets::{Glyph, styles};
use fs_components_workbenches::people::{PeopleSource, PeopleWidget};

use crate::words::{labels, t, title};

/// The people workbench as a surface.
pub struct PeopleScreen(pub PeopleWidget);

impl Surface for PeopleScreen {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        Box::new(self.0.build(width, height))
    }

    fn styles(&self) -> String {
        self.0.styles()
    }
}

/// "Groups": People keeps no groups, FreeSynergy Groups does; the page says so with a symbol.
struct GroupsPage;

impl Surface for GroupsPage {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        let empty = EmptyState::new(&t("groups-title"))
            .id("groups")
            .glyph(Glyph::Person)
            .text(&t("groups-text"));
        Box::new(div().w(width).h(height).child(empty.build()))
    }

    fn styles(&self) -> String {
        styles()
    }
}

/// The symbol in front of a name in the navigation (characters every font has; the screen list takes text only).
const ALL_SYMBOL: &str = "\u{263A}";
const GROUPS_SYMBOL: &str = "\u{263B}";

/// The shell around the workbench over `source`.
///
/// # Errors
/// [`AppError`] if the shell cannot be made.
pub fn app_widget(source: Arc<dyn PeopleSource>) -> Result<AppWidget, AppError> {
    let widget = PeopleWidget::new(source, labels());
    let screens = vec![
        Screen::new(
            "all",
            format!("{ALL_SYMBOL}  {}", t("nav-all")),
            Arc::new(PeopleScreen(widget)),
        ),
        Screen::new(
            "groups",
            format!("{GROUPS_SYMBOL}  {}", t("nav-groups")),
            Arc::new(GroupsPage),
        ),
    ];
    AppWidget::new(title(), screens)
}
