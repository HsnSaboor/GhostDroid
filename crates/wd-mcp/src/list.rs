//! `tools/list` with cursor pagination + cache hints.
//!
//! Spec 2026-07-28: stateless; `cursor`, `ttlMs`, `cacheScope`.
//! Page size 12 so clients exercise `nextCursor`.

use crate::{LIST_CACHE_SCOPE, LIST_TTL_MS, tools};

/// One page of the tool registry.
const PAGE: usize = 12;

/// List tools at `cursor` (byte offset as string, opaque to clients).
#[must_use]
pub fn tools_list(cursor: Option<&str>) -> serde_json::Value {
    tracing::debug!(?cursor, "list: in");
    let start: usize = cursor.and_then(|c| c.parse().ok()).unwrap_or(0);
    let start = start.min(tools::TOOLS.len());
    let end = (start + PAGE).min(tools::TOOLS.len());
    let page: Vec<serde_json::Value> = tools::TOOLS[start..end]
        .iter()
        .map(|t| tools::schema(t))
        .collect();
    let mut out = serde_json::json!({
        "resultType": "complete",
        "tools": page,
        "ttlMs": LIST_TTL_MS,
        "cacheScope": LIST_CACHE_SCOPE,
    });
    if end < tools::TOOLS.len() {
        out["nextCursor"] = serde_json::Value::String(end.to_string());
    }
    tracing::info!(start, end, bytes = out.to_string().len(), "list: out");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_page_has_cursor() {
        let v = tools_list(None);
        assert_eq!(v["ttlMs"], 60_000);
        assert_eq!(v["cacheScope"], "server");
        assert!(v.get("nextCursor").is_some());
    }

    #[test]
    fn last_page_no_cursor() {
        let v = tools_list(Some("12"));
        assert_eq!(v["tools"].as_array().unwrap().len(), 12);
        assert!(v.get("nextCursor").is_none());
    }
}
