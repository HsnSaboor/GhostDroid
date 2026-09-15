//! Page dispatch. Bodies live in per-page modules.

pub mod devices;
pub mod keys;
pub mod library;
pub mod logs;
pub mod spoof;

use gpui_kit::{AnyElement, Entity, IntoElement as _};

use crate::app::ShellView;
use crate::state::{AppState, Page};

/// Dispatch to the active page body. Takes the view entity so button
/// clicks mutate state (same `on_click` + `view.update` pattern as nav).
#[must_use]
pub fn render_page(view: &Entity<ShellView>, page: &Page, state: &AppState) -> AnyElement {
    tracing::debug!(page = ?page, "render page");
    match page {
        Page::Library => library::render_library(view, state).into_any_element(),
        Page::Devices => devices::render_devices(view, state).into_any_element(),
        Page::Spoof => spoof::render_spoof(view, state).into_any_element(),
        Page::Keys => keys::render_keys(view, state).into_any_element(),
        Page::Logs => logs::render_logs(view, state).into_any_element(),
    }
}
