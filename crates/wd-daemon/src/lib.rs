//! wd-daemon: supervisor plus IPC. One method = one `wd-waydroid` fn.
//!
//! NDJSON serve on `socket_path()` + dispatch. No D-Bus. No drift.
//! Refs: `.devdocs/phantom/phantom/src/ipc.rs`, `.plans/02-core-daemon.md`.
#![deny(missing_docs)]

pub mod dispatch;
pub mod ipc;
pub mod serve;
pub mod supervisor;

pub use dispatch::dispatch;
pub use ipc::{
    decode_req_line, encode_req, encode_resp, legacy_socket_path, socket_path, socket_paths,
};
pub use serve::{CONN_READ_TIMEOUT, CONN_WRITE_TIMEOUT, serve};
pub use supervisor::{
    CHILD_TIMEOUT_MS, Child, child_timeout_ms, is_frozen_status, supervised_children,
};
