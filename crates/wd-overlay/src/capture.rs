//! Key-capture: double-click capture per plan 07 helper flow.
//!
//! Accepts letters/numbers/F-keys/Ctrl/Alt/Shift, Left/Right/Middle,
//! combos e.g. `Ctrl+A` (`KEY_MAPPING.md` flow). Single source:
//! `wd_input::keys::canonical` per `+`-separated part (DRY).
#![deny(missing_docs)]

/// Mouse buttons capturable by double-click (plan 07 flow).
pub const MOUSE_BUTTONS: [&str; 3] = ["MouseLeft", "MouseRight", "MouseMiddle"];

fn normalize_inner(raw: &str) -> Option<String> {
    let mut parts = Vec::new();
    for part in raw.split('+') {
        parts.push(wd_input::keys::canonical(part)?);
    }
    Some(parts.join("+"))
}

/// Normalize a capture string (`Ctrl+A`, `F1`, `MouseLeft`). Logs bind.
#[must_use]
pub fn normalize_capture(raw: &str) -> Option<String> {
    let out = normalize_inner(raw)?;
    tracing::info!(raw, normalized = %out, "wd-overlay: capture bound");
    Some(out)
}

/// True when the capture string binds (keys/F/mouse/combos).
#[must_use]
pub fn capturable(raw: &str) -> bool {
    tracing::trace!(raw, "wd-overlay: capture check");
    normalize_inner(raw).is_some()
}
