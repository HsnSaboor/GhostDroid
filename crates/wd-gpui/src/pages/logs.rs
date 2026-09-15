//! Logs page body: scrollable daemon stream + clear + line count.

use gpui_kit::component::{
    Sizable as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    label::Label,
    scroll::ScrollableElement as _,
    v_flex,
};
use gpui_kit::{
    AnyElement, InteractiveElement as _, IntoElement, ParentElement as _, Styled as _, div, px,
};

use crate::shared::{empty_state, log_list};
use crate::state::AppState;

/// Daemon log stream body. Mirrors the Slint Logs section.
pub fn render_logs(state: &AppState) -> impl IntoElement {
    tracing::debug!(count = state.logs.lines.len(), "render logs");
    let stream: AnyElement = if state.logs.lines.is_empty() {
        empty_state("No log lines", "Daemon output will appear here.").into_any_element()
    } else {
        div()
            .flex_1()
            .id("logs-scroll")
            .overflow_y_scrollbar()
            .child(log_list(&state.logs.lines))
            .into_any_element()
    };
    v_flex().gap(px(8.)).child(stream).child(
        h_flex()
            .gap(px(8.))
            .items_center()
            .child(Button::new("logs-clear").ghost().small().label("Clear"))
            .child(Label::new(format!("{} lines", state.logs.lines.len()))),
    )
}
