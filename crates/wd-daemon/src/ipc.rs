//! IPC socket path plus newline-delimited JSON frame helpers.
//!
//! Logs stderr only so MCP stdio stays clean.
//! Primary socket is `/tmp/ghostdroid.sock` (`WD_DAEMON_SOCK` /
//! `GHOSTDROID_SOCK` override); `/run/wd/daemon.sock` stays as a
//! legacy alias the server also binds when it differs.
//! Ref: `.devdocs/phantom/phantom/src/ipc.rs`, `.plans/02-core-daemon.md`.

/// Primary daemon socket path: `/tmp/ghostdroid.sock` unless overridden.
#[must_use]
pub fn socket_path() -> String {
    tracing::debug!("ipc: socket path requested");
    std::env::var("WD_DAEMON_SOCK")
        .or_else(|_| std::env::var("GHOSTDROID_SOCK"))
        .map(|v| v.trim().to_owned())
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "/tmp/ghostdroid.sock".to_owned())
}

/// Legacy alias kept for older clients (`/run/wd/daemon.sock`).
#[must_use]
pub fn legacy_socket_path() -> String {
    "/run/wd/daemon.sock".to_owned()
}

/// All socket paths [`crate::serve`] should bind (deduped).
#[must_use]
pub fn socket_paths() -> Vec<String> {
    let primary = socket_path();
    let legacy = legacy_socket_path();
    if primary == legacy {
        vec![primary]
    } else {
        vec![primary, legacy]
    }
}

/// Encode a `Req` as one newline-delimited JSON line.
#[must_use]
pub fn encode_req(req: &wd_core::Req) -> String {
    tracing::debug!(method = %req.method, "ipc: encode req");
    let mut line = serde_json::to_string(req).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "ipc: req encode failed");
        String::from("{\"method\":\"\",\"params\":null}")
    });
    line.push('\n');
    line
}

/// Encode a `Resp` as one newline-delimited JSON line (serve path, DRY).
#[must_use]
pub fn encode_resp(resp: &wd_core::Resp) -> String {
    tracing::debug!(ok = resp.ok, "ipc: encode resp");
    let mut line = serde_json::to_string(resp)
        .unwrap_or_else(|_| r#"{"ok":false,"data":"encode failed"}"#.to_owned());
    line.push('\n');
    line
}

/// Decode one JSON line into a `Req`. Pure parse, logs shape only.
#[must_use]
pub fn decode_req_line(line: &str) -> Option<wd_core::Req> {
    tracing::debug!(len = line.len(), "ipc: decode req line");
    match serde_json::from_str(line.trim()) {
        Ok(req) => Some(req),
        Err(e) => {
            tracing::warn!(error = %e, "ipc: bad req line");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_line() {
        let req = wd_core::Req {
            method: "device.status".to_owned(),
            params: serde_json::Value::Null,
        };
        let line = encode_req(&req);
        assert!(line.ends_with('\n'));
        let back = decode_req_line(&line);
        assert!(back.is_some());
        assert_eq!(back.map_or_default(|r| r.method), "device.status");
        assert!(decode_req_line("not json").is_none());
        assert!(!socket_path().is_empty());
    }

    #[test]
    fn primary_is_ghostdroid_sock() {
        assert_eq!(socket_path(), "/tmp/ghostdroid.sock");
        assert_eq!(legacy_socket_path(), "/run/wd/daemon.sock");
        assert_eq!(socket_paths().len(), 2);
        let resp = wd_core::Resp::ok(serde_json::Value::Null);
        assert!(encode_resp(&resp).ends_with('\n'));
    }
}
