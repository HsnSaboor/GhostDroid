//! Click converters.
#![deny(missing_docs)]

use wd_core::Result;

use super::{SlotAlloc, err, key_name};
use crate::nodes::Node;

/// Build tap node.
#[allow(clippy::missing_const_for_fn)]
pub fn tap_node(slots: &mut SlotAlloc, id: String, pos: wd_core::RelPos, key: String) -> Node {
    Node::Tap(crate::basic::TapNode {
        id,
        layer: String::new(),
        slot: slots.take(),
        pos,
        key,
    })
}

/// Double click → two taps.
pub fn twice(
    slots: &mut SlotAlloc,
    out: &mut Vec<Node>,
    key: Option<&String>,
    pos: Option<&wd_core::RelPos>,
    idx: usize,
) -> Result<()> {
    let (name, at) = pair(key, pos, idx)?;
    out.push(tap_node(
        slots,
        format!("click_twice_a_{idx}"),
        at,
        name.clone(),
    ));
    out.push(tap_node(slots, format!("click_twice_b_{idx}"), at, name));
    Ok(())
}

/// Multi click → macro.
pub fn multi_node(
    slots: &mut SlotAlloc,
    key: Option<&String>,
    clicks: &[crate::qt_nodes::QtClickNode],
    idx: usize,
) -> Result<Node> {
    let name = key
        .map(String::as_str)
        .map(key_name)
        .transpose()?
        .ok_or_else(|| err(&format!("node {idx}: multi missing key")))?;
    let mut seq = vec![];
    for click in clicks {
        let Some(at) = click.pos else {
            return Err(err(&format!("node {idx}: multi click missing pos")));
        };
        let slot = slots.take();
        seq.push(crate::types::MacroStep {
            action: crate::types::MacroAction::Down,
            pos: Some(at),
            slot,
            delay_ms: click.delay,
        });
        seq.push(crate::types::MacroStep {
            action: crate::types::MacroAction::Up,
            pos: Some(at),
            slot,
            delay_ms: 0,
        });
    }
    Ok(Node::Macro(crate::adv::MacroNode {
        id: format!("multi_{idx}"),
        layer: String::new(),
        key: name,
        mode: crate::types::MacroMode::OneShot,
        sequence: seq,
    }))
}

/// Key+pos pair.
pub fn pair(
    key: Option<&String>,
    pos: Option<&wd_core::RelPos>,
    idx: usize,
) -> Result<(String, wd_core::RelPos)> {
    match (key, pos) {
        (Some(name), Some(at)) => Ok((key_name(name)?, *at)),
        _ => Err(err(&format!("node {idx}: click missing key/pos"))),
    }
}
