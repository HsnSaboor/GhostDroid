//! Shared row/card compositions. Plain data in, elements out.
//!
//! Dark-only: all text goes through theme-aware kit components (`Label`,
//! `Tag`, `DescriptionList`, `Empty`) or inherits `Empty`'s foreground.
//! No raw `div().child(text)` — kit default text is unreadable on dark.

use gpui_kit::component::{
    Sizable as _, description_list::DescriptionList, h_flex, label::Label, progress::Progress,
    tag::Tag, v_flex,
};
use gpui_kit::{IntoElement, ParentElement as _, Styled as _, px};

/// Section heading. All page titles go through kit `Label` (dark tokens,
/// WCAG AAA foreground) — never raw div text.
pub fn section_title(text: &str) -> impl IntoElement {
    tracing::debug!(text, "section title");
    Label::new(text.to_owned())
}

/// Label + value + bar row.
///
/// Kit 0.6 has no `Slider`, so live tunables (sensor noise, aim
/// sensitivity, reach) render as `Progress` bars with a theme-aware value
/// `Label` — same dark tokens, no custom colors.
pub fn slider_row(label: &str, id: String, percent: f32, value: &str) -> impl IntoElement {
    tracing::debug!(label, percent, "slider row");
    v_flex()
        .gap(px(4.))
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .gap(px(8.))
                .child(Label::new(label.to_owned()))
                .child(Label::new(value.to_owned())),
        )
        .child(Progress::new(id).value(percent))
}

/// Game card: title + package + genre/state tags. Ports Slint `GameCard`.
pub fn game_card(game: &wd_shell::GameRow) -> impl IntoElement {
    tracing::debug!(title = %game.title, "game card");
    v_flex()
        .gap(px(4.))
        .child(Label::new(game.title.clone()))
        .child(Label::new(game.pkg.clone()))
        .child(
            h_flex()
                .gap(px(6.))
                .child(status_tag(&game.genre))
                .child(status_tag(&game.state)),
        )
}

/// Small status chip. Ports Slint `StatusChip`.
#[must_use]
pub fn status_tag(text: &str) -> impl IntoElement {
    Tag::new().small().child(text.to_owned())
}

/// Key/value row. Ports Slint `PropRow`.
pub fn prop_row(key: &str, value: &str) -> impl IntoElement {
    tracing::debug!(key, "prop row");
    DescriptionList::vertical().item(key, value, 1)
}

/// Label + control row. Ports Slint `SettingRow`.
pub fn setting_row(label: &str, content: impl IntoElement) -> impl IntoElement {
    tracing::debug!(label, "setting row");
    h_flex()
        .items_center()
        .justify_between()
        .gap(px(8.))
        .child(Label::new(label.to_owned()))
        .child(content)
}

/// Centered Empty-style empty state built from theme-aware `Label`s
/// (kit 0.6 has no `empty` module). No raw div text, no contrast gap.
pub fn empty_state(title: &str, description: &str) -> impl IntoElement {
    tracing::debug!(title, "empty state");
    v_flex()
        .w_full()
        .items_center()
        .justify_center()
        .gap(px(4.))
        .py(px(24.))
        .child(Label::new(title.to_owned()))
        .child(Label::new(description.to_owned()))
}

/// Device state chip: named theme tokens only — `success` ready,
/// `warning` frozen, `secondary` offline. Dark-aware via kit `Tag`
/// variants, never raw `rgb()` hex.
pub fn device_dot(ready: bool, frozen: bool) -> impl IntoElement {
    tracing::debug!(ready, frozen, "device dot");
    if frozen {
        Tag::warning().small().child("frozen")
    } else if ready {
        Tag::success().small().child("ready")
    } else {
        Tag::secondary().small().child("offline")
    }
}

/// Log tail list. Ports Slint `LogView`.
pub fn log_list(lines: &[String]) -> impl IntoElement {
    tracing::debug!(count = lines.len(), "log list");
    v_flex()
        .gap(px(2.))
        .children(lines.iter().map(|line| Label::new(line.clone())))
}
