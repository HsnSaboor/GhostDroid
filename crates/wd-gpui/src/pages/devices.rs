//! Devices page body: live container state + power actions + scan.
//!
//! Status text mirrors `waydroid status` verbatim (RUNNING/FROZEN/STOPPED)
//! so the page never drifts from the daemon. Power buttons call
//! `ShellView::device_request`; Scan re-polls `fetch_device`.

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
use crate::shared::{device_dot, section_title};
use crate::state::AppState;
use crate::sync::DeviceAction;

/// Live container/session words from `waydroid status`.
fn status_words(state: &AppState) -> (&'static str, &'static str) {
    if state.device.frozen {
        ("FROZEN", "RUNNING")
    } else if state.device.ready {
        ("RUNNING", "RUNNING")
    } else {
        ("STOPPED", "STOPPED")
    }
}

/// Power button row: Start / Stop / Restart, all wired (none dead).
fn power_row(view: &Entity<ShellView>) -> impl IntoElement {
    h_flex().gap(px(8.)).children(
        [
            DeviceAction::Start,
            DeviceAction::Stop,
            DeviceAction::Restart,
        ]
        .map(|action| {
            let view = view.clone();
            Button::new(format!("device-{}", action.label().to_lowercase()))
                .small()
                .label(action.label())
                .on_click(move |_, _, cx| {
                    view.update(cx, |this, cx| {
                        this.device_request(action, cx);
                    });
                })
        }),
    )
}

/// Device state + scan body. Scan flips `busy`; sync clears it.
pub fn render_devices(view: &Entity<ShellView>, state: &AppState) -> impl IntoElement {
    tracing::debug!(
        ready = state.device.ready,
        frozen = state.device.frozen,
        busy = state.busy,
        "render devices"
    );
    let (container, session) = status_words(state);
    let scan: AnyElement = if state.busy {
        Label::new("Scanning…").into_any_element()
    } else {
        Progress::new("device-scan").value(100.).into_any_element()
    };
    v_flex()
        .gap(px(8.))
        .child(section_title("Devices"))
        .child(
            h_flex()
                .gap(px(8.))
                .items_center()
                .child(device_dot(state.device.ready, state.device.frozen)),
        )
        .child(
            DescriptionList::vertical()
                .item("container", container, 1)
                .item("session", session, 1)
                .item("ip", state.ip.clone(), 1),
        )
        .child(scan)
        .child(power_row(view))
        .child({
            let view = view.clone();
            Button::new("device-scan-btn")
                .ghost()
                .small()
                .label("Scan")
                .on_click(move |_, _, cx| {
                    view.update(cx, |this, cx| {
                        this.refresh_device(cx);
                    });
                })
        })
}
