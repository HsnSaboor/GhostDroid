//! `QtScrcpy` key node shapes.
#![deny(missing_docs)]

use wd_core::RelPos;

/// Raw key node.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum QtNode {
    /// Click.
    #[serde(rename = "KMT_CLICK")]
    Click {
        /// Key.
        #[serde(default)]
        key: Option<String>,
        /// Pos.
        #[serde(default)]
        pos: Option<RelPos>,
    },
    /// Double click → two taps.
    #[serde(rename = "KMT_CLICK_TWICE")]
    ClickTwice {
        /// Key.
        #[serde(default)]
        key: Option<String>,
        /// Pos.
        #[serde(default)]
        pos: Option<RelPos>,
    },
    /// Multi click.
    #[serde(rename = "KMT_CLICK_MULTI")]
    ClickMulti {
        /// Key.
        #[serde(default)]
        key: Option<String>,
        /// Clicks.
        #[serde(rename = "clickNodes", default)]
        clicks: Vec<QtClickNode>,
    },
    /// Drag.
    #[serde(rename = "KMT_DRAG")]
    Drag {
        /// Key.
        #[serde(default)]
        key: Option<String>,
        /// Start.
        #[serde(rename = "startPos", default)]
        start: Option<RelPos>,
        /// End.
        #[serde(rename = "endPos", default)]
        end: Option<RelPos>,
        /// Speed 0-1.
        #[serde(rename = "dragSpeed", default = "full_speed")]
        speed: f64,
        /// Start delay ms.
        #[serde(rename = "startDelay", default)]
        delay: Option<u64>,
    },
    /// Steer wheel.
    #[serde(rename = "KMT_STEER_WHEEL")]
    Steer {
        /// Center.
        #[serde(rename = "centerPos", default)]
        center: Option<RelPos>,
        /// Left key.
        #[serde(rename = "leftKey", default)]
        left: Option<String>,
        /// Right key.
        #[serde(rename = "rightKey", default)]
        right: Option<String>,
        /// Up key.
        #[serde(rename = "upKey", default)]
        up: Option<String>,
        /// Down key.
        #[serde(rename = "downKey", default)]
        down: Option<String>,
        /// Left offset.
        #[serde(rename = "leftOffset", default = "steer_offset")]
        left_offset: f64,
        /// Right offset.
        #[serde(rename = "rightOffset", default = "steer_offset")]
        right_offset: f64,
        /// Up offset.
        #[serde(rename = "upOffset", default = "steer_offset")]
        up_offset: f64,
        /// Down offset.
        #[serde(rename = "downOffset", default = "steer_offset")]
        down_offset: f64,
    },
}

/// Multi-click entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QtClickNode {
    /// Delay ms.
    #[serde(default)]
    pub delay: u64,
    /// Pos.
    #[serde(default)]
    pub pos: Option<RelPos>,
}

const fn full_speed() -> f64 {
    1.0
}

const fn steer_offset() -> f64 {
    0.1
}
