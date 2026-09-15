//! Library page body: search + filters + game grid + toolbar.
//!
//! Pure render over `&AppState` — no entity creation here. The search box is a
//! static div showing `query` text; a later pass upgrades it to a kit `Input`
//! backed by an `Entity<InputState>` (which needs `window`+`cx` at build time,
//! unavailable to these `&AppState` render fns). Filter tabs are a `Button`
//! row until the same pass wires clicks to `AppState::set_filter`.

use gpui_kit::component::{
    Sizable as _,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};
use gpui_kit::{AnyElement, IntoElement, ParentElement as _, Styled as _, div, px};
use wd_shell::GameRow;

use crate::shared::game_card;
use crate::state::AppState;

/// Segmented filter labels. Index mirrors `AppState::filter_index`.
const FILTERS: [&str; 3] = ["All", "Ready", "Running"];
/// Toolbar actions, mirroring the Slint emu-action row.
const ACTIONS: [(&str, &str); 4] = [
    ("lib-pause", "Pause"),
    ("lib-save", "Save"),
    ("lib-load", "Load"),
    ("lib-add", "Add"),
];

/// Rows matching query (title/pkg substring) + filter index (1=ready 2=running).
fn visible(state: &AppState) -> Vec<&GameRow> {
    let q = state.query.to_lowercase();
    state
        .games
        .iter()
        .filter(|g| {
            let hit = q.is_empty()
                || g.title.to_lowercase().contains(&q)
                || g.pkg.to_lowercase().contains(&q);
            let state_hit = match state.filter_index {
                1 => g.state.to_lowercase().contains("ready"),
                2 => g.state.to_lowercase().contains("running"),
                _ => true,
            };
            hit && state_hit
        })
        .collect()
}

/// Static search box. Entity-backed `Input` upgrade owns interactivity later.
fn search_box(state: &AppState) -> impl IntoElement {
    let text = if state.query.is_empty() {
        "Search games…".to_owned()
    } else {
        state.query.clone()
    };
    div().px(px(8.)).py(px(4.)).text_sm().child(text)
}

/// Filter tab row. Selected tab is primary, rest ghost.
fn filter_row(state: &AppState) -> impl IntoElement {
    h_flex()
        .gap(px(8.))
        .children(FILTERS.iter().enumerate().map(|(ix, label)| {
            let ix = i32::try_from(ix).unwrap_or(i32::MAX);
            let mut btn = Button::new(*label).small().label(*label);
            btn = if ix == state.filter_index {
                btn.primary()
            } else {
                btn.ghost()
            };
            btn
        }))
}

/// Bottom toolbar. Ghost buttons; click wiring lands with the entity pass.
fn toolbar() -> impl IntoElement {
    h_flex().gap(px(8.)).justify_end().children(
        ACTIONS
            .iter()
            .map(|(id, label)| Button::new(*id).ghost().small().label(*label)),
    )
}

/// Library grid body.
pub fn render_library(state: &AppState) -> impl IntoElement {
    tracing::debug!(
        query = %state.query,
        filter = state.filter_index,
        total = state.games.len(),
        "render library"
    );
    let rows = visible(state);
    let body: AnyElement = if rows.is_empty() {
        // Kit 0.6 has no `empty` module; plain centered text until upgrade.
        div()
            .w_full()
            .justify_center()
            .child("No ROMs yet — Scan to add games.")
            .into_any_element()
    } else {
        v_flex()
            .gap(px(8.))
            .children(rows.iter().map(|g| game_card(g)))
            .into_any_element()
    };
    v_flex()
        .gap(px(12.))
        .child(search_box(state))
        .child(filter_row(state))
        .child(body)
        .child(toolbar())
}
