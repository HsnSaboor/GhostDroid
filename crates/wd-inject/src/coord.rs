//! Coord scaling via `RelPos`. `round(rel*max)` clamp.
//! Ref `ANDROID_SOCKET_PROTOCOL.md` coord contract.
#![deny(missing_docs)]

use wd_core::RelPos;

/// Scale rel → pixels via shared `RelPos::to_pixels` (DRY).
#[must_use]
pub fn scale(pos: RelPos, w: u32, h: u32) -> (i32, i32) {
    tracing::trace!(x = pos.x, y = pos.y, w, h, "wd-inject: scale");
    pos.to_pixels(w, h)
}
