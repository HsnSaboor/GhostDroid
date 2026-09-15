//! NDJSON listen loop over [`crate::socket_path`]. No new protocol.
//!
//! One line = one `Req`, one line = one `Resp`, via existing
//! [`crate::dispatch`]. Stale socket removed before bind; bind
//! failure (CI has no `/run/wd`) logs and returns (caller keeps seeded state).

use std::io::{BufRead as _, BufReader, Write as _};
use std::os::unix::net::{UnixListener, UnixStream};

use crate::{decode_req_line, dispatch, socket_path};

/// Bind [`socket_path`] and serve forever. Threads per conn.
pub fn serve() {
    let path = socket_path();
    let _ = std::fs::remove_file(&path);
    let listener = match UnixListener::bind(&path) {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!(path = %path, error = %e, "wd-daemon: no listen (bind failed)");
            return;
        }
    };
    tracing::info!(path = %path, "wd-daemon: listen");
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                std::thread::spawn(move || handle_conn(s));
            }
            Err(e) => tracing::warn!(error = %e, "wd-daemon: accept failed"),
        }
    }
}

/// Serve one conn: each non-empty line → [`dispatch`] → one JSON line.
fn handle_conn(stream: UnixStream) {
    tracing::debug!("wd-daemon: conn in");
    let mut writer = match stream.try_clone() {
        Ok(w) => w,
        Err(e) => {
            tracing::warn!(error = %e, "wd-daemon: clone failed");
            return;
        }
    };
    for line in BufReader::new(stream).lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                tracing::warn!(error = %e, "wd-daemon: read failed");
                break;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        let resp = decode_req_line(&line)
            .map_or_else(|| wd_core::Resp::err("bad req line"), |req| dispatch(&req));
        let mut out = serde_json::to_string(&resp)
            .unwrap_or_else(|_| r#"{"ok":false,"data":"encode failed"}"#.to_owned());
        out.push('\n');
        if writer.write_all(out.as_bytes()).is_err() {
            break;
        }
    }
    tracing::debug!("wd-daemon: conn out");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conn_roundtrip() {
        let (mut client, server) = UnixStream::pair().unwrap();
        let handle = std::thread::spawn(move || handle_conn(server));
        client
            .write_all(b"{\"method\":\"device.list\",\"params\":null}\nbad line\n")
            .unwrap();
        let mut reader = BufReader::new(client.try_clone().unwrap());
        let mut first = String::new();
        reader.read_line(&mut first).unwrap();
        let resp: wd_core::Resp = serde_json::from_str(&first).unwrap();
        assert!(resp.ok);
        drop(reader);
        drop(client);
        handle.join().unwrap();
    }
}
