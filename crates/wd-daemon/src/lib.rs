//! wd-daemon: supervisor plus IPC. One method = one `wd-waydroid` fn.
//!
//! NDJSON serve on `socket_path()` + dispatch. No D-Bus. No drift.
//! Refs: `.devdocs/phantom/phantom/src/ipc.rs`, `docs/IPC.md`.
#![deny(missing_docs)]

pub mod dispatch;
pub mod ipc;
pub mod serve;
pub mod supervisor;

pub use dispatch::dispatch;
pub use ipc::{decode_req_line, encode_req, socket_path};
pub use serve::serve;
pub use supervisor::{Child, supervised_children};
