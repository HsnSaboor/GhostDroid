//! Canonical aim + repeat + wheel + macro + layer nodes.
#![deny(missing_docs)]

use wd_core::RelPos;

use crate::types::{Activation, AimCurve, LayerMode, MacroMode, MacroStep, Region};

/// Aim node fields.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AimNode {
    /// Id.
    pub id: String,
    /// Layer.
    #[serde(default)]
    pub layer: String,
    /// Slot.
    pub slot: u8,
    /// Anchor (default .75,.5).
    #[serde(default = "crate::nodes::default_anchor")]
    pub anchor: RelPos,
    /// Reach (0,0.45].
    #[serde(default = "crate::nodes::default_reach")]
    pub reach: f64,
    /// Sensitivity > 0.
    #[serde(default = "crate::nodes::default_sens")]
    pub sensitivity: f64,
    /// Curve.
    #[serde(default)]
    pub curve: AimCurve,
    /// Activation.
    #[serde(default, alias = "activation_mode")]
    pub activation: Activation,
    /// Activation key.
    #[serde(default)]
    pub activation_key: Option<String>,
    /// Invert Y.
    #[serde(default)]
    pub invert_y: bool,
    /// Legacy region (consumed by `normalized`).
    #[serde(default, skip_serializing)]
    pub region: Option<Region>,
}

/// Repeat-tap node fields.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RepeatNode {
    /// Id.
    pub id: String,
    /// Layer.
    #[serde(default)]
    pub layer: String,
    /// Slot.
    pub slot: u8,
    /// Pos.
    pub pos: RelPos,
    /// Key.
    pub key: String,
    /// Interval ms > 0.
    pub interval_ms: u64,
}

/// Wheel node fields.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WheelNode {
    /// Id.
    pub id: String,
    /// Layer.
    #[serde(default)]
    pub layer: String,
    /// Up slot.
    pub up_slot: u8,
    /// Up pos.
    pub up_pos: RelPos,
    /// Down slot.
    pub down_slot: u8,
    /// Down pos.
    pub down_pos: RelPos,
}

/// Macro node fields.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MacroNode {
    /// Id.
    pub id: String,
    /// Layer.
    #[serde(default)]
    pub layer: String,
    /// Key.
    pub key: String,
    /// Mode.
    #[serde(default)]
    pub mode: MacroMode,
    /// Steps.
    pub sequence: Vec<MacroStep>,
}

/// Layer-shift node fields.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ShiftNode {
    /// Id.
    pub id: String,
    /// Key.
    pub key: String,
    /// Layer name.
    pub layer_name: String,
    /// Mode.
    #[serde(default)]
    pub mode: LayerMode,
    /// Suspend base.
    #[serde(default)]
    pub suspend_base: bool,
}
