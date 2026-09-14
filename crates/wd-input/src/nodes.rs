//! Canonical node enum.
//!
//! Port of phantom `profile.rs:62-165`.
#![deny(missing_docs)]

use wd_core::RelPos;

use crate::adv::{AimNode, MacroNode, RepeatNode, ShiftNode, WheelNode};
use crate::basic::{DragNode, JoyNode, TapNode};

/// Canonical node. One schema only.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Node {
    /// Tap (`hold_tap` legacy alias).
    #[serde(alias = "hold_tap")]
    Tap(TapNode),
    /// Toggle tap.
    ToggleTap(TapNode),
    /// Joystick.
    Joystick(JoyNode),
    /// Drag.
    Drag(DragNode),
    /// Aim (`mouse_camera` legacy alias).
    #[serde(alias = "mouse_camera")]
    Aim(AimNode),
    /// Repeat tap.
    RepeatTap(RepeatNode),
    /// Wheel (two slots).
    Wheel(WheelNode),
    /// Macro.
    Macro(MacroNode),
    /// Layer shift.
    LayerShift(ShiftNode),
}

/// Default aim anchor (.75,.5). Ref `profile.rs:default_aim_anchor`.
#[must_use]
pub const fn default_anchor() -> RelPos {
    RelPos { x: 0.75, y: 0.5 }
}

/// Default aim reach. Ref `profile.rs:default_aim_reach`.
#[must_use]
pub const fn default_reach() -> f64 {
    0.18
}

/// Default sensitivity.
#[must_use]
pub const fn default_sens() -> f64 {
    1.0
}
