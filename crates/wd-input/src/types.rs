//! Shared node field types. Port of phantom `profile.rs` enums.
//! Refs: `.devdocs/phantom/phantom/src/profile.rs:24-60`.
#![deny(missing_docs)]

use wd_core::RelPos;

/// Aim response curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AimCurve {
    /// Linear.
    Linear,
    /// Precision.
    Precision,
    /// Balanced (default).
    #[default]
    Balanced,
}

/// Aim activation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Activation {
    /// Always on (default).
    #[default]
    AlwaysOn,
    /// While held.
    WhileHeld,
    /// Toggle.
    Toggle,
}

/// Layer switch mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayerMode {
    /// Hold (default).
    #[default]
    Hold,
    /// Toggle.
    Toggle,
}

/// Macro run mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MacroMode {
    /// Cancel on release (default).
    #[default]
    CancelOnRelease,
    /// One shot.
    OneShot,
}

/// Macro step action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MacroAction {
    /// Down.
    Down,
    /// Up.
    Up,
}

/// Single macro step.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MacroStep {
    /// Action.
    pub action: MacroAction,
    /// Pos (required for down).
    pub pos: Option<RelPos>,
    /// Slot.
    pub slot: u8,
    /// Delay ms.
    #[serde(default)]
    pub delay_ms: u64,
}

/// WASD keys.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct JoystickKeys {
    /// Up.
    pub up: String,
    /// Down.
    pub down: String,
    /// Left.
    pub left: String,
    /// Right.
    pub right: String,
}

/// Joystick stick mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JoystickMode {
    /// Fixed stick (default).
    #[default]
    Fixed,
    /// Follow finger.
    Follow,
}

/// Legacy aim region (consumed by `normalized`).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Region {
    /// X.
    pub x: f64,
    /// Y.
    pub y: f64,
    /// W.
    pub w: f64,
    /// H.
    pub h: f64,
}
