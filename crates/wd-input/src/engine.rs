//! Keymap runtime engine: aim curves + stick coords + grab state.
//!
//! Pure math only (no IO). `wd-inject` turns outputs into frames.
#![deny(missing_docs)]

pub mod aim;
pub mod grab;
pub mod stick;

pub use aim::{apply_curve, apply_deadzone, clamp_reach, effective_deadzone, shape_delta};
pub use grab::{GrabState, grab_edge, grab_rising_edge};
pub use stick::{StickInput, dir_vector, stick_pos};
