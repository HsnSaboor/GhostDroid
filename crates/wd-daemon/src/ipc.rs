//! IPC socket path plus newline-delimited JSON frame helpers.
//!
//! Logs stderr only so MCP stdio stays clean.
//! Ref: `.devdocs/phantom/phantom/src/ipc.rs`, `docs/IPC.md`.

/// Daemon socket path.
#[must_use]
pub fn socket_path() -> String {
    tracing::debug!("ipc: socket path requested");
    "/run/wd/daemon.sock".to_owned()
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
}
