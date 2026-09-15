//! Keys page body. Pure render, no mutation.

use gpui_kit::component::{Selectable as _, button::Button, h_flex, label::Label, v_flex};
use gpui_kit::{IntoElement, ParentElement as _, Styled as _, px};

use crate::shared::{empty_state, prop_row};
use crate::state::AppState;

const TABS: [&str; 3] = ["Map", "Aim", "DPad"];

/// Keymap editor. Ports Slint page 3.
pub fn render_keys(state: &AppState) -> impl IntoElement {
    tracing::debug!(profile = %state.keymap.profile, tab = state.keymap.tab_index, "render keys");
    let tab = usize::try_from(state.keymap.tab_index)
        .ok()
        .and_then(|ix| TABS.get(ix))
        .unwrap_or(&TABS[0]);
    // Absolute-positioned node canvas lives in the overlay (plan 07);
    // kit `Empty` owns the placeholder here with node/tab info.
    let canvas = empty_state("Node canvas", "Canvas lives in the overlay.");
    v_flex()
        .gap(px(8.))
        .child(prop_row("Profile", &state.keymap.profile))
        .child(prop_row("Fire key", &state.keymap.fire_key))
        .child(
            h_flex()
                .gap(px(8.))
                .children(TABS.iter().enumerate().map(|(ix, label)| {
                    Button::new(format!("keys-tab-{ix}"))
                        .label(*label)
                        .selected(i32::try_from(ix).unwrap_or(i32::MAX) == state.keymap.tab_index)
                })),
        )
        .child(canvas)
        .child(Label::new(format!("0 nodes · {tab} tab")))
}
