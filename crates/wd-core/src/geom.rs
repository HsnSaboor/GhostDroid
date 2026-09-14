//! Relative geometry per `QtScrcpy` rule.

use serde::{Deserialize, Serialize};

/// Relative position, axes in range.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RelPos {
    /// `x` fraction.
    pub x: f32,
    /// `y` fraction.
    pub y: f32,
}

impl RelPos {
    /// Create a validated relative position.
    ///
    /// # Errors
    /// Returns [`crate::err::WdError::Validation`] when out of range.
    pub fn new(x: f32, y: f32) -> Result<Self, crate::err::WdError> {
        if !(0.0..=1.0).contains(&x) || !(0.0..=1.0).contains(&y) {
            return Err(crate::err::WdError::Validation(format!(
                "RelPos out of range: ({x}, {y})"
            )));
        }
        Ok(Self { x, y })
    }

    /// Range check (non-failing).
    #[must_use]
    pub const fn valid(self) -> bool {
        self.x >= 0.0 && self.x <= 1.0 && self.y >= 0.0 && self.y <= 1.0
    }

    /// To pixels with clamp.
    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn to_pixels(self, w: u32, h: u32) -> (i32, i32) {
        let wi = w.cast_signed();
        let hi = h.cast_signed();
        let x = (f64::from(self.x.clamp(0.0, 1.0)) * f64::from(wi)).round() as i32;
        let y = (f64::from(self.y.clamp(0.0, 1.0)) * f64::from(hi)).round() as i32;
        (x.min(wi - 1).max(0), y.min(hi - 1).max(0))
    }
}
