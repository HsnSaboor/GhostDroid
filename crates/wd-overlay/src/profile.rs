//! Profile bridge: overlay calls `wd-input`, never owns schema.
//!
//! Plan 07 DRY: `wd-input::{load,validate,audit}` + `wd-shell` viewmodels.
//! Every call logs so Mapping/Edit stays traceable.
#![deny(missing_docs)]

use std::path::Path;

use crate::canvas::EditorCanvas;
use crate::capture::normalize_capture;
use crate::widgets::WidgetKind;

/// Load a profile via shared `wd-input` (plan 03 schema).
///
/// # Errors
///
/// Returns [`wd_core::WdError`] on IO, JSON, or validation failure.
pub fn load_profile(path: &Path) -> wd_core::Result<wd_input::Profile> {
    tracing::info!(path = %path.display(), "wd-overlay: profile load");
    let profile = wd_input::load(path)?;
    tracing::info!(name = %profile.name, "wd-overlay: profile loaded");
    Ok(profile)
}

/// Validate via shared `wd-input::validate`.
///
/// # Errors
///
/// Returns [`wd_core::WdError::Validation`](wd_core::WdError::Validation) on failure.
pub fn validate_profile(p: &wd_input::Profile) -> wd_core::Result<()> {
    tracing::debug!(name = %p.name, "wd-overlay: profile validate");
    let out = wd_input::validate(p);
    tracing::info!(name = %p.name, ok = out.is_ok(), "wd-overlay: profile validated");
    out
}

/// Audit via shared `wd-input::audit`.
#[must_use]
pub fn audit_profile(p: &wd_input::Profile) -> wd_input::AuditReport {
    tracing::debug!(name = %p.name, "wd-overlay: profile audit");
    let report = wd_input::audit(p);
    tracing::info!(issues = report.issues.len(), "wd-overlay: profile audited");
    report
}

/// Canvas bind check via shared `wd-input` allowlist.
#[must_use]
pub fn canvas_binds(canvas: &EditorCanvas) -> Vec<String> {
    tracing::debug!("wd-overlay: canvas bind check");
    let mut out = vec![];
    for w in &canvas.widgets {
        if w.key.trim().is_empty() {
            continue;
        }
        if normalize_capture(&w.key).is_none() {
            tracing::warn!(id = %w.id, "wd-overlay: bad bind");
            out.push(format!("{}: bad key {}", w.id, w.key));
        }
    }
    out
}

/// Shared shell viewmodel (DRY, plan 06+07): profile page reads this.
#[must_use]
pub fn canvas_keymap(canvas: &EditorCanvas, profile: &str) -> wd_shell::KeymapState {
    tracing::debug!(profile, "wd-overlay: keymap viewmodel");
    let fire = canvas
        .widgets
        .iter()
        .find(|w| w.kind == WidgetKind::Fire)
        .map_or_else(String::new, |w| w.key.clone());
    wd_shell::KeymapState {
        profile: profile.to_string(),
        fire_key: fire,
        tab_index: 0,
        node_count: canvas.widgets.len(),
    }
}
