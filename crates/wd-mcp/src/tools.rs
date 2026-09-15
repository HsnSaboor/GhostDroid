//! Tool registry: the ONE tool list CLI+MCP share.
//!
//! 24 names. YAGNI cut holds: 24 vs adb-mcp 90 vs scrcpy-mcp 39.
//! `prop.get/set`, `keymap.load`, `spoof.load` ride as `shell.exec`
//! / `file.push` params, not separate tools — one list, no drift.
//!
//! Refs: `.devdocs/adb-mcp/adb_mcp/tools/` (90-tool bloat we cut),
//! `.devdocs/scrcpy-mcp/src/tools/` (39), `us/src/config.ts`
//! (category filter idea).

/// Every tool name. CLI verbs map 1:1 here.
pub const TOOLS: &[&str] = &[
    "device.list",
    "device.boot",
    "device.status",
    "app.install",
    "app.uninstall",
    "app.start",
    "app.stop",
    "app.list",
    "activity.current",
    "input.tap",
    "input.swipe",
    "input.key",
    "input.text",
    "vision.screenshot",
    "vision.stream_start",
    "vision.stream_stop",
    "ui.dump",
    "ui.find",
    "shell.exec",
    "file.push",
    "file.pull",
    "logcat.dump",
    "logcat.start",
    "logcat.stop",
];

/// Number of registered tools.
#[must_use]
pub const fn tool_count() -> usize {
    TOOLS.len()
}

/// True when `name` is a known tool.
#[must_use]
pub fn has_tool(name: &str) -> bool {
    tracing::debug!(name, "tools: lookup");
    let hit = TOOLS.contains(&name);
    tracing::debug!(name, hit, "tools: lookup result");
    hit
}

/// Minimal JSON schema per tool (object params; full
/// zod-style shapes live in `call.rs` dispatch validation).
/// Spec `server/tools#data-types`: `name` + `description` + `inputSchema`.
#[must_use]
pub fn schema(name: &str) -> serde_json::Value {
    tracing::debug!(name, "tools: schema");
    serde_json::json!({"name": name, "description": name, "inputSchema": {"type": "object"}})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_covers_plan_core() {
        for t in [
            "device.list",
            "device.boot",
            "device.status",
            "ui.dump",
            "ui.find",
            "shell.exec",
            "vision.screenshot",
            "logcat.dump",
        ] {
            assert!(has_tool(t), "missing {t}");
        }
        assert_eq!(tool_count(), 24);
    }

    #[test]
    fn unknown_rejected() {
        assert!(!has_tool("nope.tool"));
    }
}
