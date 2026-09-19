//! Joystick math: WASD booleans → 8-dir circular coords.
#![deny(missing_docs)]

use wd_core::RelPos;

/// Pressed WASD set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "WASD is four directional bools by nature"
)]
pub struct StickInput {
    /// Up held.
    pub up: bool,
    /// Down held.
    pub down: bool,
    /// Left held.
    pub left: bool,
    /// Right held.
    pub right: bool,
}

impl StickInput {
    /// Build from four booleans.
    #[must_use]
    #[allow(
        clippy::fn_params_excessive_bools,
        reason = "WASD is four directional bools by nature"
    )]
    pub const fn new(up: bool, down: bool, left: bool, right: bool) -> Self {
        Self {
            up,
            down,
            left,
            right,
        }
    }
}

/// 8-dir unit vector (diagonals normalized by 1/√2). Opposites cancel.
#[must_use]
pub fn dir_vector(input: StickInput) -> (f64, f64) {
    let x = f64::from(input.right) - f64::from(input.left);
    let y = f64::from(input.down) - f64::from(input.up);
    if x == 0.0 && y == 0.0 {
        return (0.0, 0.0);
    }
    // ponytail: stdlib hypot beats hand-rolled norm.
    let mag = x.hypot(y);
    tracing::trace!(x, y, "wd-input: stick dir");
    (x / mag, y / mag)
}

/// Stick touch point: center + dir * radius, clamped to [0,1].
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    reason = "rel 0-1 into f32 is intentional"
)]
pub fn stick_pos(center: RelPos, radius: f64, input: StickInput) -> RelPos {
    let (dx, dy) = dir_vector(input);
    let r = radius.clamp(0.0, 1.0);
    let pos = RelPos {
        x: dx.mul_add(r, f64::from(center.x)) as f32,
        y: dy.mul_add(r, f64::from(center.y)) as f32,
    };
    tracing::trace!(x = pos.x, y = pos.y, "wd-input: stick pos");
    RelPos {
        x: pos.x.clamp(0.0, 1.0),
        y: pos.y.clamp(0.0, 1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CENTER: RelPos = RelPos { x: 0.158, y: 0.789 };

    #[test]
    fn neutral_is_center() {
        assert_eq!(stick_pos(CENTER, 0.082, StickInput::default()), CENTER);
    }

    #[test]
    fn cardinals_hit_rim() {
        let p = stick_pos(CENTER, 0.082, StickInput::new(true, false, false, false));
        assert!((f64::from(p.y) - (0.789 - 0.082)).abs() < 1e-6);
        let p = stick_pos(CENTER, 0.082, StickInput::new(false, false, false, true));
        assert!((f64::from(p.x) - (0.158 + 0.082)).abs() < 1e-6);
    }

    #[test]
    fn diagonal_normalized() {
        let (x, y) = dir_vector(StickInput::new(true, false, false, true));
        let want = std::f64::consts::FRAC_1_SQRT_2;
        assert!((x - want).abs() < 1e-12 && (y + want).abs() < 1e-12);
        let p = stick_pos(CENTER, 0.082, StickInput::new(true, false, false, true));
        assert!((f64::from(p.x).hypot(f64::from(p.y)) > 0.0));
    }

    #[test]
    fn opposites_cancel() {
        let i = StickInput::new(true, true, true, true);
        assert_eq!(dir_vector(i), (0.0, 0.0));
    }
}
