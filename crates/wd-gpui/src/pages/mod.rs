//! Page dispatch. Bodies live in per-page modules.

pub mod devices;
pub mod keys;
pub mod library;
pub mod logs;
pub mod spoof;

use gpui_kit::{AnyElement, IntoElement as _};

use crate::state::{AppState, Page};

/// Dispatch to the active page body.
#[must_use]
pub fn render_page(page: &Page, state: &AppState) -> AnyElement {
    tracing::debug!(page = ?page, "render page");
    match page {
        Page::Library => library::render_library(state).into_any_element(),
        Page::Devices => devices::render_devices(state).into_any_element(),
        Page::Spoof => spoof::render_spoof(state).into_any_element(),
        Page::Keys => keys::render_keys(state).into_any_element(),
        Page::Logs => logs::render_logs(state).into_any_element(),
    }
}
