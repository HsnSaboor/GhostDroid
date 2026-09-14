//! Profile load + validate.
//!
//! Port of phantom `profile.rs:466-660`.
#![deny(missing_docs)]

use std::collections::HashSet;
use std::path::Path;

use wd_core::{RelPos, Result, WdError};

use crate::nodes::Node;
use crate::schema::Profile;
use crate::validate_node::validate_node;

/// Load + normalize + validate from path.
///
/// # Errors
///
/// Returns [`WdError`] on IO, JSON, or validation failure.
pub fn load(path: &Path) -> Result<Profile> {
    tracing::info!(path = %path.display(), "wd-input: load profile");
    let raw = std::fs::read_to_string(path)?;
    let profile: Profile = serde_json::from_str(&raw)?;
    let profile = normalized(profile);
    validate(&profile)?;
    tracing::info!(name = %profile.name, nodes = profile.nodes.len(), "wd-input: loaded");
    Ok(profile)
}

/// Normalize legacy: `region` → anchor/reach (round3).
#[must_use]
pub fn normalized(mut profile: Profile) -> Profile {
    tracing::debug!(name = %profile.name, "wd-input: normalize");
    for node in &mut profile.nodes {
        if let Node::Aim(a) = node
            && let Some(rect) = a.region.take()
        {
            normalize_aim(&mut a.anchor, &mut a.reach, &rect);
        }
    }
    profile
}

#[allow(clippy::cast_possible_truncation)]
fn normalize_aim(anchor: &mut RelPos, reach: &mut f64, rect: &crate::types::Region) {
    if (f64::from(anchor.x) - 0.75).abs() < f64::EPSILON
        && (f64::from(anchor.y) - 0.5).abs() < f64::EPSILON
    {
        *anchor = RelPos {
            x: round3(rect.x + rect.w / 2.0),
            y: round3(rect.y + rect.h / 2.0),
        };
    }
    if (*reach - 0.18).abs() < f64::EPSILON {
        *reach = f64::from(round3((rect.w.min(rect.h) / 2.0).clamp(0.05, 0.45)));
    }
    tracing::debug!(reach = *reach, "wd-input: region normalized");
}

#[allow(clippy::cast_possible_truncation)]
fn round3(value: f64) -> f32 {
    ((value * 1000.0).round() / 1000.0) as f32
}

/// Validate profile. First error wins (phantom parity).
///
/// # Errors
///
/// Returns [`WdError::Validation`](wd_core::WdError::Validation) on any failure.
pub fn validate(profile: &Profile) -> Result<()> {
    tracing::debug!(name = %profile.name, "wd-input: validate");
    base_checks(profile)?;
    let mut ids = HashSet::new();
    let mut slots = HashSet::new();
    for node in &profile.nodes {
        if !ids.insert(node.id()) {
            return Err(err(&format!("nodes.{}.id", node.id()), "duplicate node id"));
        }
        claim_slots(&mut slots, node)?;
        validate_node(node)?;
    }
    tracing::debug!(name = %profile.name, "wd-input: valid");
    Ok(())
}

fn base_checks(profile: &Profile) -> Result<()> {
    if profile.name.trim().is_empty() {
        return Err(err("name", "profile name cannot be empty"));
    }
    if profile.version != crate::PROFILE_VERSION {
        return Err(err(
            "version",
            &format!("unsupported version {}, expected 1", profile.version),
        ));
    }
    if profile.global_sensitivity <= 0.0 {
        return Err(err(
            "global_sensitivity",
            "global_sensitivity must be positive",
        ));
    }
    let Some(screen) = profile.screen.as_ref() else {
        return Err(err("screen", "screen override is required"));
    };
    if screen.width == 0 || screen.height == 0 {
        return Err(err("screen", "screen width and height must be > 0"));
    }
    if profile.nodes.is_empty() {
        return Err(err("nodes", "profile has no nodes"));
    }
    Ok(())
}

fn claim_slots(slots: &mut HashSet<u8>, node: &Node) -> Result<()> {
    match node {
        Node::Wheel(w) => {
            for slot in [w.up_slot, w.down_slot] {
                slot_unique(slots, node.id(), slot)?;
            }
        }
        Node::Macro { .. } | Node::LayerShift { .. } => {}
        _ => {
            if let Some(slot) = node.slot() {
                slot_unique(slots, node.id(), slot)?;
            }
        }
    }
    Ok(())
}

fn slot_unique(slots: &mut HashSet<u8>, id: &str, slot: u8) -> Result<()> {
    if slot == crate::RESERVED_SLOT {
        return Err(err("slot", "slot reserved for runtime mouse-touch"));
    }
    if !slots.insert(slot) {
        return Err(err(
            &format!("nodes.{id}.slot"),
            &format!("duplicate slot {slot}"),
        ));
    }
    Ok(())
}

fn err(field: &str, msg: &str) -> WdError {
    WdError::Validation(format!("{field}: {msg}"))
}
