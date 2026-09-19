//! Logs page body: live daemon stream + refresh + clear + line count.
//!
//! Source is the live socket (`/tmp/ghostdroid.sock`, fallback
//! `/tmp/wd-daemon.sock`) via `ShellView::refresh_logs`; boot also pulls one
//! tail. Refresh never blanks the page — socket failure keeps the tail.

use gpui_kit::component::{
    Sizable as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    label::Label,
    scroll::ScrollableElement as _,
    v_flex,
};
use gpui_kit::{
    AnyElement, Entity, InteractiveElement as _, IntoElement, ParentElement as _, Styled as _, div,
    px,
};

use crate::app::ShellView;
use crate::shared::{empty_state, log_list, section_title};
use crate::state::AppState;

/// Daemon log stream. Clear wipes the tail via `ShellView::clear_logs`.
pub fn render_logs(view: &Entity<ShellView>, state: &AppState) -> impl IntoElement {
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
    v_flex()
        .gap(px(8.))
        .child(section_title("Logs"))
        .child(Label::new("Live via /tmp/ghostdroid.sock"))
        .child(stream)
        .child(
            h_flex()
                .gap(px(8.))
                .items_center()
                .child({
                    let view = view.clone();
                    Button::new("logs-refresh")
                        .small()
                        .label("Refresh")
                        .on_click(move |_, _, cx| {
                            view.update(cx, |this, cx| {
                                this.refresh_logs(cx);
                            });
                        })
                })
                .child({
                    let view = view.clone();
                    Button::new("logs-clear")
                        .ghost()
                        .small()
                        .label("Clear")
                        .on_click(move |_, _, cx| {
                            view.update(cx, |this, cx| {
                                this.clear_logs(cx);
                            });
                        })
                })
                .child(Label::new(format!("{} lines", state.logs.lines.len()))),
        )
}
