//! Devices page body: state dot + props + scan progress.

use gpui_kit::component::{
    Sizable as _,
    button::{Button, ButtonVariants as _},
    description_list::DescriptionList,
    h_flex,
    label::Label,
    progress::Progress,
    v_flex,
};
use gpui_kit::{AnyElement, Entity, IntoElement, ParentElement as _, Styled as _, px};

use crate::app::ShellView;
use crate::shared::device_dot;
use crate::state::AppState;

/// Device state + scan body. Scan flips `busy`; sync clears it.
pub fn render_devices(view: &Entity<ShellView>, state: &AppState) -> impl IntoElement {
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
    let session = if state.device.ready {
        "running"
    } else {
        "stopped"
    };
    let scan: AnyElement = if state.busy {
        Label::new("Scanning…").into_any_element()
    } else {
        Progress::new("device-scan").value(100.).into_any_element()
    };
    let ip = if state.ip.is_empty() {
        "—"
    } else {
        state.ip.as_str()
    };
    v_flex()
        .gap(px(8.))
        .child(
            h_flex()
                .gap(px(8.))
                .items_center()
                .child(device_dot(state.device.ready, state.device.frozen))
                .child(Label::new(status)),
        )
        .child(
            DescriptionList::vertical()
                .item("status", status, 1)
                .item("session", session, 1)
                .item("ip", ip, 1),
        )
        .child(scan)
        .child({
            let view = view.clone();
            Button::new("device-scan-btn")
                .ghost()
                .small()
                .label("Scan")
                .on_click(move |_, _, cx| {
                    view.update(cx, |this, cx| {
                        this.set_busy(true, cx);
                    });
                })
        })
}
