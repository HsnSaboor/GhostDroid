//! NDJSON listen loop over [`crate::socket_path`]. No new protocol.
//!
//! One line = one `Req`, one line = one `Resp`, via existing
//! [`crate::dispatch`]. Concurrent like async: one thread per conn,
//! per-conn read/write timeouts so a wedged client cannot wedge the
//! daemon. Binds [`crate::socket_paths`] (primary `/tmp/ghostdroid.sock`
//! plus legacy `/run/wd/daemon.sock`); stale sockets removed before
//! bind; bind failure (CI has no `/run/wd`) logs and continues.

use std::io::{BufRead as _, BufReader, Write as _};
use std::os::unix::net::{UnixListener, UnixStream};
use std::time::Duration;

use crate::{decode_req_line, dispatch, encode_resp, socket_paths};

/// Per-conn read timeout: wedged writers get cut, not the daemon.
pub const CONN_READ_TIMEOUT: Duration = Duration::from_secs(30);
/// Per-conn write timeout.
pub const CONN_WRITE_TIMEOUT: Duration = Duration::from_secs(10);

/// Bind every [`socket_paths`] entry and serve forever. Threads per conn.
pub fn serve() {
    let paths = socket_paths();
    let mut bound = 0usize;
    for path in &paths {
        if let Some(parent) = std::path::Path::new(path).parent()
            && let Err(e) = std::fs::create_dir_all(parent)
        {
            tracing::warn!(path = %path, error = %e, "wd-daemon: no socket dir");
            continue;
        }
        let _ = std::fs::remove_file(path);
        match UnixListener::bind(path) {
            Ok(l) => {
                bound += 1;
                tracing::info!(path = %path, "wd-daemon: listen");
                std::thread::spawn(move || serve_listener(&l));
            }
            Err(e) => {
                tracing::warn!(path = %path, error = %e, "wd-daemon: no listen (bind failed)");
            }
        }
    }
    if bound == 0 {
        tracing::warn!("wd-daemon: no sockets bound (caller keeps seeded state)");
        return;
    }
    // Park forever; listener threads own the work.
    loop {
        std::thread::park();
    }
}

/// Accept loop for one bound socket.
fn serve_listener(listener: &UnixListener) {
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
/// Read/write timeouts bound wedged peers (timeout protection).
fn handle_conn(stream: UnixStream) {
    tracing::debug!("wd-daemon: conn in");
    if let Err(e) = stream.set_read_timeout(Some(CONN_READ_TIMEOUT)) {
        tracing::warn!(error = %e, "wd-daemon: read timeout set failed");
    }
    if let Err(e) = stream.set_write_timeout(Some(CONN_WRITE_TIMEOUT)) {
        tracing::warn!(error = %e, "wd-daemon: write timeout set failed");
    }
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
                tracing::warn!(error = %e, "wd-daemon: read failed/timeout");
                break;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        let resp = decode_req_line(&line)
            .map_or_else(|| wd_core::Resp::err("bad req line"), |req| dispatch(&req));
        let out = encode_resp(&resp);
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

    #[test]
    fn timeouts_are_bounded() {
        assert!(CONN_READ_TIMEOUT.as_secs() <= 60);
        assert!(CONN_WRITE_TIMEOUT.as_secs() <= 30);
    }
}
