//! Keymapper Studio body: live node list + tunables.
//!
//! Node rows mirror `profiles/keymap/pubg.json` (21 nodes); the header count
//! comes from `sync::keymap_node_count` via `AppState::keymap.node_count`
//! (always the live pubg total). Tabs filter rows Map/Aim/DPad. Tunables render
//! as `shared::slider_row` bars (kit 0.6 has no `Slider`).

use gpui_kit::component::{Selectable as _, button::Button, h_flex, label::Label, v_flex};
use gpui_kit::{AnyElement, Entity, IntoElement, ParentElement as _, Styled as _, px};

use crate::app::ShellView;
use crate::shared::{prop_row, section_title, slider_row};
use crate::state::AppState;

const TABS: [&str; 3] = ["Map", "Aim", "DPad"];

/// `(tab_index, node id, key)` — canonical mirror of `pubg.json` (21 rows).
const NODES: [(i32, &str, &str); 21] = [
    (0, "move", "WASD"),
    (0, "sprint_lock", "LeftShift"),
    (0, "free_look", "LeftAlt"),
    (0, "interact", "F"),
    (0, "jump", "Space"),
    (0, "crouch", "C"),
    (0, "prone", "Z"),
    (0, "map", "M"),
    (0, "reload", "R"),
    (1, "camera", "Tab"),
    (1, "fire", "MouseLeft"),
    (1, "secondary_fire", "MouseLeft"),
    (1, "ads_toggle", "MouseRight"),
    (1, "grenade", "G"),
    (1, "throwable_select", "G"),
    (1, "rapid_left_ctrl", "LeftCtrl"),
    (1, "rapid_interact", "E"),
    (2, "weapon_slot_0", "0"),
    (2, "weapon_slot_1", "1"),
    (2, "weapon_slot_2", "2"),
    (2, "weapon_slot_3", "3"),
];

/// Rows for the active tab as `id · key` labels.
fn node_rows(tab: i32) -> impl IntoElement {
    v_flex().gap(px(2.)).children(
        NODES
            .iter()
            .filter(|(t, _, _)| *t == tab)
            .map(|(_, id, key)| Label::new(format!("{id} · {key}"))),
    )
}

/// Keymap editor. Tab clicks select via `ShellView::set_keymap_tab`.
pub fn render_keys(view: &Entity<ShellView>, state: &AppState) -> impl IntoElement {
    tracing::debug!(profile = %state.keymap.profile, tab = state.keymap.tab_index, "render keys");
    let tab = usize::try_from(state.keymap.tab_index)
        .ok()
        .and_then(|ix| TABS.get(ix))
        .unwrap_or(&TABS[0]);
    let nodes: AnyElement = node_rows(state.keymap.tab_index).into_any_element();
    v_flex()
        .gap(px(8.))
        .child(section_title("Keymapper Studio"))
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
        .child(nodes)
        .child(slider_row(
            "Global sensitivity",
            "keys-global-sens".to_owned(),
            50.,
            "1.00",
        ))
        .child(slider_row(
            "Aim sensitivity",
            "keys-aim-sens".to_owned(),
            58.,
            "1.15",
        ))
        .child(slider_row(
            "Aim reach",
            "keys-aim-reach".to_owned(),
            18.,
            "0.18",
        ))
        .child(Label::new(format!(
            "{} nodes · {tab} tab",
            state.keymap.node_count
        )))
}
