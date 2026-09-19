//! Aim checks.
#![deny(missing_docs)]

use wd_core::Result;

use super::{err, field};

/// Aim shape.
pub(super) fn check(
    id: &str,
    anchor: wd_core::RelPos,
    reach: f64,
    sensitivity: f64,
    activation: crate::types::Activation,
    activation_key: Option<&str>,
    region: Option<crate::types::Region>,
    deadzone: Option<f32>,
) -> Result<()> {
    super::shape::pos_ok(anchor, &field(id, "anchor"))?;
    if reach <= 0.0 || reach > 0.45 {
        return Err(err(&field(id, "reach"), "reach must be in (0, 0.45]"));
    }
    if sensitivity <= 0.0 {
        return Err(err(
            &field(id, "sensitivity"),
            "sensitivity must be positive",
        ));
    }
    if let Some(d) = deadzone {
        let d = f64::from(d);
        if !(0.0..1.0).contains(&d) {
            return Err(err(&field(id, "deadzone"), "deadzone must be in [0, 1)"));
        }
    }
    if let Some(rect) = region {
        if rect.w <= 0.0 || rect.h <= 0.0 {
            return Err(err(&field(id, "region"), "region dims must be positive"));
        }
        if rect.x < 0.0 || rect.y < 0.0 || rect.x + rect.w > 1.0 || rect.y + rect.h > 1.0 {
            return Err(err(&field(id, "region"), "region outside [0,1]"));
        }
    }
    match activation {
        crate::types::Activation::AlwaysOn => {
            if activation_key.is_some() {
                return Err(err(&field(id, "activation_key"), "omit key when always_on"));
            }
        }
        crate::types::Activation::WhileHeld | crate::types::Activation::Toggle => {
            let Some(name) = activation_key else {
                return Err(err(&field(id, "activation_key"), "key required"));
            };
            super::shape::key_ok(name, &field(id, "activation_key"))?;
        }
    }
    Ok(())
}
