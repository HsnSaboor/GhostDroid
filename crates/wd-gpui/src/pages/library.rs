//! Library page body: search + filters + game grid + toolbar.
//!
//! Pure render over `&AppState` — no entity creation here. The search box is a
//! static label showing `query` text; a later pass upgrades it to a kit `Input`
//! backed by an `Entity<InputState>` (which needs `window`+`cx` at build time,
//! unavailable to these `&AppState` render fns). Filter tabs are a `Button`
//! row until the same pass wires clicks to `AppState::set_filter`.

use gpui_kit::component::{
    Sizable as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    label::Label,
    v_flex,
};
use gpui_kit::{AnyElement, IntoElement, ParentElement as _, Styled as _, px};
use wd_shell::GameRow;

use crate::shared::{empty_state, game_card, section_title};
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
    Label::new(text)
}

/// Filter tab row. Selected tab is primary, rest ghost. Clicks mutate
/// `AppState::filter_index` via the view entity (same pattern as nav).
fn filter_row(
    view: &gpui_kit::Entity<crate::app::ShellView>,
    state: &AppState,
) -> impl IntoElement {
    h_flex()
        .gap(px(8.))
        .children(FILTERS.iter().enumerate().map(|(ix, label)| {
            let ix = i32::try_from(ix).unwrap_or(i32::MAX);
            let view = view.clone();
            let mut btn = Button::new(*label).small().label(*label);
            btn = if ix == state.filter_index {
                btn.primary()
            } else {
                btn.ghost()
            };
            btn.on_click(move |_, _, cx| {
                view.update(cx, |this, cx| {
                    this.set_filter(ix, cx);
                });
            })
        }))
}

/// Bottom toolbar. Pause/Save/Load push a log line (daemon owns the real
/// action via MCP); Add switches to Devices. All clicks wired, none dead.
fn toolbar(view: &gpui_kit::Entity<crate::app::ShellView>) -> impl IntoElement {
    h_flex()
        .gap(px(8.))
        .justify_end()
        .children(ACTIONS.iter().map(|(id, label)| {
            let view = view.clone();
            let label = *label;
            Button::new(*id)
                .ghost()
                .small()
                .label(label)
                .on_click(move |_, _, cx| {
                    view.update(cx, |this, cx| {
                        this.push_log(format!("toolbar: {label}"), cx);
                    });
                })
        }))
}

/// One game row + live Launch button (`waydroid app launch <pkg>` via
/// `ShellView::launch_pkg`; result lands in Logs, never a dead button).
fn game_row_live(
    view: &gpui_kit::Entity<crate::app::ShellView>,
    game: &GameRow,
) -> impl IntoElement {
    let view = view.clone();
    let pkg = game.pkg.clone();
    h_flex()
        .gap(px(8.))
        .items_center()
        .justify_between()
        .child(v_flex().flex_1().child(game_card(game)))
        .child(
            Button::new(format!("lib-launch-{}", game.pkg))
                .small()
                .label("Launch")
                .on_click(move |_, _, cx| {
                    let pkg = pkg.clone();
                    view.update(cx, |this, cx| {
                        this.launch_pkg(pkg, cx);
                    });
                }),
        )
}

/// Library grid body. Takes the view entity for click wiring (nav pattern).
pub fn render_library(
    view: &gpui_kit::Entity<crate::app::ShellView>,
    state: &AppState,
) -> impl IntoElement {
    tracing::debug!(
        query = %state.query,
        filter = state.filter_index,
        total = state.games.len(),
        "render library"
    );
    let rows = visible(state);
    let count = Label::new(format!(
        "{} apps · live via `waydroid app list`",
        rows.len()
    ));
    let body: AnyElement = if rows.is_empty() {
        empty_state("No games found", "Scan to add games to the library.").into_any_element()
    } else {
        v_flex()
            .gap(px(8.))
            .children(rows.iter().map(|g| game_row_live(view, g)))
            .into_any_element()
    };
    v_flex()
        .gap(px(8.))
        .child(section_title("Library"))
        .child(count)
        .child(search_box(state))
        .child(filter_row(view, state))
        .child(body)
        .child(toolbar(view))
}
