//! Edit canvas: drag/resize/capture over widget rects + guards.
//!
//! Flow `COPY KEY_MAPPING.md`: Edit default → drag place → handles resize →
//! double-click capture → F1 Mapping. Calls `wd-input::{validate,audit}`
//! + `wd-shell` viewmodels; draws nothing (plan 07 DRY).
#![deny(missing_docs)]

use crate::capture::normalize_capture;
use crate::hotkeys::{HotkeyAction, action_for, is_visibility_key};
use crate::hud::HudState;
use crate::modes::{EditorMode, switch_on_key};
use crate::osd::OsdState;
use crate::widgets::{WidgetKind, WidgetRect};

/// Edit canvas: owns mode + rects + OSD. Single editor source.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EditorCanvas {
    /// Current mode.
    pub mode: EditorMode,
    /// Widget rects.
    pub widgets: Vec<WidgetRect>,
    /// Selected widget id.
    pub selected: Option<String>,
    /// Screen px (rel 0-1 → px base).
    pub screen: (u32, u32),
    /// F12 transparency flag.
    pub transparent: bool,
    /// F12 HUD visibility (starts shown).
    pub visible: bool,
    /// Transparent HUD state (dots + binds + grab).
    pub hud: HudState,
    /// OSD state.
    pub osd: OsdState,
}

impl EditorCanvas {
    /// New canvas at `screen`. Edit default, no grab.
    #[must_use]
    pub fn new(screen: (u32, u32)) -> Self {
        tracing::info!(w = screen.0, h = screen.1, "wd-overlay: canvas new");
        Self {
            screen,
            visible: true,
            hud: HudState::new(),
            ..Self::default()
        }
    }

    /// Add a widget at rel pos. Logs add.
    pub fn add(&mut self, id: &str, kind: WidgetKind, x: f32, y: f32) {
        let rect = WidgetRect::placed(id, kind, x, y, self.screen);
        tracing::info!(id, kind = ?kind, "wd-overlay: widget add");
        self.selected = Some(id.to_string());
        self.widgets.push(rect);
    }

    /// Select by id. `None` clears.
    pub fn select(&mut self, id: Option<&str>) {
        self.selected = id.map(str::to_string);
        tracing::debug!(selected = ?self.selected, "wd-overlay: select");
    }

    /// Selected rect mutably.
    fn selected_mut(&mut self) -> Option<&mut WidgetRect> {
        let id = self.selected.as_ref()?;
        self.widgets.iter_mut().find(|w| &w.id == id)
    }

    /// Drag selected by rel delta (Edit only).
    pub fn drag_selected(&mut self, dx: f32, dy: f32) {
        if self.mode != EditorMode::Edit {
            tracing::warn!("wd-overlay: drag blocked outside Edit");
            return;
        }
        if let Some(w) = self.selected_mut() {
            w.moved(dx, dy);
        }
    }

    /// Resize selected by rel delta (Edit only).
    pub fn resize_selected(&mut self, dw: f32, dh: f32) {
        if self.mode != EditorMode::Edit {
            tracing::warn!("wd-overlay: resize blocked outside Edit");
            return;
        }
        let screen = self.screen;
        if let Some(w) = self.selected_mut() {
            w.resized(dw, dh, screen);
        }
    }

    /// Double-click capture: bind key string to selected (Edit only).
    #[must_use]
    pub fn capture_selected(&mut self, raw: &str) -> bool {
        if self.mode != EditorMode::Edit {
            tracing::warn!("wd-overlay: capture blocked outside Edit");
            return false;
        }
        let Some(norm) = normalize_capture(raw) else {
            tracing::warn!(raw, "wd-overlay: capture rejected");
            return false;
        };
        if let Some(w) = self.selected_mut() {
            tracing::info!(id = %w.id, key = %norm, "wd-overlay: widget bind");
            w.key = norm;
            true
        } else {
            false
        }
    }

    /// Handle a key press: F1 mode switch, F12 transparency, F9 pause, F2/F8/F10.
    #[must_use]
    pub fn on_key(&mut self, key: &str, ctrl: bool) -> Option<HotkeyAction> {
        let next = switch_on_key(self.mode, key);
        if next != self.mode {
            self.mode = next;
            self.osd.set_minimal(next == EditorMode::Mapping);
        }
        if EditorMode::is_transparency_key(key) && !ctrl {
            self.transparent = !self.transparent;
            tracing::info!(transparent = self.transparent, "wd-overlay: transparency");
        }
        if is_visibility_key(key) && !ctrl {
            self.visible = !self.visible;
            self.hud.visible = self.visible;
            tracing::info!(visible = self.visible, "wd-overlay: visibility");
        }
        let action = action_for(key, ctrl);
        if action == Some(HotkeyAction::Pause) || action == Some(HotkeyAction::PauseXt) {
            self.osd.set_paused(!self.osd.paused);
        }
        action
    }

    /// Alt/Ctrl rising edge toggles pointer grab (delegates to HUD).
    pub fn on_modifier(&mut self, alt: bool, ctrl: bool) -> bool {
        let grabbed = self.hud.on_modifier(alt, ctrl);
        tracing::debug!(grabbed, "wd-overlay: canvas grab");
        grabbed
    }

    /// Edge variant with explicit previous state.
    pub fn on_modifier_edge(&mut self, edge: crate::GrabEdge) -> bool {
        let grabbed = self.hud.on_modifier_edge(edge);
        tracing::debug!(grabbed, "wd-overlay: canvas grab");
        grabbed
    }

    /// Grab guard: Edit never grabs even with widgets present.
    #[must_use]
    pub fn wants_grab(&self) -> bool {
        tracing::trace!(mode = ?self.mode, "wd-overlay: canvas grab guard");
        self.mode.wants_grab()
    }
}
