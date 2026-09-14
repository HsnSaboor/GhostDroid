//! `wd-ctl` bin: thin CLI over `wd-mcp` lib fns (dual-use).
//!
//! Port of `waydroid-mcp/cli.py`: `--json` always, exits
//! `3=mismatch 4=device`. Every command = one lib fn so CLI
//! and MCP cannot drift. No heavy deps (weak i5).

use std::io::Write as _;

/// CLI verbs → tool names (24, 1:1 with [`wd_mcp::TOOLS`]).
/// `prop-get/set`, `keymap-load`, `spoof-load` are NOT tools —
/// they ride `shell.exec` / local validate helpers below.
const VERBS: &[(&str, &str)] = &[
    ("devices", "device.list"),
    ("boot", "device.boot"),
    ("status", "device.status"),
    ("install", "app.install"),
    ("uninstall", "app.uninstall"),
    ("launch", "app.start"),
    ("stop", "app.stop"),
    ("list-apps", "app.list"),
    ("current", "activity.current"),
    ("tap", "input.tap"),
    ("swipe", "input.swipe"),
    ("key", "input.key"),
    ("text", "input.text"),
    ("screenshot", "vision.screenshot"),
    ("record-start", "vision.stream_start"),
    ("record-stop", "vision.stream_stop"),
    ("ui-dump", "ui.dump"),
    ("ui-find", "ui.find"),
    ("shell", "shell.exec"),
    ("push", "file.push"),
    ("pull", "file.pull"),
    ("logcat", "logcat.dump"),
    ("logcat-start", "logcat.start"),
    ("logcat-stop", "logcat.stop"),
];

/// Local-only verbs handled without MCP dispatch.
const LOCAL_VERBS: &[&str] = &["prop-get", "prop-set", "keymap-load", "spoof-load"];

/// Parse `argv` → `(tool, params)`. No `clap` (zero new deps).
fn parse(argv: &[String]) -> Result<(String, serde_json::Value), String> {
    tracing::debug!(?argv, "wd-ctl: parse in");
    let mut rest: Vec<&str> = Vec::new();
    let mut serial: Option<&str> = None;
    let mut i = 0;
    while i < argv.len() {
        match argv[i].as_str() {
            "-s" | "--device" => {
                i += 1;
                serial = argv.get(i).map(String::as_str);
            }
            "--json" => {}
            a => rest.push(a),
        }
        i += 1;
    }
    let (verb, tail) = rest.split_first().ok_or_else(usage)?;
    if LOCAL_VERBS.contains(verb) {
        return Ok(((*verb).to_owned(), serde_json::json!({"args": tail})));
    }
    let tool = VERBS
        .iter()
        .find(|(v, _)| v == verb)
        .map(|(_, t)| *t)
        .ok_or_else(usage)?;
    let mut params = serde_json::json!({"args": tail});
    if let Some(s) = serial {
        params["serial"] = s.into();
    }
    tracing::info!(verb, tool, "wd-ctl: parsed");
    Ok((tool.to_owned(), params))
}

/// Emit result as one JSON line; exit per `exit_code_for`.
fn emit(result: &serde_json::Value) -> i32 {
    let code = wd_mcp::exit_code_for(result);
    tracing::info!(code, "wd-ctl: emit");
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let _ = writeln!(out, "{result}");
    code
}

fn usage() -> String {
    "usage: wd-ctl [-s SERIAL] [--json] <verb> [args..]".to_owned()
}

fn main() {
    wd_mcp::init_stderr_logging();
    tracing::info!("wd-ctl: start");
    let argv: Vec<String> = std::env::args().skip(1).collect();
    tracing::debug!(bytes = argv.join(" ").len(), "wd-ctl: argv bytes");
    let code = match parse(&argv) {
        Ok((tool, params)) => emit(&run_local(&tool, &params)),
        Err(u) => {
            eprintln!("{u}");
            2
        }
    };
    tracing::info!(code, "wd-ctl: exit");
    std::process::exit(code);
}

/// Run: local-only verbs via lib helpers, else MCP dispatch.
fn run_local(verb: &str, params: &serde_json::Value) -> serde_json::Value {
    tracing::info!(verb, "wd-ctl: run in");
    let args: Vec<String> = params
        .get("args")
        .and_then(|a| a.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    let out = match verb {
        "prop-get" => args.first().map(|k| {
            serde_json::json!({"ok": true, "tool": "prop.get", "argv": wd_waydroid::get_args(k)})
        }),
        "prop-set" => match args.as_slice() {
            [k, v, ..] => Some(
                serde_json::json!({"ok": true, "tool": "prop.set", "argv": wd_waydroid::set_args(k, v)}),
            ),
            _ => None,
        },
        "keymap-load" => args.first().and_then(|p| wd_mcp::keymap_load(std::path::Path::new(p)).ok()),
        "spoof-load" => args.first().and_then(|p| wd_mcp::spoof_load(std::path::Path::new(p)).ok()),
        _ => None,
    };
    match out {
        Some(v) => v,
        None if LOCAL_VERBS.contains(&verb) => {
            serde_json::json!({"ok": false, "error": format!("{verb}: missing/invalid args")})
        }
        None => wd_mcp::dispatch(verb, params),
    }
}
