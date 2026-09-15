//! wd-waydroid: session/props/apps over stock Waydroid 1.6.3.
//!
//! Arg builders plus parsers only. No spawn, no sleep, no D-Bus.
//! Refs: `.devdocs/waydroid/tools/actions/`,
//! `.devdocs/waydroid-mcp/src/waydroid_mcp/adb.py:39-51`.
#![deny(missing_docs)]

pub mod android_server;
pub mod apps;
pub mod exec;
pub mod props;
pub mod ready;
pub mod session;

pub use android_server::{
    SERVER_BIND_HOST, SERVER_CLASS, SERVER_JAR_DEVICE_PATH, SERVER_LOG_DEVICE_PATH, is_server_up,
    launch_args as server_launch_args, log_args as server_log_args, push_args as server_push_args,
};
pub use apps::{
    AppRow, install_args, intent_args, launch_args, list_args, parse_app_list, remove_args,
};
pub use exec::{is_app_verb, run_waydroid};
pub use props::{BASE_PROP, PROP_KEYS, get_args, is_managed, set_args};
pub use ready::{DEFAULT_TIMEOUT_S, POLL_GAP_MS, WAYLAND_SOCKET, WaitDecision, WaitReady, decide};
pub use session::{
    Status, boot_args, classify_timeout, freeze_args, parse_status, shutdown_args, unfreeze_args,
};
