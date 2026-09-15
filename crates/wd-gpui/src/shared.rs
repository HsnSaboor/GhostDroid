//! Shared row/card compositions. Plain data in, elements out.
//!
//! Dark-only: all text goes through theme-aware kit components (`Label`,
//! `Tag`, `DescriptionList`, `Empty`) or inherits `Empty`'s foreground.
//! No raw `div().child(text)` — kit default text is unreadable on dark.

use gpui_kit::component::{
    Sizable as _, description_list::DescriptionList, h_flex, label::Label, tag::Tag, v_flex,
};
use gpui_kit::{IntoElement, ParentElement as _, Styled as _, div, px, rgb};

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

/// Device state dot: green ready, amber frozen, gray offline.
pub fn device_dot(ready: bool, frozen: bool) -> impl IntoElement {
    tracing::debug!(ready, frozen, "device dot");
    let color = if ready {
        if frozen {
            rgb(0x00fb_bf24)
        } else {
            rgb(0x0022_c55e)
        }
    } else {
        rgb(0x006b_7280)
    };
    div().w(px(12.)).h(px(12.)).rounded(px(6.)).bg(color)
}

/// Log tail list. Ports Slint `LogView`.
pub fn log_list(lines: &[String]) -> impl IntoElement {
    tracing::debug!(count = lines.len(), "log list");
    v_flex()
        .gap(px(2.))
        .children(lines.iter().map(|line| Label::new(line.clone())))
}
