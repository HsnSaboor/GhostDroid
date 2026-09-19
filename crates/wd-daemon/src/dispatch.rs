//! `Req` -> `Resp` dispatch. Pure delegation to `wd-mcp::call`.
//!
//! Live-first: every registered tool delegates to the ONE shared
//! `wd_mcp::dispatch` (10s-class timeouts live in each arm), so CLI +
//! MCP + daemon never drift. `device.status` keeps its probe path:
//! `params.status_output` wins (tests/sync feed it); otherwise it
//! delegates like everything else. Frozen detection runs on every
//! status-shaped output via `parse_status`. No stub ever masks a
//! real device: missing binaries fall back to argv preview inside
//! `wd-mcp`, never to a fake RUNNING device.
//! `Req`/`Resp` shape from `wd-core::rpc` stays MCP-compatible.
//! Ref: `.plans/02-core-daemon.md`.

use wd_core::{Req, Resp};
use wd_waydroid::parse_status;

/// Short probe timeout, ms (status-probe path only; live arms own
/// their timeouts inside `wd-mcp::call`).
pub const DISPATCH_TIMEOUT_MS: u64 = 10_000;

/// Dispatch one request. Logs method in, frozen, and out.
#[must_use]
pub fn dispatch(req: &Req) -> Resp {
    tracing::info!(method = %req.method, params = %req.params, "dispatch: in");
    let resp = if has_probe(req) {
        device_status_probe(req)
    } else if wd_mcp::has_tool(&req.method) {
        delegate(req)
    } else {
        tracing::warn!(method = %req.method, "dispatch: unknown method");
        Resp::err("unknown method")
    };
    if frozen_in(&resp) {
        tracing::warn!(method = %req.method, "dispatch: frozen detected");
    }
    tracing::info!(method = %req.method, ok = resp.ok, "dispatch: out");
    resp
}

/// True when `device.status` carries an inline probe output.
fn has_probe(req: &Req) -> bool {
    req.method == "device.status"
        && req
            .params
            .get("status_output")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|s| !s.is_empty())
}

/// `device.status` probe path: parse inline `status_output`, no spawn.
fn device_status_probe(req: &Req) -> Resp {
    let probe = req
        .params
        .get("status_output")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    tracing::debug!(len = probe.len(), "dispatch: status probe parse");
    let parsed = parse_status(probe);
    Resp::ok(serde_json::json!({
        "session": parsed.session,
        "container": parsed.container,
        "frozen": parsed.frozen,
        "ip": parsed.ip,
    }))
}

/// Delegate one request to the shared `wd-mcp` dispatch and map the
/// MCP-style value (`{"ok": bool, ...}`) to `Resp`.
fn delegate(req: &Req) -> Resp {
    tracing::debug!(method = %req.method, "dispatch: delegate to wd-mcp");
    let out = wd_mcp::dispatch(&req.method, &req.params);
    let ok = out.get("ok") == Some(&serde_json::Value::Bool(true));
    Resp { ok, data: out }
}

/// Frozen flag surfaced anywhere in the delegated payload.
fn frozen_in(resp: &Resp) -> bool {
    resp.data.get("frozen") == Some(&serde_json::Value::Bool(true))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(method: &str) -> Req {
        Req {
            method: method.to_owned(),
            params: serde_json::Value::Null,
        }
    }

    #[test]
    fn core_methods_ok() {
        assert!(dispatch(&req("device.status")).ok);
        assert!(dispatch(&req("device.list")).ok);
        assert!(dispatch(&req("app.list")).ok);
        assert!(!dispatch(&req("nope")).ok);
    }

    #[test]
    fn delegates_all_registered_tools() {
        // Every registered tool resolves (ok or param-gated isError),
        // never "unknown method" — pure delegation, no 3-method stub.
        for tool in wd_mcp::TOOLS {
            let resp = dispatch(&req(tool));
            assert!(
                resp.data != serde_json::Value::String("unknown method".to_owned()),
                "{tool} must delegate, not 404"
            );
        }
        // Param-gated arms surface their need-error through Resp.
        let mut r = req("app.start");
        assert!(!dispatch(&r).ok);
        r = req("prop.get");
        assert!(!dispatch(&r).ok);
        // Removed stubs stay unknown.
        assert!(!dispatch(&req("vision.stream_start")).ok);
    }

    #[test]
    fn frozen_probe_surfaces() {
        let mut r = req("device.status");
        r.params = serde_json::json!({"status_output": "Container: FROZEN"});
        let resp = dispatch(&r);
        assert_eq!(resp.data["frozen"], serde_json::Value::Bool(true));
    }
}
