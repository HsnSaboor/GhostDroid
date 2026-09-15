//! Devices page body: state dot + props + scan progress.

use gpui_kit::component::{
    Sizable as _,
    button::{Button, ButtonVariants as _},
    description_list::DescriptionList,
    h_flex, progress::Progress, v_flex,
};
use gpui_kit::{AnyElement, IntoElement, ParentElement as _, Styled as _, div, px};

use crate::shared::device_dot;
use crate::state::AppState;

/// Device state + scan body. Mirrors the Slint Devices section.
pub fn render_devices(state: &AppState) -> impl IntoElement {
    tracing::debug!(
        ready = state.device.ready,
        frozen = state.device.frozen,
        busy = state.busy,
        "render devices"
    );
    let status = if state.device.frozen {
        "frozen"
    } else if state.device.ready {
        "ready"
    } else {
        "offline"
    };
    let scan: AnyElement = if state.busy {
        div().child("Scanning…").into_any_element()
    } else {
        Progress::new("device-scan").value(100.).into_any_element()
    };
    v_flex()
        .gap(px(8.))
        .child(
            h_flex()
                .gap(px(8.))
                .items_center()
                .child(device_dot(state.device.ready, state.device.frozen))
                .child(div().child(status)),
        )
        .child(
            DescriptionList::vertical()
                .item("serial", "emulator-5554", 1)
                .item("state", "device", 1),
        )
        .child(scan)
        .child(Button::new("device-scan-btn").ghost().small().label("Scan"))
}
