//! wd-input: canonical keymap schema + validate + audit + import.
//! Refs: `.devdocs/phantom/phantom/src/profile.rs:13-22`,
//! `.devdocs/QtScrcpy/docs/KeyMapDes.md`, `.research/03-tech-stack.md:72-87`.
#![deny(missing_docs)]

/// Profile version. Must == 1 else reject.
pub const PROFILE_VERSION: u32 = 1;
/// Max concurrent touches (`touch.rs:MAX_CONCURRENT_TOUCHES=10`).
pub const MAX_TOUCHES: usize = 10;
/// Reserved runtime slot (`mouse_touch.rs:RUNTIME_MOUSE_TOUCH_SLOT`).
pub const RESERVED_SLOT: u8 = u8::MAX;

pub mod access;
pub mod adv;
pub mod audit;
pub mod basic;
pub mod bindings;
pub mod checks;
pub mod import;
pub mod keys;
pub mod nodes;
pub mod qt;
pub mod qt_nodes;
pub mod schema;
pub mod types;
pub mod validate;
pub mod validate_node;

pub use audit::{AuditReport, audit, audit_screen};
pub use checks::{dup_keys, list, slot_clashes};
pub use import::import_qt;
pub use nodes::{Node, default_anchor, default_reach, default_sens};
pub use schema::{Profile, Screen};
pub use types::{
    Activation, AimCurve, JoystickKeys, LayerMode, MacroAction, MacroMode, MacroStep, Region,
};
pub use validate::{load, normalized, validate};

/// Validate profile version.
#[must_use]
pub fn version_ok(v: u32) -> bool {
    tracing::debug!(v, "wd-input: version check");
    v == PROFILE_VERSION
}
