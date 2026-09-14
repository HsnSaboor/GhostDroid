//! `wd-mcp` bin: stdio default, HTTP opt-in.
//!
//! stdio: newline JSON-RPC on stdin → `handle_request` → stdout;
//! logs stderr only (never stdout — breaks framing). HTTP opt-in:
//! `MCP_TRANSPORT=http` serves single POST `/mcp` per spec
//! 2026-07-28 streamable HTTP: `MCP-Protocol-Version` required,
//! `Mcp-Method` / `Mcp-Name` validated vs body (`-32020` mismatch),
//! unknown method → 404, bad version → 400, GET/DELETE → 405.
//! No new deps: std `TcpListener` only, one request per conn.

use std::io::{BufRead as _, Write as _};

/// Serve one HTTP POST /mcp connection (std only, weak-i5 safe).
fn serve_http_once(stream: &std::net::TcpStream) -> bool {
    use std::io::Read as _;
    let mut reader = std::io::BufReader::new(stream);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return false;
    }
    let req_method = request_line
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_uppercase();
    if req_method == "GET" || req_method == "DELETE" {
        tracing::warn!(%req_method, "wd-mcp: http method not allowed");
        let out =
            "HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        return write_raw(stream, out);
    }
    if req_method != "POST" && !req_method.is_empty() {
        tracing::warn!(%req_method, "wd-mcp: http bad method");
        let out = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        return write_raw(stream, out);
    }
    let mut head = String::new();
    let mut content_len = 0usize;
    let (mut method_hdr, mut name_hdr, mut ver_hdr) = (String::new(), String::new(), String::new());
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() || line == "\r\n" || line == "\n" || line.is_empty()
        {
            break;
        }
        tracing::debug!(bytes = line.len(), "wd-mcp: http header in");
        let low = line.to_ascii_lowercase();
        if let Some(v) = low.strip_prefix("content-length:") {
            content_len = v.trim().parse().unwrap_or(0);
        } else if let Some(v) = low.strip_prefix("mcp-method:") {
            v.trim().clone_into(&mut method_hdr);
        } else if let Some(v) = low.strip_prefix("mcp-name:") {
            v.trim().clone_into(&mut name_hdr);
        } else if let Some(v) = low.strip_prefix("mcp-protocol-version:") {
            v.trim().clone_into(&mut ver_hdr);
        }
        head.push_str(&line);
        if head.len() > 16_384 {
            break;
        }
    }
    let mut body = vec![0u8; content_len.min(1_048_576)];
    let _ = reader.read_exact(&mut body);
    tracing::info!(bytes = body.len(), "wd-mcp: http body in");
    let req: serde_json::Value = if body.is_empty() && !method_hdr.is_empty() {
        serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": method_hdr,
            "params": {"name": name_hdr, "arguments": {}}})
    } else {
        serde_json::from_slice(&body)
            .unwrap_or_else(|_| serde_json::json!({"jsonrpc": "2.0", "id": null}))
    };
    let resp_val = wd_mcp::handle_request(&req);
    if !ver_hdr.is_empty() {
        let body_method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let body_name = req
            .get("params")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("");
        let mismatch = (!method_hdr.is_empty() && method_hdr != body_method)
            || (!name_hdr.is_empty() && !body_name.is_empty() && name_hdr != body_name);
        if mismatch {
            tracing::warn!("wd-mcp: header/body mismatch");
            let err = serde_json::json!({"jsonrpc": "2.0", "id": req.get("id"),
                "error": {"code": -32020, "message": "HeaderMismatch"}});
            return write_json(stream, &err, 400);
        }
        let body_ver = req
            .get("params")
            .and_then(|p| p.get("_meta"))
            .and_then(|m| {
                m.get("io.modelcontextprotocol/protocolVersion")
                    .or_else(|| m.get("protocolVersion"))
            })
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if !body_ver.is_empty() && ver_hdr != body_ver {
            tracing::warn!("wd-mcp: version header mismatch");
            let err = serde_json::json!({"jsonrpc": "2.0", "id": req.get("id"),
                "error": {"code": -32020, "message": "HeaderMismatch"}});
            return write_json(stream, &err, 400);
        }
    }
    let status = status_for(&resp_val);
    write_json(stream, &resp_val, status)
}

/// Map JSON-RPC error code → HTTP status per spec 2026-07-28.
fn status_for(resp: &serde_json::Value) -> u16 {
    match resp
        .get("error")
        .and_then(|e| e.get("code"))
        .and_then(serde_json::Value::as_i64)
    {
        Some(-32601) => 404,
        Some(_) => 400,
        None => 200,
    }
}

/// Write one JSON value with HTTP status.
fn write_json(stream: &std::net::TcpStream, val: &serde_json::Value, status: u16) -> bool {
    let resp = val.to_string();
    tracing::info!(bytes = resp.len(), status, "wd-mcp: http body out");
    let reason = match status {
        400 => "Bad Request",
        404 => "Not Found",
        _ => "OK",
    };
    let out = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{resp}",
        resp.len()
    );
    write_raw(stream, &out)
}

/// Write a raw HTTP response string.
fn write_raw(stream: &std::net::TcpStream, out: &str) -> bool {
    stream.set_nonblocking(false).ok();
    let mut w = stream
        .try_clone()
        .unwrap_or_else(|_| stream.try_clone().unwrap());
    write!(w, "{out}").map(|()| true).unwrap_or(false)
}

/// stdio loop: stdin lines → dispatch → stdout lines.
fn stdio_loop() {
    tracing::info!("wd-mcp: stdio loop start");
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                tracing::warn!(%e, "wd-mcp: stdin read fail");
                continue;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        tracing::info!(bytes = line.len(), "wd-mcp: stdin line in");
        let req: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(%e, "wd-mcp: bad json line");
                let _ = writeln!(
                    out,
                    "{{\"jsonrpc\":\"2.0\",\"id\":null,\"result\":{{\"ok\":false,\"error\":\"bad json\"}}}}"
                );
                continue;
            }
        };
        let resp = wd_mcp::handle_request(&req).to_string();
        tracing::info!(bytes = resp.len(), "wd-mcp: stdout line out");
        let _ = writeln!(out, "{resp}");
        let _ = out.flush();
    }
    tracing::info!("wd-mcp: stdio loop end");
}

fn main() {
    wd_mcp::init_stderr_logging();
    tracing::info!("wd-mcp: start");
    if std::env::var("MCP_TRANSPORT").is_ok_and(|v| v.eq_ignore_ascii_case("http")) {
        let addr = std::env::var("MCP_ADDR").unwrap_or_else(|_| "127.0.0.1:8765".into());
        tracing::info!(%addr, "wd-mcp: http opt-in serve");
        let listener = std::net::TcpListener::bind(&addr).unwrap_or_else(|e| {
            tracing::error!(%e, "wd-mcp: bind fail");
            std::process::exit(1);
        });
        for stream in listener.incoming() {
            match stream {
                Ok(s) => {
                    serve_http_once(&s);
                }
                Err(e) => tracing::warn!(%e, "wd-mcp: accept fail"),
            }
        }
    } else {
        stdio_loop();
    }
}
