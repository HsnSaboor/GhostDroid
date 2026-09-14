//! wd-waydroid: session/props/apps over stock Waydroid 1.6.3.
//!
//! Arg builders plus parsers only. No spawn, no sleep, no D-Bus.
//! Refs: `.devdocs/waydroid/tools/actions/`,
//! `.devdocs/waydroid-mcp/src/waydroid_mcp/adb.py:39-51`.
#![deny(missing_docs)]

pub mod apps;
pub mod props;
pub mod ready;
pub mod session;

pub use apps::{install_args, intent_args, launch_args, list_args, remove_args};
pub use props::{BASE_PROP, PROP_KEYS, get_args, is_managed, set_args};
pub use ready::{DEFAULT_TIMEOUT_S, POLL_GAP_MS, WAYLAND_SOCKET, WaitDecision, WaitReady, decide};
pub use session::{
    Status, boot_args, classify_timeout, freeze_args, parse_status, shutdown_args, unfreeze_args,
};
