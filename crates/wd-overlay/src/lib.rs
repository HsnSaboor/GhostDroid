//! wd-overlay: glow editor + OSD state machine. Logic-only (no `eframe`).
//!
//! Refs: `.plans/07-overlay-egui.md`; phantom hotkeys
//! `.devdocs/phantom/phantom/src/config.rs:131-138`; helper modes
//! `.devdocs/waydroid-helper/waydroid_helper/controller/app/mode_controller.py:17-60`.
//! Calls `wd-input::{load,validate,audit}` + `wd-shell` viewmodels. Draws nothing.
#![deny(missing_docs)]

pub mod canvas;
pub mod capture;
pub mod hotkeys;
pub mod hud;
pub mod layout;
pub mod modes;
pub mod osd;
pub mod profile;
pub mod widgets;

pub use canvas::EditorCanvas;
pub use capture::{MOUSE_BUTTONS, capturable, normalize_capture};
pub use hotkeys::{
    GrabEdge, HotkeyAction, action_for, action_for_full, action_for_full_edge, defaults, dup_free,
    is_visibility_key,
};
pub use hud::{BindRow, HudState, TouchDot};
pub use layout::{LAYOUT_VERSION, LayoutRow, OverlayLayout};
pub use modes::{EditorMode, switch_on_key};
pub use osd::{OsdState, TOAST_FRAMES};
pub use profile::{audit_profile, canvas_binds, canvas_keymap, load_profile, validate_profile};
pub use widgets::{MAPPING_PX, WidgetKind, WidgetRect};

/// Run headless: state machine only (glow renderer attaches in its own proc).
pub fn run() {
    tracing::info!("wd-overlay run headless");
}
