//! Editor widget palette: 9 waydroid-helper behaviors + rects.
//!
//! Behavior port of `.devdocs/waydroid-helper/waydroid_helper/controller/widgets/`
//! (`single_click.py:375`, `repeated_click.py:593`, `aim.py:888` rect,
//! `fire.py:866` LMB-locked, `directional_pad.py:791`, `skill_casting.py:1191`,
//! `right_click_to_walk.py:692`, `macro.py:975`); sizes from
//! `base_widget.py:62-63` (mapping 50x50), drag/resize `base_widget.py:117-242`.
//! Rects stored relative 0-1 (`QtScrcpy` rule); px via shared
//! `wd_core::RelPos::to_pixels`, never a local copy (`DRY`).
#![deny(missing_docs)]

/// Mapping-mode default widget size px (`base_widget.py:62-63`).
pub const MAPPING_PX: f32 = 50.0;

/// Editor widget kind (9-helper palette + owned cursor).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WidgetKind {
    /// Tap (`single_click.py:375`).
    #[default]
    Tap,
    /// Repeat tap (`repeated_click.py:593`).
    RepeatTap,
    /// Aim rect (`aim.py:888`).
    Aim,
    /// Fire, LMB-locked (`fire.py:866`).
    Fire,
    /// D-pad (`directional_pad.py:791`).
    Dpad,
    /// Skill cast (`skill_casting.py:1191`).
    SkillCast,
    /// Right-click-to-walk (`right_click_to_walk.py:692`).
    RightWalk,
    /// Macro (`macro.py:975` click/press/release/switch).
    Macro,
    /// `MenuTouch` owned cursor (phantom `overlay.rs` menu-touch).
    MenuTouch,
}

impl WidgetKind {
    /// Canonical wd-input node kind bound for audit. `None` = owned cursor.
    #[must_use]
    pub fn bound_node_kind(self) -> Option<&'static str> {
        tracing::trace!(kind = ?self, "wd-overlay: widget bind kind");
        match self {
            Self::Tap | Self::Fire | Self::RightWalk => Some("tap"),
            Self::RepeatTap => Some("repeat_tap"),
            Self::Aim => Some("aim"),
            Self::Dpad => Some("joystick"),
            Self::SkillCast => Some("drag"),
            Self::Macro => Some("macro"),
            Self::MenuTouch => None,
        }
    }

    /// All 9 palette kinds in menu order.
    #[must_use]
    pub fn palette() -> [Self; 9] {
        tracing::trace!("wd-overlay: palette");
        [
            Self::Tap,
            Self::RepeatTap,
            Self::Aim,
            Self::Fire,
            Self::Dpad,
            Self::SkillCast,
            Self::RightWalk,
            Self::Macro,
            Self::MenuTouch,
        ]
    }
}

/// Widget rect, relative 0-1 axes (`QtScrcpy` rule).
#[derive(Debug, Clone, PartialEq)]
pub struct WidgetRect {
    /// Widget id (layout `widgets[]` entry).
    pub id: String,
    /// Palette kind.
    pub kind: WidgetKind,
    /// Rel x (0-1).
    pub x: f32,
    /// Rel y (0-1).
    pub y: f32,
    /// Rel w (0-1).
    pub w: f32,
    /// Rel h (0-1).
    pub h: f32,
    /// Bound key label (capture result, e.g. `LeftCtrl+A`).
    pub key: String,
}

impl WidgetRect {
    /// Place at rel pos with 50x50px default size at `screen`.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn placed(id: &str, kind: WidgetKind, x: f32, y: f32, screen: (u32, u32)) -> Self {
        let w = (f64::from(MAPPING_PX) / f64::from(screen.0.max(1))) as f32;
        let h = (f64::from(MAPPING_PX) / f64::from(screen.1.max(1))) as f32;
        tracing::info!(id, kind = ?kind, x, y, "wd-overlay: widget place");
        Self {
            id: id.to_string(),
            kind,
            x: x.clamp(0.0, 1.0),
            y: y.clamp(0.0, 1.0),
            w,
            h,
            key: String::new(),
        }
    }

    /// Top-left px via shared `RelPos::to_pixels` (`DRY`, `QtScrcpy` rule).
    #[must_use]
    pub fn top_left_px(&self, screen: (u32, u32)) -> (i32, i32) {
        tracing::trace!(id = %self.id, "wd-overlay: widget to px");
        wd_core::RelPos {
            x: self.x,
            y: self.y,
        }
        .to_pixels(screen.0, screen.1)
    }

    /// Size px (50x50 default round-trips at placement screen).
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn size_px(&self, screen: (u32, u32)) -> (i32, i32) {
        tracing::trace!(id = %self.id, "wd-overlay: widget size px");
        let w = (f64::from(self.w) * f64::from(screen.0)).round() as i32;
        let h = (f64::from(self.h) * f64::from(screen.1)).round() as i32;
        (w.max(1), h.max(1))
    }

    /// Move by rel delta, clamped 0-1. Logs move.
    pub fn moved(&mut self, dx: f32, dy: f32) {
        self.x = (self.x + dx).clamp(0.0, 1.0);
        self.y = (self.y + dy).clamp(0.0, 1.0);
        tracing::info!(id = %self.id, x = self.x, y = self.y, "wd-overlay: widget move");
    }

    /// Resize by rel delta, min 50px-equiv. Logs resize.
    #[allow(clippy::cast_possible_truncation)]
    pub fn resized(&mut self, dw: f32, dh: f32, screen: (u32, u32)) {
        let min_w = (f64::from(MAPPING_PX) / f64::from(screen.0.max(1))) as f32;
        let min_h = (f64::from(MAPPING_PX) / f64::from(screen.1.max(1))) as f32;
        self.w = (self.w + dw).clamp(min_w, 1.0);
        self.h = (self.h + dh).clamp(min_h, 1.0);
        tracing::info!(id = %self.id, w = self.w, h = self.h, "wd-overlay: widget resize");
    }
}
