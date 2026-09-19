//! Steer + aim converters.
#![deny(missing_docs)]

use wd_core::Result;

use super::{MAX_REACH, SlotAlloc, err, key_name};
use crate::nodes::Node;
use crate::types::{Activation, AimCurve, JoystickKeys};

/// Steer wheel → joystick.
#[allow(clippy::too_many_arguments)]
pub fn steer_node(
    slots: &mut SlotAlloc,
    center: Option<&wd_core::RelPos>,
    left: Option<&String>,
    right: Option<&String>,
    up: Option<&String>,
    down: Option<&String>,
    idx: usize,
) -> Result<Node> {
    let Some(at) = center else {
        return Err(err(&format!("node {idx}: steer missing center")));
    };
    Ok(Node::Joystick(crate::basic::JoyNode {
        id: format!("steer_{idx}"),
        layer: String::new(),
        slot: slots.take(),
        pos: *at,
        radius: 0.1,
        mode: crate::types::JoystickMode::default(),
        keys: JoystickKeys {
            up: opt_key(up, idx, "up")?,
            down: opt_key(down, idx, "down")?,
            left: opt_key(left, idx, "left")?,
            right: opt_key(right, idx, "right")?,
        },
    }))
}

fn opt_key(key: Option<&String>, idx: usize, dir: &str) -> Result<String> {
    key.map(String::as_str)
        .map(key_name)
        .transpose()?
        .ok_or_else(|| err(&format!("node {idx}: steer missing {dir}")))
}

/// Mouse move → aim (ratio ≥ 0.00225 per `KeyMapDes.md`).
pub fn aim_node(
    slots: &mut SlotAlloc,
    start: Option<wd_core::RelPos>,
    ratio: Option<f64>,
    ratio_x: Option<f64>,
    ratio_y: Option<f64>,
) -> Node {
    let anchor = start.unwrap_or(wd_core::RelPos { x: 0.75, y: 0.5 });
    let speed = ratio.or(ratio_x).or(ratio_y).unwrap_or(1.0).max(0.002_25);
    Node::Aim(crate::adv::AimNode {
        id: "mouse_move".to_string(),
        layer: String::new(),
        slot: slots.take(),
        anchor,
        reach: (0.18 / speed).clamp(0.05, MAX_REACH),
        sensitivity: 1.0,
        curve: AimCurve::Balanced,
        activation: Activation::AlwaysOn,
        activation_key: None,
        invert_y: false,
        deadzone: None,
        region: None,
    })
}
