//! JSON-RPC router: `server/discover`, `tools/list`, `tools/call`.
//!
//! Spec 2026-07-28: stateless, per-request `_meta`
//! (`io.modelcontextprotocol/protocolVersion`, bare
//! `protocolVersion` accepted as back-compat). Unknown method →
//! JSON-RPC `-32601`; bad version → `-32022` with `supported`.
//! Logs transport bytes in/out for debug/perf.

use crate::{
    SUPPORTED_VERSIONS,
    call::dispatch,
    discover::{META_PROTO, META_SERVER_INFO},
};

/// Extract requested protocol version: namespaced first, bare fallback.
fn requested_version(params: &serde_json::Value) -> &str {
    params
        .get("_meta")
        .and_then(|m| {
            m.get(META_PROTO)
                .or_else(|| m.get("protocolVersion"))
                .and_then(|v| v.as_str())
        })
        .unwrap_or(crate::PROTOCOL_VERSION)
}

/// Handle one JSON-RPC request value → response value.
///
/// Version negotiate per `basic/versioning`: unknown version →
/// `-32022` + `data{supported,requested}`; unknown method → `-32601`.
/// Success echoes namespaced `_meta` + `serverInfo`.
#[must_use]
pub fn handle_request(req: &serde_json::Value) -> serde_json::Value {
    let bytes = req.to_string().len();
    tracing::info!(bytes, "transport: request in");
    let id = req.get("id").cloned().unwrap_or(serde_json::Value::Null);
    let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let params = req
        .get("params")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let proto = requested_version(&params).to_owned();
    if !SUPPORTED_VERSIONS.contains(&proto.as_str()) {
        tracing::warn!(%proto, "transport: unsupported version");
        return serde_json::json!({"jsonrpc": "2.0", "id": id, "error": {
            "code": -32022, "message": "Unsupported protocol version",
            "data": {"supported": SUPPORTED_VERSIONS, "requested": proto}}});
    }
    let result = match method {
        "server/discover" => crate::discover(),
        "tools/list" => crate::tools_list(params.get("cursor").and_then(|c| c.as_str())),
        "tools/call" => {
            let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let args = params.get("arguments").unwrap_or(&serde_json::Value::Null);
            dispatch(name, args)
        }
        _ => {
            tracing::warn!(method, "transport: unknown method");
            return serde_json::json!({"jsonrpc": "2.0", "id": id,
                "error": {"code": -32601, "message": format!("Method not found: {method}")}});
        }
    };
    let out = serde_json::json!({"jsonrpc": "2.0", "id": id, "result": result,
        "_meta": {META_PROTO: proto, "protocolVersion": proto,
            META_SERVER_INFO: {"name": "wd-mcp", "version": "0.1.0"}}});
    tracing::info!(
        bytes = out.to_string().len(),
        method,
        "transport: response out"
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpc_roundtrip() {
        let req = serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "server/discover", "params": {}});
        let resp = handle_request(&req);
        assert!(
            resp["result"]["supportedVersions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v == "2026-07-28")
        );
        assert_eq!(resp["_meta"][META_PROTO], "2026-07-28");
    }

    #[test]
    fn unknown_method_is_error() {
        let req = serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "nope", "params": {}});
        assert_eq!(handle_request(&req)["error"]["code"], -32601);
    }

    #[test]
    fn bad_version_is_32022() {
        let req = serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "server/discover",
            "params": {"_meta": {"io.modelcontextprotocol/protocolVersion": "1900-01-01"}}});
        let resp = handle_request(&req);
        assert_eq!(resp["error"]["code"], -32022);
        assert!(
            resp["error"]["data"]["supported"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v == "2026-07-28")
        );
    }

    #[test]
    fn namespaced_meta_accepted() {
        let req = serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "server/discover",
            "params": {"_meta": {"io.modelcontextprotocol/protocolVersion": "2026-03-26"}}});
        let resp = handle_request(&req);
        assert_eq!(resp["_meta"][META_PROTO], "2026-03-26");
    }
}
