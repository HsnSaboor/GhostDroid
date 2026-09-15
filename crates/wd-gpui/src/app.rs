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

impl ShellView {
    fn new(cx: &mut Context<Self>) -> Self {
        tracing::debug!("shell view new");
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
            .child(Sidebar::new("wd-nav").child(
                SidebarMenu::new().children(Page::ALL.iter().map(|page| {
                    let item = *page;
                    let view = view.clone();
                    SidebarMenuItem::new(item.title())
                        .active(item == active)
                        .on_click(move |_, _, cx| {
                            view.update(cx, |this, cx| {
                                this.state.set_page(item);
                                cx.notify();
                            });
                        })
                })),
            ))
            .child(
                v_flex()
                    .flex_1()
                    .p(px(16.))
                    .child(render_page(&active, &self.state)),
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
