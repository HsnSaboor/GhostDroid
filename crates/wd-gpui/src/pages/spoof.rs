//! Spoof Lab body: profile switcher + props + diff viewer + tunables.
//!
//! Picker ids (`s26-ultra`/`rog8`/`pixel8`) map to
//! `profiles/spoof/{s26-ultra,gaming-full,gaming-basic}.toml` via
//! `sync::spoof_props`. Diff viewer shows the delta vs the `s26-ultra`
//! baseline so ROG8/Pixel8 drift is visible before applying.

use gpui_kit::component::{
    Selectable as _, button::Button, description_list::DescriptionList, h_flex, label::Label,
    v_flex,
};
use gpui_kit::{AnyElement, Entity, IntoElement, ParentElement as _, Styled as _, px};

use crate::app::ShellView;
use crate::shared::{empty_state, section_title, setting_row, slider_row, status_tag};
use crate::state::AppState;
use crate::sync::{spoof_diff, spoof_props};

/// Spoof picker + props + diff. Clicks select via `ShellView::set_spoof`.
pub fn render_spoof(view: &Entity<ShellView>, state: &AppState) -> impl IntoElement {
    tracing::debug!(id = %state.spoof.id, props = state.spoof.props.len(), "render spoof");
    // Select entity needs window+cx (SelectState::new), so pure-render
    // pass uses a Button row; upgrade to Select entity in view cx later.
    let picker = if state.spoof.ids.is_empty() {
        status_tag(state.spoof.id.as_str()).into_any_element()
    } else {
        h_flex()
            .gap(px(8.))
            .children(state.spoof.ids.iter().enumerate().map(|(ix, id)| {
                let view = view.clone();
                let ix = i32::try_from(ix).unwrap_or(i32::MAX);
                Button::new(format!("spoof-profile-{ix}"))
                    .label(id.clone())
                    .selected(ix == state.spoof.selected)
                    .on_click(move |_, _, cx| {
                        view.update(cx, |this, cx| {
                            this.set_spoof(ix, cx);
                        });
                    })
            }))
            .into_any_element()
    };
    let live = spoof_props(&state.spoof.id);
    let rows = if live.is_empty() {
        state.spoof.props.clone()
    } else {
        live
    };
    let props: AnyElement = if rows.is_empty() {
        empty_state("No spoof props", "Load a profile to see device props.").into_any_element()
    } else {
        rows.iter()
            .fold(DescriptionList::vertical(), |list, (k, v)| {
                list.item(k.clone(), v.clone(), 1)
            })
            .into_any_element()
    };
    let diffs = spoof_diff(&state.spoof.id);
    let diff: AnyElement = if diffs.is_empty() {
        Label::new("Baseline profile — no diff vs s26-ultra.").into_any_element()
    } else {
        v_flex()
            .gap(px(2.))
            .children(diffs.iter().map(|line| Label::new(line.clone())))
            .into_any_element()
    };
    // Switch::new needs id+checked+on_change with cx.listener; without
    // view cx, use a status Tag placeholder instead.
    let hide_root = setting_row("Hide root", status_tag("on"));
    v_flex()
        .gap(px(8.))
        .child(section_title("Spoof Lab"))
        .child(picker)
        .child(props)
        .child(section_title("Diff vs s26-ultra"))
        .child(diff)
        .child(hide_root)
        .child(slider_row(
            "Sensor noise",
            "sensor-noise".to_owned(),
            50.,
            "50",
        ))
}
