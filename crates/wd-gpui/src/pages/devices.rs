//! Devices page stub. Page agent owns this body.

use gpui_kit::{IntoElement, ParentElement as _, div};

use crate::state::AppState;

/// Device state + scan body.
pub fn render_devices(_state: &AppState) -> impl IntoElement {
    tracing::debug!("render devices stub");
    div().child("Devices — page agent owns this")
}
