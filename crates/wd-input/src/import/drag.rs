//! Drag converter.
#![deny(missing_docs)]

use wd_core::Result;

use super::{DRAG_MS, SlotAlloc, err, key_name};
use crate::nodes::Node;

/// Drag node (`dragSpeed` 0-1 → ms).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn drag_node(
    slots: &mut SlotAlloc,
    key: Option<&String>,
    start: Option<&wd_core::RelPos>,
    end: Option<&wd_core::RelPos>,
    speed: f64,
    delay: Option<u64>,
    idx: usize,
) -> Result<Node> {
    let name = key
        .map(String::as_str)
        .map(key_name)
        .transpose()?
        .ok_or_else(|| err(&format!("node {idx}: drag missing key")))?;
    let (from, to) = start
        .zip(end)
        .ok_or_else(|| err(&format!("node {idx}: drag missing pos")))?;
    let dur = (DRAG_MS * speed.clamp(0.0, 1.0)).round().max(1.0) as u64 + delay.unwrap_or(0);
    Ok(Node::Drag(crate::basic::DragNode {
        id: format!("drag_{idx}"),
        layer: String::new(),
        slot: slots.take(),
        start: *from,
        end: *to,
        key: name,
        duration_ms: dur,
    }))
}
