//! wd-core: shared types only. No logic, no Waydroid calls.
//! Refs: `.research/00-overview-decision.md`, `.devdocs/phantom/phantom/src/config.rs`.
//! Single source: split modules below. No dup definitions here (DRY).

pub mod cfg;
pub mod err;
pub mod geom;
pub mod ids;
pub mod rpc;
pub mod tracing_init;

pub use cfg::WdConfig;
pub use err::{Result, WdError};
pub use geom::RelPos;
pub use ids::{Pkg, Serial};
pub use rpc::{Req, Resp};
pub use tracing_init::init_tracing;

/// Crate version year marker: 2026 stack. Note: `edition = "2024"` in Cargo.toml
/// is the Rust language edition, NOT a year stamp. Keep edition 2024.
pub const STACK_YEAR: u32 = 2026;

/// Init tracing once. Logs at everything for fast debug/perf triage.
pub fn init_logging() {
    let _ = init_tracing("info");
    tracing::info!(year = STACK_YEAR, "wd-core: logging init");
}
