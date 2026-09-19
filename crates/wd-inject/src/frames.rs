//! Frame builders.
//!
//! LE per `ANDROID_SOCKET_PROTOCOL.md:139`.
//! `DOWN/MOVE [tag,slot,x LE,y LE]=10B`, `UP [tag,slot]=2B`,
//! `CANCEL [tag]=1B` (server `cancelAll`, no slot), `PING=1B`.
//! Matches `java/PhantomServer.java` + vendor
//! `contrib/android-server/.../PhantomServer.java` (`case CMD_TOUCH_CANCEL:
//! injector.cancelAll()` reads no payload byte).
#![deny(missing_docs)]

use crate::{TAG_CANCEL, TAG_DOWN, TAG_MOVE, TAG_PING, TAG_UP};

/// Build DOWN frame.
#[must_use]
pub fn down_frame(slot: u8, x: i32, y: i32) -> [u8; 10] {
    tracing::trace!(slot, x, y, "wd-inject: down frame");
    touch_frame(TAG_DOWN, slot, x, y)
}

/// Build MOVE frame.
#[must_use]
pub fn move_frame(slot: u8, x: i32, y: i32) -> [u8; 10] {
    tracing::trace!(slot, x, y, "wd-inject: move frame");
    touch_frame(TAG_MOVE, slot, x, y)
}

fn touch_frame(tag: u8, slot: u8, x: i32, y: i32) -> [u8; 10] {
    let mut b = [0u8; 10];
    b[0] = tag;
    b[1] = slot;
    b[2..6].copy_from_slice(&x.to_le_bytes());
    b[6..10].copy_from_slice(&y.to_le_bytes());
    b
}

/// Build UP frame.
#[must_use]
pub fn up_frame(slot: u8) -> [u8; 2] {
    tracing::trace!(slot, "wd-inject: up frame");
    [TAG_UP, slot]
}

/// Build CANCEL frame (no slot: server runs `cancelAll`).
#[must_use]
pub fn cancel_frame() -> [u8; 1] {
    tracing::trace!("wd-inject: cancel frame");
    [TAG_CANCEL]
}

/// Build PING frame.
#[must_use]
pub fn ping_frame() -> [u8; 1] {
    tracing::trace!("wd-inject: ping frame");
    [TAG_PING]
}
