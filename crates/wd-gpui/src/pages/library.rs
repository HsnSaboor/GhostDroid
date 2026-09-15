//! Library page stub. Page agent owns this body.

use gpui_kit::{IntoElement, ParentElement as _, div};

use crate::state::AppState;

/// Library grid body.
pub fn render_library(_state: &AppState) -> impl IntoElement {
    tracing::debug!("render library stub");
    div().child("Library — page agent owns this")
}
