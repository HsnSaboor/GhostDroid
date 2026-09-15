//! Logs page stub. Page agent owns this body.

use gpui_kit::{IntoElement, ParentElement as _, div};

use crate::state::AppState;

/// Daemon log stream body.
pub fn render_logs(_state: &AppState) -> impl IntoElement {
    tracing::debug!("render logs stub");
    div().child("Logs — page agent owns this")
}
