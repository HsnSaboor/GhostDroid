//! wd-daemon: supervisor plus IPC. One method = one `wd-waydroid` fn.
//!
//! Stub dispatch only, no spawn, no D-Bus. No drift.
//! Refs: `.devdocs/phantom/phantom/src/ipc.rs`, `docs/IPC.md`.
#![deny(missing_docs)]

pub mod dispatch;
pub mod ipc;
pub mod supervisor;

pub use dispatch::dispatch;
pub use ipc::{decode_req_line, encode_req, socket_path};
pub use supervisor::{Child, supervised_children};
