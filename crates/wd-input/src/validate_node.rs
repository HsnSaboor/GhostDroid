//! Per-node dispatch.
#![deny(missing_docs)]

use wd_core::{Result, WdError};

use crate::nodes::Node;

mod aim;
mod shape;

/// Validate one node.
///
/// # Errors
///
/// Returns [`WdError::Validation`](wd_core::WdError::Validation) on any failure.
pub fn validate_node(node: &Node) -> Result<()> {
    tracing::trace!(id = node.id(), "wd-input: validate node");
    let id = node.id();
    if id.trim().is_empty() {
        return Err(err("nodes.id", "node id cannot be empty"));
    }
    if !matches!(node, Node::LayerShift(_)) && node.layer().chars().any(char::is_whitespace) {
        return Err(err(
            &field(id, "layer"),
            "layer names cannot contain whitespace",
        ));
    }
    match node {
        Node::Tap(t) | Node::ToggleTap(t) => {
            shape::pos_ok(t.pos, &field(id, "pos"))?;
            shape::key_ok(&t.key, &field(id, "key"))?;
        }
        Node::RepeatTap(r) => {
            shape::pos_ok(r.pos, &field(id, "pos"))?;
            shape::key_ok(&r.key, &field(id, "key"))?;
            if r.interval_ms == 0 {
                return Err(err(&field(id, "interval_ms"), "interval_ms must be > 0"));
            }
        }
        Node::Joystick(j) => shape::joystick(id, j.pos, j.radius, &j.keys)?,
        Node::Drag(d) => shape::drag(id, d.start, d.end, &d.key, d.duration_ms)?,
        Node::Aim(a) => aim::check(
            id,
            a.anchor,
            a.reach,
            a.sensitivity,
            a.activation,
            a.activation_key.as_deref(),
            a.region,
        )?,
        Node::Wheel(w) => shape::wheel(id, w.up_pos, w.down_pos, w.up_slot, w.down_slot)?,
        Node::Macro(m) => shape::check_macro(id, &m.key, &m.sequence)?,
        Node::LayerShift(s) => {
            shape::key_ok(&s.key, &field(id, "key"))?;
            if s.layer_name.trim().is_empty() {
                return Err(err(&field(id, "layer_name"), "layer_name cannot be empty"));
            }
            if s.layer_name.chars().any(char::is_whitespace) {
                return Err(err(&field(id, "layer_name"), "no whitespace in layer_name"));
            }
        }
    }
    Ok(())
}

pub(crate) fn field(id: &str, name: &str) -> String {
    format!("nodes.{id}.{name}")
}

pub(crate) fn err(field: &str, msg: &str) -> WdError {
    WdError::Validation(format!("{field}: {msg}"))
}
