//! `Req` -> `Resp` dispatch. One method = one `wd-waydroid` fn.
//!
//! Stub data only, no spawn. `Req`/`Resp` shape from `wd-core::rpc`
//! stays MCP-compatible. Ref: `.devdocs/phantom/phantom/src/ipc.rs`.

use wd_core::{Req, Resp};
use wd_waydroid::parse_status;

/// Dispatch one request. Logs method in, parse, frozen, and out.
#[must_use]
pub fn dispatch(req: &Req) -> Resp {
    tracing::info!(method = %req.method, params = %req.params, "dispatch: in");
    let resp = match req.method.as_str() {
        "device.status" => device_status(req),
        "device.list" => device_list(),
        "app.list" => app_list(),
        other => {
            tracing::warn!(method = %other, "dispatch: unknown method");
            Resp::err("unknown method")
        }
    };
    tracing::info!(method = %req.method, ok = resp.ok, "dispatch: out");
    resp
}

/// `device.status`: parse optional `params.status_output` probe.
fn device_status(req: &Req) -> Resp {
    let probe = req
        .params
        .get("status_output")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    tracing::debug!(len = probe.len(), "dispatch: status probe parse");
    let parsed = parse_status(probe);
    if parsed.frozen {
        tracing::warn!("dispatch: frozen detected in status probe");
    }
    tracing::debug!(
        session = parsed.session,
        container = parsed.container,
        frozen = parsed.frozen,
        "dispatch: device.status stub"
    );
    Resp::ok(serde_json::json!({
        "session": parsed.session,
        "container": parsed.container,
        "frozen": parsed.frozen,
    }))
}

/// `device.list`: single stub entry proving the shape.
fn device_list() -> Resp {
    tracing::debug!("dispatch: device.list stub");
    Resp::ok(serde_json::json!({
        "devices": [{"serial": "waydroid-0", "state": "STOPPED"}],
    }))
}

/// `app.list`: empty stub list proving the shape.
fn app_list() -> Resp {
    tracing::debug!("dispatch: app.list stub");
    Resp::ok(serde_json::json!({ "apps": [] }))
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
    fn three_methods_ok() {
        assert!(dispatch(&req("device.status")).ok);
        assert!(dispatch(&req("device.list")).ok);
        assert!(dispatch(&req("app.list")).ok);
        assert!(!dispatch(&req("nope")).ok);
    }

    #[test]
    fn frozen_probe_surfaces() {
        let mut r = req("device.status");
        r.params = serde_json::json!({"status_output": "Container: FROZEN"});
        let resp = dispatch(&r);
        assert_eq!(resp.data["frozen"], serde_json::Value::Bool(true));
    }
}
