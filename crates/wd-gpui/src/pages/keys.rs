//! Keys page body. Pure render, no mutation.

use gpui_kit::component::{Selectable as _, button::Button, h_flex, v_flex};
use gpui_kit::{IntoElement, ParentElement as _, Styled as _, div, px};

use crate::shared::prop_row;
use crate::state::AppState;

const TABS: [&str; 3] = ["Map", "Aim", "DPad"];

/// Keymap editor stubs. Ports Slint page 3.
pub fn render_keys(state: &AppState) -> impl IntoElement {
    tracing::debug!(profile = %state.keymap.profile, tab = state.keymap.tab_index, "render keys");
    // Absolute-positioned node canvas is the biggest gap — overlay owns
    // canvas per plan 07; kit 0.6 has no Empty, so div placeholder here.
    let canvas = div().child("canvas lives in overlay");
    v_flex()
        .gap(px(8.))
        .child(prop_row("Profile", &state.keymap.profile))
        .child(prop_row("Fire key", &state.keymap.fire_key))
        .child(
            h_flex()
                .gap(px(6.))
                .children(TABS.iter().enumerate().map(|(ix, label)| {
                    Button::new(format!("keys-tab-{ix}"))
                        .label(*label)
                        .selected(i32::try_from(ix).unwrap_or(i32::MAX) == state.keymap.tab_index)
                })),
        )
        .child(canvas)
        .child(div().child("0 nodes"))
}
