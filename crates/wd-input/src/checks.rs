//! Audit checks: dup keys, slots, off-screen, switchKey, list.
#![deny(missing_docs)]

use std::collections::{HashMap, HashSet};
use std::path::Path;

use wd_core::Result;

use crate::nodes::Node;

/// Dup keys across nodes (informational; phantom validate allows reuse).
#[must_use]
pub fn dup_keys(p: &crate::schema::Profile) -> Vec<String> {
    let mut out = vec![];
    let mut seen: HashMap<String, String> = HashMap::new();
    for n in &p.nodes {
        if matches!(n, Node::Aim { .. }) && n.bound_keys() == vec!["MouseMove".to_string()] {
            continue;
        }
        for key in n.bound_keys() {
            if key == "MouseMove" || key == "WheelUp" || key == "WheelDown" {
                continue;
            }
            let norm = key.trim().to_ascii_uppercase();
            if let Some(prev) = seen.get(&norm) {
                out.push(format!("dup key {key}: {prev} vs {}", n.id()));
            } else {
                seen.insert(norm, n.id().to_string());
            }
        }
    }
    out
}

/// Slot clashes across nodes (informational; phantom validate rejects).
#[must_use]
pub fn slot_clashes(p: &crate::schema::Profile) -> Vec<String> {
    let mut out = vec![];
    let mut seen = HashSet::new();
    for n in &p.nodes {
        for slot in n.slots() {
            if slot == crate::RESERVED_SLOT {
                out.push(format!("slot {slot} reserved ({})", n.id()));
            } else if !seen.insert(slot) {
                out.push(format!("slot clash {slot} ({})", n.id()));
            }
            if let Node::Wheel(w) = n
                && w.up_slot == w.down_slot
            {
                out.push(format!("wheel slots equal ({})", n.id()));
            }
        }
    }
    out
}

/// Off-screen positions.
pub(crate) fn check_offscreen(p: &crate::schema::Profile, issues: &mut Vec<String>) {
    for n in &p.nodes {
        for pos in n.positions() {
            if !pos.valid() {
                issues.push(format!("off-screen {} ({},{})", n.id(), pos.x, pos.y));
            }
        }
    }
}

/// Missing switchKey when layers used.
pub(crate) fn check_switch_key(p: &crate::schema::Profile, issues: &mut Vec<String>) {
    let has = p.nodes.iter().any(|n| matches!(n, Node::LayerShift { .. }));
    let wants = p.nodes.iter().any(|n| !n.layer().trim().is_empty());
    if wants && !has {
        issues.push("missing switchKey: layered nodes but no LayerShift".to_string());
    }
}

/// Load profile files under dir.
///
/// # Errors
///
/// Returns [`WdError`](wd_core::WdError) on unreadable dir.
pub fn list(dir: &Path) -> Result<Vec<String>> {
    tracing::debug!(dir = %dir.display(), "wd-input: list");
    let mut out = vec![];
    for entry in std::fs::read_dir(dir)?.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|x| x == "json") {
            out.push(path.to_string_lossy().to_string());
        }
    }
    out.sort();
    Ok(out)
}
