//! `QtScrcpy` import.
//!
//! Import-only, no dual store.
//! Refs: `.devdocs/QtScrcpy/docs/KeyMapDes.md:94`.
#![deny(missing_docs)]

use wd_core::{Result, WdError};

use crate::nodes::Node;
use crate::schema::{Profile, Screen};
use crate::types::LayerMode;

mod click;
mod drag;
mod steer;

/// Max aim reach cap (phantom parity 0.45).
pub(super) const MAX_REACH: f64 = 0.45;
/// Qt `dragSpeed` (0-1) → ms scale.
pub(super) const DRAG_MS: f64 = 500.0;

/// Import Qt JSON text → canonical profile.
///
/// # Errors
/// Returns [`WdError`] on bad JSON, unknown keys, or failed validation.
pub fn import_qt(name: &str, raw: &str, screen: Screen) -> Result<Profile> {
    tracing::info!(name, "wd-input: import qt");
    let qt: crate::qt::QtKeymap = serde_json::from_str(raw)?;
    let mut slots = SlotAlloc::default();
    let mut out: Vec<Node> = vec![];
    if let Some(mm) = &qt.mouse_move {
        out.push(steer::aim_node(
            &mut slots, mm.start, mm.ratio, mm.ratio_x, mm.ratio_y,
        ));
        if let Some((key, pos)) = mm
            .small_eyes
            .as_ref()
            .and_then(|se| se.key.as_ref().zip(se.pos.as_ref()))
        {
            out.push(click::tap_node(
                &mut slots,
                format!("small_eyes_{}", out.len()),
                *pos,
                key_name(key)?,
            ));
        }
    }
    for (idx, node) in qt.nodes.iter().enumerate() {
        dispatch(&mut slots, &mut out, node, idx)?;
    }
    if let Some(sw) = &qt.switch_key {
        out.push(Node::LayerShift(crate::adv::ShiftNode {
            id: "switch_key".to_string(),
            key: key_name(sw)?,
            layer_name: "custom".to_string(),
            mode: LayerMode::Toggle,
            suspend_base: false,
        }));
    }
    let profile = Profile {
        name: name.to_string(),
        version: crate::PROFILE_VERSION,
        screen: Some(screen),
        global_sensitivity: 1.0,
        nodes: out,
    };
    crate::validate::validate(&profile)?;
    tracing::info!(name, count = profile.nodes.len(), "wd-input: imported");
    Ok(profile)
}

fn dispatch(
    slots: &mut SlotAlloc,
    out: &mut Vec<Node>,
    node: &crate::qt_nodes::QtNode,
    idx: usize,
) -> Result<()> {
    tracing::trace!(idx, "wd-input: import node");
    match node {
        crate::qt_nodes::QtNode::Click { key, pos } => {
            let (name, at) = click::pair(key.as_ref(), pos.as_ref(), idx)?;
            out.push(click::tap_node(slots, format!("click_{idx}"), at, name));
        }
        crate::qt_nodes::QtNode::ClickTwice { key, pos } => {
            click::twice(slots, out, key.as_ref(), pos.as_ref(), idx)?;
        }
        crate::qt_nodes::QtNode::ClickMulti { key, clicks } => {
            out.push(click::multi_node(slots, key.as_ref(), clicks, idx)?);
        }
        crate::qt_nodes::QtNode::Drag {
            key,
            start,
            end,
            speed,
            delay,
        } => {
            out.push(drag::drag_node(
                slots,
                key.as_ref(),
                start.as_ref(),
                end.as_ref(),
                *speed,
                *delay,
                idx,
            )?);
        }
        crate::qt_nodes::QtNode::Steer {
            center,
            left,
            right,
            up,
            down,
            ..
        } => {
            out.push(steer::steer_node(
                slots,
                center.as_ref(),
                left.as_ref(),
                right.as_ref(),
                up.as_ref(),
                down.as_ref(),
                idx,
            )?);
        }
    }
    Ok(())
}

/// Slot allocator (sequential, saturating).
#[derive(Debug, Default)]
pub(crate) struct SlotAlloc {
    /// Next slot.
    next: u8,
}

impl SlotAlloc {
    pub(crate) const fn take(&mut self) -> u8 {
        let slot = self.next;
        self.next = self.next.saturating_add(1);
        slot
    }
}

pub(crate) fn key_name(raw: &str) -> Result<String> {
    crate::keys::canonical(raw).ok_or_else(|| WdError::Validation(format!("unknown key '{raw}'")))
}

pub(crate) fn err(msg: &str) -> WdError {
    WdError::Validation(msg.to_string())
}
