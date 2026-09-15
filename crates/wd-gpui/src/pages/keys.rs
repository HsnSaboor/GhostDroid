//! Keys page stub. Page agent owns this body.

use gpui_kit::{IntoElement, ParentElement as _, div};

use crate::state::AppState;

/// Keymap editor body.
pub fn render_keys(_state: &AppState) -> impl IntoElement {
    tracing::debug!("render keys stub");
    div().child("Keys — page agent owns this")
}
