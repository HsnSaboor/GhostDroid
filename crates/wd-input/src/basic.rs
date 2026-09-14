//! Canonical tap + joystick + drag nodes.
#![deny(missing_docs)]

use wd_core::RelPos;

use crate::types::JoystickKeys;

/// Tap node fields.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TapNode {
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
}

/// Joystick node fields.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct JoyNode {
    /// Id.
    pub id: String,
    /// Layer.
    #[serde(default)]
    pub layer: String,
    /// Slot.
    pub slot: u8,
    /// Pos.
    pub pos: RelPos,
    /// Radius (0,1].
    pub radius: f64,
    /// Keys.
    pub keys: JoystickKeys,
}

/// Drag node fields.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DragNode {
    /// Id.
    pub id: String,
    /// Layer.
    #[serde(default)]
    pub layer: String,
    /// Slot.
    pub slot: u8,
    /// Start.
    pub start: RelPos,
    /// End.
    pub end: RelPos,
    /// Key.
    pub key: String,
    /// Duration ms > 0.
    pub duration_ms: u64,
}
