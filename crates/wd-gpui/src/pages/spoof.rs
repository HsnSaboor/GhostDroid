//! Spoof page body. Pure render, no mutation.

use gpui_kit::component::{
    Selectable as _, button::Button, description_list::DescriptionList, h_flex, label::Label,
    progress::Progress, v_flex,
};
use gpui_kit::{AnyElement, Entity, IntoElement, ParentElement as _, Styled as _, px};

use crate::app::ShellView;
use crate::shared::{empty_state, setting_row, status_tag};
use crate::state::AppState;

/// Spoof profile picker + props + toggles. Ports Slint page 2. Picker
/// clicks select the profile id via `ShellView::set_spoof`.

/// Spoof profile picker + props + toggles. Ports Slint page 2.
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
    let props: AnyElement = if state.spoof.props.is_empty() {
        empty_state("No spoof props", "Load a profile to see device props.").into_any_element()
    } else {
        state
            .spoof
            .props
            .iter()
            .fold(DescriptionList::vertical(), |list, (k, v)| {
                list.item(k.clone(), v.clone(), 1)
            })
            .into_any_element()
    };
    // Switch::new needs id+checked+on_change with cx.listener; without
    // view cx, use a status Tag placeholder instead.
    let hide_root = setting_row("Hide root", status_tag("on"));
    let sensor = setting_row(
        "Sensor noise",
        h_flex()
            .gap(px(8.))
            .child(Label::new("50"))
            .child(Progress::new("sensor-noise").value(50.)),
    );
    v_flex()
        .gap(px(8.))
        .child(picker)
        .child(props)
        .child(hide_root)
        .child(sensor)
}
