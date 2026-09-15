//! Spoof page body. Pure render, no mutation.

use gpui_kit::component::{
    Selectable as _, button::Button, description_list::DescriptionList, h_flex,
    progress::Progress, v_flex,
};
use gpui_kit::{IntoElement, ParentElement as _, Styled as _, div, px};

use crate::shared::{setting_row, status_tag};
use crate::state::AppState;

/// Spoof profile picker + props + toggles. Ports Slint page 2.
pub fn render_spoof(state: &AppState) -> impl IntoElement {
    tracing::debug!(id = %state.spoof.id, props = state.spoof.props.len(), "render spoof");
    // Select entity needs window+cx (SelectState::new), so pure-render
    // pass uses a Button row; upgrade to Select entity in view cx later.
    let picker = if state.spoof.ids.is_empty() {
        div().child(status_tag(state.spoof.id.as_str())).into_any_element()
    } else {
        h_flex()
            .gap(px(6.))
            .children(state.spoof.ids.iter().enumerate().map(|(ix, id)| {
                Button::new(format!("spoof-profile-{ix}"))
                    .label(id.clone())
                    .selected(ix as i32 == state.spoof.selected)
            }))
            .into_any_element()
    };
    let props = state
        .spoof
        .props
        .iter()
        .fold(DescriptionList::vertical(), |list, (k, v)| {
            list.item(k.clone(), v.clone(), 1)
        });
    // Switch::new needs id+checked+on_change with cx.listener; without
    // view cx, use a status Tag placeholder instead.
    let hide_root = setting_row("Hide root", status_tag("on"));
    let sensor = setting_row(
        "Sensor noise",
        h_flex()
            .gap(px(8.))
            .child(div().child("50"))
            .child(Progress::new("sensor-noise").value(50.)),
    );
    v_flex()
        .gap(px(8.))
        .child(picker)
        .child(props)
        .child(hide_root)
        .child(sensor)
}
