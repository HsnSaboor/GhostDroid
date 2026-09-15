//! Spoof page stub. Page agent owns this body.

use gpui_kit::{IntoElement, ParentElement as _, div};

use crate::state::AppState;

/// Spoof profile picker body.
pub fn render_spoof(_state: &AppState) -> impl IntoElement {
    tracing::debug!("render spoof stub");
    div().child("Spoof — page agent owns this")
}
