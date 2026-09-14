//! Audit report.
//!
//! Covers dup keys, slot clash, off-screen, version, screen mismatch.
#![deny(missing_docs)]

use crate::nodes::Node;

/// One touch-slot row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotRow {
    /// Slot.
    pub slot: u8,
    /// Node id.
    pub node_id: String,
    /// Kind.
    pub kind: &'static str,
    /// Layer (base if empty).
    pub layer: String,
    /// Bindings.
    pub bindings: Vec<String>,
}

/// Non-touch row (macro/layershift).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuxRow {
    /// Node id.
    pub node_id: String,
    /// Kind.
    pub kind: &'static str,
    /// Layer.
    pub layer: String,
    /// Bindings.
    pub bindings: Vec<String>,
}

/// Audit report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditReport {
    /// Name.
    pub name: String,
    /// Issues (empty = clean).
    pub issues: Vec<String>,
    /// Touch rows sorted by slot.
    pub slots: Vec<SlotRow>,
    /// Aux rows sorted by id.
    pub aux: Vec<AuxRow>,
}

/// Audit loaded profile: dup keys, slot clash, off-screen, switchKey, version.
#[must_use]
pub fn audit(p: &crate::schema::Profile) -> AuditReport {
    tracing::info!(name = %p.name, "wd-input: audit");
    let mut issues = audit_screen(p, p.screen.map(|s| (s.width, s.height)));
    super::checks::check_offscreen(p, &mut issues);
    super::checks::check_switch_key(p, &mut issues);
    let (mut slots, mut aux) = rows(p);
    slots.sort_by_key(|r| r.slot);
    aux.sort_by(|a, b| a.node_id.cmp(&b.node_id));
    let report = AuditReport {
        name: p.name.clone(),
        issues,
        slots,
        aux,
    };
    tracing::info!(name = %p.name, issues = report.issues.len(), "wd-input: audited");
    report
}

/// Screen-level checks: version, screen required, daemon mismatch.
#[must_use]
pub fn audit_screen(p: &crate::schema::Profile, daemon: Option<(u32, u32)>) -> Vec<String> {
    let mut issues = vec![];
    if p.version != crate::PROFILE_VERSION {
        issues.push(format!("version {} != 1", p.version));
    }
    let Some(s) = p.screen else {
        issues.push("screen required".to_string());
        return issues;
    };
    if s.width == 0 || s.height == 0 {
        issues.push("screen zero".to_string());
    }
    if let Some((width, height)) = daemon
        && (width, height) != (0, 0)
        && (s.width, s.height) != (width, height)
    {
        issues.push(format!(
            "screen mismatch profile {}x{} vs daemon {width}x{height}",
            s.width, s.height
        ));
    }
    issues
}

fn rows(p: &crate::schema::Profile) -> (Vec<SlotRow>, Vec<AuxRow>) {
    let mut slots = vec![];
    let mut aux = vec![];
    for n in &p.nodes {
        let layer = display(n.layer());
        match n {
            Node::Wheel(w) => {
                slots.push(SlotRow {
                    slot: w.up_slot,
                    node_id: format!("{}:up", n.id()),
                    kind: n.kind(),
                    layer: layer.clone(),
                    bindings: vec!["WheelUp".into()],
                });
                slots.push(SlotRow {
                    slot: w.down_slot,
                    node_id: format!("{}:down", n.id()),
                    kind: n.kind(),
                    layer,
                    bindings: vec!["WheelDown".into()],
                });
            }
            Node::Macro { .. } | Node::LayerShift { .. } => aux.push(AuxRow {
                node_id: n.id().to_string(),
                kind: n.kind(),
                layer,
                bindings: n.bound_keys(),
            }),
            _ => slots.push(SlotRow {
                slot: n.slot().unwrap_or(0),
                node_id: n.id().to_string(),
                kind: n.kind(),
                layer,
                bindings: n.bound_keys(),
            }),
        }
    }
    (slots, aux)
}

fn display(layer: &str) -> String {
    if layer.trim().is_empty() {
        "base".to_string()
    } else {
        layer.to_string()
    }
}
