//! Root window wiring: sidebar nav, page dispatch, overlay layers.

use gpui_kit::component::{
    Root,
    sidebar::{Sidebar, SidebarMenu, SidebarMenuItem},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Bounds, Context, FocusHandle, Focusable, InteractiveElement as _,
    IntoElement, ParentElement as _, Render, Size, Styled as _, Window, WindowBounds,
    WindowOptions, div, px,
};

use crate::pages::render_page;
use crate::state::{AppState, Page};

/// Shell root view: nav sidebar + active page body.
pub struct ShellView {
    state: AppState,
    focus: FocusHandle,
}

/// State mutations callable from page `on_click` handlers. Library's
/// filter/toolbar already use `set_filter`/`push_log` via this path.
impl ShellView {
    /// Switch page.
    pub fn set_page(&mut self, page: Page, cx: &mut Context<Self>) {
        tracing::debug!(page = ?page, "shell: set page");
        self.state.set_page(page);
        cx.notify();
    }

    /// Set the library filter index.
    pub fn set_filter(&mut self, index: i32, cx: &mut Context<Self>) {
        tracing::debug!(index, "shell: set filter");
        self.state.set_filter(index);
        cx.notify();
    }

    /// Push one daemon log line (toolbar actions report here).
    pub fn push_log(&mut self, line: String, cx: &mut Context<Self>) {
        tracing::debug!(line = %line, "shell: push log");
        self.state.push_log(line);
        cx.notify();
    }

    /// Pick a spoof profile by picker index.
    pub fn set_spoof(&mut self, index: i32, cx: &mut Context<Self>) {
        tracing::debug!(index, "shell: set spoof");
        self.state.set_spoof(index);
        cx.notify();
    }

    /// Pick a keymap tab by picker index.
    pub fn set_keymap_tab(&mut self, index: i32, cx: &mut Context<Self>) {
        tracing::debug!(index, "shell: set keymap tab");
        self.state.set_keymap_tab(index);
        cx.notify();
    }

    /// Mark scan in flight; Devices page shows the busy label.
    pub fn set_busy(&mut self, busy: bool, cx: &mut Context<Self>) {
        tracing::debug!(busy, "shell: set busy");
        self.state.set_busy(busy);
        cx.notify();
    }

    /// Clear the daemon log tail.
    pub fn clear_logs(&mut self, cx: &mut Context<Self>) {
        tracing::debug!("shell: clear logs");
        self.state.clear_logs();
        cx.notify();
    }
}

impl ShellView {
    fn new(cx: &mut Context<Self>) -> Self {
        tracing::debug!("shell view new");
        // Self-boot: blocking `waydroid` calls run on the background
        // executor (never the UI thread); results land via weak-entity
        // `update`. Missing binary keeps the seeded fallback.
        cx.spawn(async move |view, cx| {
            tracing::info!("sync: ensure booted");
            let boot = cx
                .background_spawn(async { crate::sync::ensure_booted() })
                .await;
            if let Err(err) = boot {
                tracing::warn!(%err, "sync: boot skipped, seeded state kept");
                return;
            }
            let games = cx
                .background_spawn(async { crate::sync::fetch_games() })
                .await
                .unwrap_or_default();
            let device = cx
                .background_spawn(async { crate::sync::fetch_device() })
                .await
                .ok();
            tracing::info!(games = games.len(), "sync: loaded");
            let _ = view.update(cx, |this, cx| {
                if !games.is_empty() {
                    this.state.set_games(games);
                }
                if let Some((dev, ip)) = device {
                    this.state.device = dev;
                    this.state.ip = ip;
                }
                cx.notify();
            });
        })
        .detach();
        Self {
            state: AppState::default(),
            focus: cx.focus_handle(),
        }
    }
}

impl Focusable for ShellView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for ShellView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        tracing::debug!(page = ?self.state.page, "render shell");
        let dialog_layer = Root::render_dialog_layer(window, cx);
        let sheet_layer = Root::render_sheet_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);
        let view = cx.entity();
        let active = self.state.page;

        div()
            .id("wd-root")
            .flex()
            .flex_row()
            .size_full()
            .child(
                Sidebar::new("wd-nav").child(SidebarMenu::new().children(Page::ALL.iter().map(
                    |page| {
                        let item = *page;
                        let view = view.clone();
                        SidebarMenuItem::new(item.title())
                            .active(item == active)
                            .on_click(move |_, _, cx| {
                                view.update(cx, |this, cx| {
                                    this.set_page(item, cx);
                                });
                            })
                    },
                ))),
            )
            .child(
                v_flex()
                    .flex_1()
                    .p(px(16.))
                    .child(render_page(&view, &active, &self.state)),
            )
            .children(sheet_layer)
            .children(dialog_layer)
            .children(notification_layer)
    }
}

/// Boot the GPUI app. Called by the `wd-gpui` binary.
///
/// # Panics
/// Panics if the OS refuses to open the application window.
pub fn run() {
    tracing::info!("wd-gpui run");
    let app = gpui_kit::application().with_assets(gpui_kit::assets::Assets);
    app.run(move |cx| {
        gpui_kit::init(cx);
        // Dark-only: GhostDroid never ships light mode. Forced before
        // window open so first paint is dark (no white flash).
        gpui_kit::component::Theme::change(gpui_kit::component::ThemeMode::Dark, None, cx);
        cx.spawn(async move |cx| {
            let window_size = Size {
                width: px(1280.),
                height: px(800.),
            };
            let bounds = cx.update(|cx| Bounds::centered(None, window_size, cx));
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            };
            cx.open_window(options, |window, cx| {
                let view = cx.new(ShellView::new);
                cx.new(|cx| Root::new(view, window, cx))
            })
            .expect("open wd-gpui window");
        })
        .detach();
    });
}
