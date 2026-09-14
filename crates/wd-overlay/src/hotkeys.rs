//! Runtime hotkeys: phantom parity + `cage` + `XtMapper`.
//!
//! Defaults `COPY .devdocs/phantom/phantom/src/config.rs:131-138`
//! (`F1` mouse-route, `F8` capture, `F9` pause, `F10` preview, `F2` shutdown,
//! dup fallback `config.rs:238`); `cage` `F10` passthrough toggle
//! (`cage-xtmapper/README`); `XtMapper` `Ctrl+P`/`E`/`S` (`KeymapConfig.java`).
#![deny(missing_docs)]

/// Runtime hotkey action (headless Mapping mode runs without UI).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAction {
    /// `F1`: route mouse game/cursor (`config.rs:134 mouse_toggle`).
    RouteMouse,
    /// `F8`: capture toggle (`config.rs:135`).
    Capture,
    /// `F9`: pause + emergency input release (plan 07 guard).
    Pause,
    /// `F10`: overlay preview / `cage` passthrough toggle.
    Preview,
    /// `F2`: shutdown (`config.rs:138`).
    Shutdown,
    /// `Ctrl+P`: pause (`XtMapper` parity).
    PauseXt,
    /// `Ctrl+E`: open editor (`XtMapper` parity).
    EditorXt,
    /// `Ctrl+S`: save profile (`XtMapper` parity).
    SaveXt,
}

/// Resolve a key press (+ctrl) to an action. `None` = not a hotkey.
#[must_use]
pub fn action_for(key: &str, ctrl: bool) -> Option<HotkeyAction> {
    let upper = key.to_ascii_uppercase();
    let action = if ctrl {
        match upper.as_str() {
            "P" => Some(HotkeyAction::PauseXt),
            "E" => Some(HotkeyAction::EditorXt),
            "S" => Some(HotkeyAction::SaveXt),
            _ => None,
        }
    } else {
        match upper.as_str() {
            "F1" => Some(HotkeyAction::RouteMouse),
            "F8" => Some(HotkeyAction::Capture),
            "F9" => Some(HotkeyAction::Pause),
            "F10" => Some(HotkeyAction::Preview),
            "F2" => Some(HotkeyAction::Shutdown),
            _ => None,
        }
    };
    tracing::info!(key, ctrl, action = ?action, "wd-overlay: hotkey");
    action
}

/// Default phantom hotkey names (`config.rs:206-238` fallback set).
#[must_use]
pub fn defaults() -> [&'static str; 5] {
    tracing::debug!("wd-overlay: hotkey defaults");
    ["F1", "F8", "F9", "F10", "F2"]
}

/// True when no key repeats (else phantom falls back to defaults).
#[must_use]
pub fn dup_free(keys: &[&str]) -> bool {
    let mut seen = std::collections::HashSet::new();
    let mut ok = true;
    for key in keys {
        if !seen.insert(key.to_ascii_uppercase()) {
            tracing::warn!(key, "wd-overlay: duplicate hotkey");
            ok = false;
        }
    }
    tracing::debug!(ok, "wd-overlay: hotkey dup check");
    ok
}
