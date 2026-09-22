//! Aim math: sensitivity curves, deadzone, raw-delta shaping, reach clamp.
//!
//! All pure; magnitudes normalized 0-1, deltas in pixels.
#![deny(missing_docs)]

use crate::types::AimCurve;

/// Default deadzone (2% of magnitude).
pub const DEFAULT_DEADZONE: f64 = 0.02;

/// Resolve an optional per-node deadzone to an effective value.
///
/// `None` (missing in JSON) yields [`DEFAULT_DEADZONE`]; `Some` clamps into
/// `[0, 1)`. Keeps old profiles without `deadzone` loading identically.
#[must_use]
pub fn effective_deadzone(deadzone: Option<f32>) -> f64 {
    deadzone.map_or(DEFAULT_DEADZONE, |d| f64::from(d).clamp(0.0, 0.999))
}

/// Raw-delta → px scale.
pub const DELTA_SCALE: f64 = 0.001;

/// Apply response curve to normalized magnitude 0-1.
#[must_use]
pub fn apply_curve(mag: f64, curve: AimCurve) -> f64 {
    let m = mag.clamp(0.0, 1.0);
    let out = match curve {
        AimCurve::Linear => m,
        AimCurve::Precision => m * m,
        AimCurve::Balanced => m * m.sqrt(),
    };
    tracing::trace!(m, ?curve, out, "wd-input: aim curve");
    out
}

/// Deadzone filter: below `dead` → 0, else rescale to 0-1.
#[must_use]
pub fn apply_deadzone(mag: f64, dead: f64) -> f64 {
    if mag <= dead {
        return 0.0;
    }
    if dead <= 0.0 {
        return mag.clamp(0.0, 1.0);
    }
    let out = ((mag - dead) / (1.0 - dead)).clamp(0.0, 1.0);
    tracing::trace!(mag, dead, out, "wd-input: deadzone");
    out
}

/// Shape a raw mouse delta into a smooth offset.
///
/// Scales by `sens * global * DELTA_SCALE`, shapes magnitude through
/// curve + deadzone, preserves direction, optionally inverts Y.
#[must_use]
pub fn shape_delta(
    dx: f64,
    dy: f64,
    sens: f64,
    global: f64,
    curve: AimCurve,
    deadzone: f64,
    invert_y: bool,
) -> (f64, f64) {
    let gain = sens.max(0.0) * global.max(0.0) * DELTA_SCALE;
    let (sx, sy) = (dx * gain, if invert_y { -dy * gain } else { dy * gain });
    let mag = sx.hypot(sy);
    if mag <= f64::EPSILON {
        return (0.0, 0.0);
    }
    let shaped = apply_deadzone(apply_curve(mag.min(1.0), curve), deadzone) * mag.max(1.0);
    let k = shaped / mag;
    tracing::trace!(dx, dy, k, "wd-input: aim delta");
    (sx * k, sy * k)
}

/// Clamp an offset into the reach circle (rel units).
#[must_use]
pub fn clamp_reach(ox: f64, oy: f64, reach: f64) -> (f64, f64) {
    let mag = ox.hypot(oy);
    if mag <= reach || mag <= f64::EPSILON {
        return (ox, oy);
    }
    let k = reach / mag;
    (ox * k, oy * k)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curves_monotonic() {
        assert!(apply_curve(0.0, AimCurve::Balanced).abs() < 1e-12);
        assert!((apply_curve(1.0, AimCurve::Linear) - 1.0).abs() < 1e-12);
        assert!((apply_curve(1.0, AimCurve::Precision) - 1.0).abs() < 1e-12);
        assert!((apply_curve(1.0, AimCurve::Balanced) - 1.0).abs() < 1e-12);
        assert!(apply_curve(0.5, AimCurve::Precision) < apply_curve(0.5, AimCurve::Linear));
    }

    #[test]
    fn deadzone_filters() {
        assert!(apply_deadzone(0.01, 0.02).abs() < 1e-12);
        assert!(apply_deadzone(0.5, 0.02) > 0.0);
        assert!((apply_deadzone(0.5, 0.0) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn effective_deadzone_defaults_and_clamps() {
        assert!((effective_deadzone(None) - DEFAULT_DEADZONE).abs() < 1e-12);
        assert!((effective_deadzone(Some(0.05)) - 0.05).abs() < 1e-6);
        assert!(effective_deadzone(Some(-1.0)).abs() < 1e-12);
        assert!(effective_deadzone(Some(9.0)) < 1.0);
    }

    #[test]
    fn delta_zero_and_invert() {
        assert_eq!(
            shape_delta(0.0, 0.0, 1.0, 1.0, AimCurve::Linear, 0.0, false),
            (0.0, 0.0)
        );
        let (ax, ay) = shape_delta(100.0, 50.0, 1.0, 1.0, AimCurve::Linear, 0.0, false);
        let (bx, by) = shape_delta(100.0, 50.0, 1.0, 1.0, AimCurve::Linear, 0.0, true);
        assert!((ax - bx).abs() < f64::EPSILON);
        assert!((ay + by).abs() < f64::EPSILON);
    }

    #[test]
    fn reach_clamps() {
        assert_eq!(clamp_reach(0.05, 0.0, 0.18), (0.05, 0.0));
        let (x, y) = clamp_reach(1.0, 0.0, 0.18);
        assert!((x - 0.18).abs() < 1e-9 && y.abs() < 1e-9);
    }
}
