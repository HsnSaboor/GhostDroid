//! Shape checks: pos, key, joystick, drag, wheel, macro.
#![deny(missing_docs)]

use wd_core::Result;

use super::{err, field};

/// Position in [0,1].
pub(super) fn pos_ok(pos: wd_core::RelPos, field: &str) -> Result<()> {
    let (x, y) = (f64::from(pos.x), f64::from(pos.y));
    if !(0.0..=1.0).contains(&x) || !(0.0..=1.0).contains(&y) {
        return Err(err(field, "coordinates out of range [0, 1]"));
    }
    Ok(())
}

/// Known key.
pub(super) fn key_ok(key: &str, field: &str) -> Result<()> {
    if !crate::keys::known(key) {
        return Err(err(field, &format!("unknown key '{key}'")));
    }
    Ok(())
}

/// Joystick shape.
pub(super) fn joystick(
    id: &str,
    pos: wd_core::RelPos,
    radius: f64,
    keys: &crate::types::JoystickKeys,
) -> Result<()> {
    pos_ok(pos, &field(id, "pos"))?;
    if radius <= 0.0 || radius > 1.0 {
        return Err(err(&field(id, "radius"), "radius must be in (0, 1]"));
    }
    key_ok(&keys.up, &field(id, "keys.up"))?;
    key_ok(&keys.down, &field(id, "keys.down"))?;
    key_ok(&keys.left, &field(id, "keys.left"))?;
    key_ok(&keys.right, &field(id, "keys.right"))?;
    Ok(())
}

/// Drag shape.
pub(super) fn drag(
    id: &str,
    start: wd_core::RelPos,
    end: wd_core::RelPos,
    key: &str,
    duration_ms: u64,
) -> Result<()> {
    pos_ok(start, &field(id, "start"))?;
    pos_ok(end, &field(id, "end"))?;
    key_ok(key, &field(id, "key"))?;
    if duration_ms == 0 {
        return Err(err(&field(id, "duration_ms"), "duration_ms must be > 0"));
    }
    Ok(())
}

/// Wheel shape.
pub(super) fn wheel(
    id: &str,
    up: wd_core::RelPos,
    down: wd_core::RelPos,
    up_slot: u8,
    down_slot: u8,
) -> Result<()> {
    pos_ok(up, &field(id, "up_pos"))?;
    pos_ok(down, &field(id, "down_pos"))?;
    if up_slot == down_slot {
        return Err(err(&field(id, "down_slot"), "up/down slots must differ"));
    }
    Ok(())
}

/// Macro shape.
pub(super) fn check_macro(id: &str, key: &str, sequence: &[crate::types::MacroStep]) -> Result<()> {
    key_ok(key, &field(id, "key"))?;
    if sequence.is_empty() {
        return Err(err(
            &field(id, "sequence"),
            "macro sequence cannot be empty",
        ));
    }
    for (idx, step) in sequence.iter().enumerate() {
        if step.slot == crate::RESERVED_SLOT {
            return Err(err("slot", "slot reserved for runtime mouse-touch"));
        }
        if matches!(step.action, crate::types::MacroAction::Down) && step.pos.is_none() {
            return Err(err(
                &field(id, &format!("sequence[{idx}].pos")),
                "down needs pos",
            ));
        }
        if let Some(at) = step.pos {
            pos_ok(at, &field(id, &format!("sequence[{idx}].pos")))?;
        }
    }
    Ok(())
}
