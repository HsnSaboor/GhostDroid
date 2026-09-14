//! wd-inject: socket frame builders + coord scaling. No socket yet (YAGNI).
//! Refs: `.devdocs/phantom/phantom/docs/ANDROID_SOCKET_PROTOCOL.md:139`,
//! `.devdocs/phantom/phantom/src/android_inject.rs`, `.research/03-tech-stack.md:85`.
#![deny(missing_docs)]

pub mod coord;
pub mod frames;

pub use coord::scale;
pub use frames::{cancel_frame, down_frame, move_frame, ping_frame, up_frame};

/// Server port. Ref `waydroid.rs:27183`.
pub const SERVER_PORT: u16 = 27183;

/// Frame tags LE.
pub const TAG_DOWN: u8 = 0x00;
/// Move tag.
pub const TAG_MOVE: u8 = 0x01;
/// Up tag.
pub const TAG_UP: u8 = 0x02;
/// Cancel tag.
pub const TAG_CANCEL: u8 = 0x03;
/// Ping tag.
pub const TAG_PING: u8 = 0x7f;
