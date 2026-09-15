//! Keys page body. Pure render, no mutation.

use gpui_kit::component::{Selectable as _, button::Button, h_flex, label::Label, v_flex};
use gpui_kit::{Entity, IntoElement, ParentElement as _, Styled as _, px};

use crate::app::ShellView;
use crate::shared::{empty_state, prop_row};
use crate::state::AppState;

const TABS: [&str; 3] = ["Map", "Aim", "DPad"];

/// Keymap editor. Tab clicks select via `ShellView::set_keymap_tab`.
pub fn render_keys(view: &Entity<ShellView>, state: &AppState) -> impl IntoElement {
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
                    let view = view.clone();
                    let ix = i32::try_from(ix).unwrap_or(i32::MAX);
                    Button::new(format!("keys-tab-{ix}"))
                        .label(*label)
                        .selected(ix == state.keymap.tab_index)
                        .on_click(move |_, _, cx| {
                            view.update(cx, |this, cx| {
                                this.set_keymap_tab(ix, cx);
                            });
                        })
                })),
        )
        .child(canvas)
        .child(Label::new(format!(
            "{} nodes · {tab} tab",
            state.keymap.node_count
        )))
}
