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
    /// `F12`: toggle HUD visibility.
    Visibility,
    /// Alt/Ctrl edge: toggle pointer grab.
    GrabToggle,
}

/// True for the F12 visibility key.
#[must_use]
pub fn is_visibility_key(key: &str) -> bool {
    tracing::trace!(key, "wd-overlay: visibility key check");
    key.eq_ignore_ascii_case("f12")
}

/// Full resolver with Alt state (Alt/Ctrl rising edge → grab toggle).
///
/// Backward-compatible level wrapper: treats the previous poll as released,
/// so a single call with Alt/Ctrl held reports the toggle. Polling callers
/// should use [`action_for_full_edge`] with the real previous state.
#[must_use]
pub fn action_for_full(key: &str, ctrl: bool, alt: bool) -> Option<HotkeyAction> {
    let edge = GrabEdge::new(
        wd_input::Modifiers::new(alt, ctrl),
        wd_input::Modifiers::default(),
    );
    action_for_full_edge(key, ctrl, edge)
}

/// Grab modifier pair (current + previous poll) for edge detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GrabEdge {
    /// Current Alt/Ctrl level.
    pub current: wd_input::Modifiers,
    /// Previous poll Alt/Ctrl level.
    pub previous: wd_input::Modifiers,
}

impl GrabEdge {
    /// New pair from current + previous levels.
    #[must_use]
    pub const fn from_levels(current: wd_input::Modifiers, previous: wd_input::Modifiers) -> Self {
        Self { current, previous }
    }

    /// New pair from raw levels.
    #[must_use]
    pub const fn new(current: wd_input::Modifiers, previous: wd_input::Modifiers) -> Self {
        Self { current, previous }
    }
}

/// Edge-triggered resolver: modifier-only events fire on the press transition.
#[must_use]
pub fn action_for_full_edge(key: &str, ctrl: bool, edge: GrabEdge) -> Option<HotkeyAction> {
    if wd_input::grab_rising_edge(edge.current, edge.previous) && key.trim().is_empty() {
        return Some(HotkeyAction::GrabToggle);
    }
    action_for(key, ctrl)
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
            "F12" => Some(HotkeyAction::Visibility),
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
