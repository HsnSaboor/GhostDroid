//! `server/discover`: stateless hello + version negotiate.
//!
//! No `initialize` handshake (outdated). Every request carries
//! `_meta.protocolVersion`; server answers with the version it
//! will speak. Ref: `.devdocs/scrcpy-mcp/src/index.ts`.

use crate::{LIST_CACHE_SCOPE, LIST_TTL_MS, PROTOCOL_VERSION, SUPPORTED_VERSIONS};

/// Namespaced `_meta` keys per spec 2026-07-28 (`basic/index#meta`).
pub const META_PROTO: &str = "io.modelcontextprotocol/protocolVersion";
/// Namespaced server identity key (`server/discover#data-types`).
pub const META_SERVER_INFO: &str = "io.modelcontextprotocol/serverInfo";

/// Build the `server/discover` payload.
///
/// Shape per `specification/2026-07-28/server/discover`:
/// `resultType`, `supportedVersions`, `capabilities`,
/// `_meta[io.modelcontextprotocol/serverInfo]`, `ttlMs`, `cacheScope`.
#[must_use]
pub fn discover() -> serde_json::Value {
    tracing::info!(version = PROTOCOL_VERSION, "discover: hello");
    let out = serde_json::json!({
        "resultType": "complete",
        "supportedVersions": SUPPORTED_VERSIONS,
        "capabilities": {"tools": {}},
        "_meta": {META_SERVER_INFO: {"name": "wd-mcp", "version": "0.1.0"}},
        "ttlMs": LIST_TTL_MS,
        "cacheScope": LIST_CACHE_SCOPE,
    });
    tracing::debug!("discover: out bytes={}", out.to_string().len());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_matches_spec() {
        let d = discover();
        assert_eq!(d["resultType"], "complete");
        let vs = d["supportedVersions"].as_array().unwrap();
        assert!(vs.iter().any(|v| v == "2026-07-28"));
        assert_eq!(d["_meta"][META_SERVER_INFO]["name"], "wd-mcp");
    }
}
