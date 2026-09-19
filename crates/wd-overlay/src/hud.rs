//! Transparent HUD state: visibility, grab, touch dots, keybind rows.
//!
//! Backend note: prefer `wlr-layer-shell` overlay layer with alpha;
//! fallback is a transparent fullscreen window. This module owns state
//! only — no Wayland/GUI deps (logic-only crate rule).
#![deny(missing_docs)]

/// One active touch visualizer dot (rel 0-1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TouchDot {
    /// Touch slot.
    pub slot: u8,
    /// Rel x.
    pub x: f32,
    /// Rel y.
    pub y: f32,
}

/// One keybind display row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindRow {
    /// Node id.
    pub id: String,
    /// Key label.
    pub key: String,
}

/// Transparent HUD overlay state.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct HudState {
    /// Visible (F12 toggles).
    pub visible: bool,
    /// Pointer grabbed (Alt/Ctrl rising edge toggles).
    pub grabbed: bool,
    /// Active touch dots.
    pub touches: Vec<TouchDot>,
    /// Keybind rows.
    pub binds: Vec<BindRow>,
    /// Modifier level seen on previous poll (edge detection).
    held: bool,
}

impl HudState {
    /// New HUD, visible by default.
    #[must_use]
    pub fn new() -> Self {
        tracing::info!("wd-overlay: hud new");
        Self {
            visible: true,
            ..Self::default()
        }
    }

    /// F12 toggles visibility. Returns new state.
    pub fn toggle_visibility(&mut self) -> bool {
        self.visible = !self.visible;
        tracing::info!(visible = self.visible, "wd-overlay: hud visibility");
        self.visible
    }

    /// Alt/Ctrl rising edge toggles pointer grab. Returns new state.
    ///
    /// Holding the modifier across polls fires exactly once.
    pub fn on_modifier(&mut self, alt: bool, ctrl: bool) -> bool {
        let now = wd_input::grab_edge(alt, ctrl);
        if now && !self.held {
            self.grabbed = !self.grabbed;
            tracing::info!(grabbed = self.grabbed, "wd-overlay: hud grab");
        }
        self.held = now;
        self.grabbed
    }

    /// Edge variant with explicit previous state (stateless caller).
    pub fn on_modifier_edge(
        &mut self,
        alt: bool,
        ctrl: bool,
        prev_alt: bool,
        prev_ctrl: bool,
    ) -> bool {
        if wd_input::grab_rising_edge(alt, ctrl, prev_alt, prev_ctrl) {
            self.grabbed = !self.grabbed;
            tracing::info!(grabbed = self.grabbed, "wd-overlay: hud grab");
        }
        self.held = wd_input::grab_edge(alt, ctrl);
        self.grabbed
    }

    /// Force-release grab (pause / emergency).
    pub fn release_grab(&mut self) {
        self.grabbed = false;
        tracing::info!("wd-overlay: hud grab released");
    }

    /// Set touch visualizer dots (clamped to rel range).
    pub fn set_touches(&mut self, dots: Vec<TouchDot>) {
        self.touches = dots
            .into_iter()
            .map(|mut d| {
                d.x = d.x.clamp(0.0, 1.0);
                d.y = d.y.clamp(0.0, 1.0);
                d
            })
            .collect();
        tracing::debug!(n = self.touches.len(), "wd-overlay: hud touches");
    }

    /// Clear all touch dots.
    pub fn clear_touches(&mut self) {
        self.touches.clear();
        tracing::debug!("wd-overlay: hud touches cleared");
    }

    /// Rebuild keybind display rows from id→key pairs.
    pub fn set_binds(&mut self, rows: Vec<BindRow>) {
        self.binds = rows;
        tracing::debug!(n = self.binds.len(), "wd-overlay: hud binds");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f12_toggles() {
        let mut h = HudState::new();
        assert!(h.visible);
        assert!(!h.toggle_visibility());
        assert!(h.toggle_visibility());
    }

    #[test]
    fn modifier_grab_edge() {
        let mut h = HudState::new();
        assert!(h.on_modifier(true, false));
        // Hold: no re-toggle.
        assert!(h.on_modifier(true, false));
        assert!(h.grabbed);
        h.on_modifier(false, false);
        assert!(!h.on_modifier(false, true));
        assert!(!h.grabbed);
        h.on_modifier(true, false);
        h.release_grab();
        assert!(!h.grabbed);
    }

    #[test]
    fn touches_clamp_and_clear() {
        let mut h = HudState::new();
        h.set_touches(vec![TouchDot {
            slot: 0,
            x: 9.0,
            y: -1.0,
        }]);
        assert!((h.touches[0].x - 1.0).abs() < f32::EPSILON);
        assert!(h.touches[0].y.abs() < f32::EPSILON);
        h.clear_touches();
        assert!(h.touches.is_empty());
    }
}
