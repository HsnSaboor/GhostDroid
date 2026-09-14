//! `QtScrcpy` raw shapes.
//!
//! Import-only, no dual store.
#![deny(missing_docs)]

use wd_core::RelPos;

use crate::qt_nodes::QtNode;

/// Raw Qt keymap file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QtKeymap {
    /// Switch key.
    #[serde(rename = "switchKey", default)]
    pub switch_key: Option<String>,
    /// Mouse move map.
    #[serde(rename = "mouseMoveMap", default)]
    pub mouse_move: Option<QtMouseMove>,
    /// Key nodes.
    #[serde(rename = "keyMapNodes", default)]
    pub nodes: Vec<QtNode>,
    /// `ScrcpyKeyMapper` canvas width.
    #[serde(default)]
    pub width: Option<u32>,
    /// Canvas height.
    #[serde(default)]
    pub height: Option<u32>,
}

/// Mouse move map.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QtMouseMove {
    /// Start pos.
    #[serde(rename = "startPos")]
    pub start: Option<RelPos>,
    /// Speed ratio.
    #[serde(rename = "speedRatio", default)]
    pub ratio: Option<f64>,
    /// X ratio.
    #[serde(rename = "speedRatioX", default)]
    pub ratio_x: Option<f64>,
    /// Y ratio.
    #[serde(rename = "speedRatioY", default)]
    pub ratio_y: Option<f64>,
    /// Small eyes button.
    #[serde(rename = "smallEyes", default)]
    pub small_eyes: Option<QtClick>,
}

/// Click shape (also smallEyes).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QtClick {
    /// Key.
    #[serde(default)]
    pub key: Option<String>,
    /// Pos.
    #[serde(default)]
    pub pos: Option<RelPos>,
}
