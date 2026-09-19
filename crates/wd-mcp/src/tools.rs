//! Tool registry: the ONE tool list CLI+MCP share.
//!
//! 24 names. YAGNI cut holds: 24 vs adb-mcp 90 vs scrcpy-mcp 39.
//! `vision.stream_*` / `logcat.start|stop` stay out: caller-spawned
//! stubs with no live impl. `prop.get/set`, `keymap.load`,
//! `spoof.load` are live tools with impls in `call.rs`.
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
    "ui.dump",
    "ui.find",
    "shell.exec",
    "file.push",
    "file.pull",
    "logcat.dump",
    "prop.get",
    "prop.set",
    "keymap.load",
    "spoof.load",
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
            "prop.get",
            "prop.set",
            "keymap.load",
            "spoof.load",
        ] {
            assert!(has_tool(t), "missing {t}");
        }
        for t in [
            "vision.stream_start",
            "vision.stream_stop",
            "logcat.start",
            "logcat.stop",
        ] {
            assert!(!has_tool(t), "stub {t} must stay out");
        }
        assert_eq!(tool_count(), 24);
    }

    #[test]
    fn unknown_rejected() {
        assert!(!has_tool("nope.tool"));
    }
}
