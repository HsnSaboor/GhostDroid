//! Aim checks.
#![deny(missing_docs)]

use wd_core::Result;

use super::{err, field};
use crate::adv::AimNode;

/// Aim shape.
pub(super) fn check(id: &str, node: &AimNode) -> Result<()> {
    super::shape::pos_ok(node.anchor, &field(id, "anchor"))?;
    if node.reach <= 0.0 || node.reach > 0.45 {
        return Err(err(&field(id, "reach"), "reach must be in (0, 0.45]"));
    }
    if node.sensitivity <= 0.0 {
        return Err(err(
            &field(id, "sensitivity"),
            "sensitivity must be positive",
        ));
    }
    if let Some(d) = node.deadzone {
        let d = f64::from(d);
        if !(0.0..1.0).contains(&d) {
            return Err(err(&field(id, "deadzone"), "deadzone must be in [0, 1)"));
        }
    }
    if let Some(rect) = node.region {
        if rect.w <= 0.0 || rect.h <= 0.0 {
            return Err(err(&field(id, "region"), "region dims must be positive"));
        }
        if rect.x < 0.0 || rect.y < 0.0 || rect.x + rect.w > 1.0 || rect.y + rect.h > 1.0 {
            return Err(err(&field(id, "region"), "region outside [0,1]"));
        }
    }
    match node.activation {
        crate::types::Activation::AlwaysOn => {
            if node.activation_key.is_some() {
                return Err(err(&field(id, "activation_key"), "omit key when always_on"));
            }
        }
        crate::types::Activation::WhileHeld | crate::types::Activation::Toggle => {
            let Some(name) = node.activation_key.as_deref() else {
                return Err(err(&field(id, "activation_key"), "key required"));
            };
            super::shape::key_ok(name, &field(id, "activation_key"))?;
        }
    }
    Ok(())
}
