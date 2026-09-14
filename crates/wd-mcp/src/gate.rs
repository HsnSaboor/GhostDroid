//! Gates + shell escaping. No spawn — pure checks.
//!
//! Ports: `us/tools/utils.ts:shellEscape/validators`,
//! `adb-mcp/core.py:quote_argv`, 2-tier gate
//! `ANDROID_MCP_ALLOW_WRITE/SHELL`, `ADB_MCP_ALLOW_SHELL=1`.

/// Write gate (`ANDROID_MCP_ALLOW_WRITE=1/true`).
#[must_use]
pub fn write_allowed() -> bool {
    let ok = std::env::var("ANDROID_MCP_ALLOW_WRITE")
        .is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
    tracing::debug!(ok, "gate: write");
    ok
}

/// Shell gate: (`ANDROID_MCP_ALLOW_SHELL` or `ADB_MCP_ALLOW_SHELL=1`)
/// AND `ALLOW_SHELL` not `0`. Default deny.
#[must_use]
pub fn shell_allowed() -> bool {
    let tier1 = std::env::var("ANDROID_MCP_ALLOW_SHELL")
        .is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        || std::env::var("ADB_MCP_ALLOW_SHELL").is_ok_and(|v| v == "1");
    let kill = std::env::var("ALLOW_SHELL").is_ok_and(|v| v == "0");
    let ok = tier1 && !kill;
    tracing::info!(ok, "gate: shell");
    ok
}

/// Single-quote escape (`us shellEscape` port).
#[must_use]
pub fn shell_escape(s: &str) -> String {
    tracing::debug!(s, "gate: escape in");
    let out = format!("'{}'", s.replace('\'', "'\\''"));
    tracing::debug!(bytes = out.len(), "gate: escape out");
    out
}

/// Quote argv injection-safe (`adb-mcp quote_argv` port).
#[must_use]
pub fn quote_argv(args: &[&str]) -> String {
    tracing::debug!(?args, "gate: quote in");
    let out = args
        .iter()
        .map(|a| shell_escape(a))
        .collect::<Vec<_>>()
        .join(" ");
    tracing::debug!(bytes = out.len(), "gate: quote out");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_and_quote() {
        assert_eq!(shell_escape("a'b"), "'a'\\''b'");
        assert_eq!(quote_argv(&["a", "b c"]), "'a' 'b c'");
        assert!(!shell_allowed());
        assert!(!write_allowed());
    }
}
