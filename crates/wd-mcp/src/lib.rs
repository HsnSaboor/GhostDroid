//! wd-mcp: ONE tool core. CLI+MCP share every fn. No replay drift.
//!
//! MCP spec 2026-07-28: stateless, `server/discover`, `tools/list`
//! (cursor + `resultType` + `ttlMs` + `cacheScope`), `tools/call`. Every
//! request carries `_meta.io.modelcontextprotocol/protocolVersion`.
//! Transports: stdio newline JSON-RPC (logs stderr only); streamable
//! HTTP opt-in, single POST `/mcp` with `MCP-Protocol-Version` +
//! `Mcp-Method` / `Mcp-Name` headers (mismatch → `-32020`).
//!
//! Refs: `.plans/08-cli-mcp-skills.md`,
//! `.devdocs/scrcpy-mcp/src/index.ts:StdioServerTransport`,
//! `.devdocs/waydroid-mcp/src/waydroid_mcp/{core.py:43-95,cli.py}`,
//! `.devdocs/adb-mcp/adb_mcp/{core.py,tools/{media,logs,ui}.py}`,
//! `.devdocs/android-mcp-server-us/src/{index.ts,tools/{ui,logcat,utils}.ts}`.
//!
//! Note: `edition = "2024"` in `Cargo.toml` is the Rust language
//! edition, NOT a year stamp. Stack year rides `wd_core::STACK_YEAR`.
#![deny(missing_docs)]

pub mod call;
pub mod discover;
pub mod gate;
pub mod list;
pub mod logcat;
pub mod screenshot;
pub mod serial;
pub mod tools;
pub mod transport;
pub mod ui_dump;

pub use call::{dispatch, exit_code_for, keymap_load, spoof_load};
pub use discover::discover;
pub use gate::{quote_argv, shell_allowed, shell_escape, write_allowed};
pub use list::tools_list;
pub use tools::{TOOLS, has_tool, schema, tool_count};
pub use transport::handle_request;

/// Init tracing with stderr writer. MUST be used by bins, never
/// stdout (breaks stdio JSON-RPC framing — spec: logs stderr only).
pub fn init_stderr_logging() {
    use tracing_subscriber::{EnvFilter, fmt};
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned());
    let _ = fmt()
        .with_env_filter(EnvFilter::new(filter))
        .with_writer(std::io::stderr)
        .try_init();
    tracing::info!(year = wd_core::STACK_YEAR, "wd-mcp: logging init (stderr)");
}
/// Protocol version we speak.
pub const PROTOCOL_VERSION: &str = "2026-07-28";
/// Versions advertised by `server/discover`.
pub const SUPPORTED_VERSIONS: &[&str] = &["2026-07-28", "2026-03-26", "2025-11-25"];
/// `tools/list` cache hint, ms.
pub const LIST_TTL_MS: u64 = 60_000;
/// `tools/list` cache scope.
pub const LIST_CACHE_SCOPE: &str = "server";
/// CLI/MCP mismatch exit (waydroid-mcp compat).
pub const EXIT_MISMATCH: i32 = 3;
/// CLI/MCP device-error exit (waydroid-mcp compat).
pub const EXIT_DEVICE: i32 = 4;
