//! Editor mode state machine: EDIT / MAPPING.
//!
//! Port of waydroid-helper
//! `.devdocs/waydroid-helper/waydroid_helper/controller/app/mode_controller.py:17-60`
//! (`EDIT_MODE`/`MAPPING_MODE`, `toggle`, `is_mode_switch_key F1`),
//! `window.py:436 switch_mode`, F12 transparency `window.py:367-372`.
//! Logic-only: no GTK/egui here (plan 07: no `eframe` compile).
#![deny(missing_docs)]

/// Editor mode. `COPY ModeController.EDIT_MODE/MAPPING_MODE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorMode {
    /// Edit: drag/resize/capture widgets, never grabs input.
    #[default]
    Edit,
    /// Mapping: minimal indicators, grabs input for play.
    Mapping,
}

impl EditorMode {
    /// Toggle mode (`COPY ModeController.toggle`). Logs the switch.
    #[must_use]
    pub fn toggled(self) -> Self {
        let next = match self {
            Self::Edit => Self::Mapping,
            Self::Mapping => Self::Edit,
        };
        tracing::info!(from = ?self, to = ?next, "wd-overlay: mode switch");
        next
    }

    /// F1 switches mode (`COPY is_mode_switch_key F1`).
    #[must_use]
    pub fn is_switch_key(key: &str) -> bool {
        tracing::trace!(key, "wd-overlay: mode-switch key check");
        key.eq_ignore_ascii_case("f1")
    }

    /// F12 toggles widget transparency (`window.py:367`).
    #[must_use]
    pub fn is_transparency_key(key: &str) -> bool {
        tracing::trace!(key, "wd-overlay: transparency key check");
        key.eq_ignore_ascii_case("f12")
    }

    /// Grab guard: Edit never grabs, Mapping grabs (plan 07 step 5).
    #[must_use]
    pub fn wants_grab(self) -> bool {
        tracing::trace!(mode = ?self, "wd-overlay: grab guard check");
        matches!(self, Self::Mapping)
    }
}

/// Switch mode on a key press. F1 toggles, else current unchanged.
#[must_use]
pub fn switch_on_key(current: EditorMode, key: &str) -> EditorMode {
    if EditorMode::is_switch_key(key) {
        current.toggled()
    } else {
        tracing::trace!(key, "wd-overlay: key ignored by mode switch");
        current
    }
}
