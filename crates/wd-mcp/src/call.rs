//! `tools/call` dispatch: ONE core CLI+MCP share.
//!
//! Every call logs in/params/out for debug/perf. Live arms spawn via
//! `wd_waydroid::run_waydroid` (binary + argv only, never a shell) and
//! fall back to argv preview when the binary is missing (CI).
//! Gates live in `gate.rs`.

use crate::{EXIT_DEVICE, EXIT_MISMATCH, gate, tools};

/// Dispatch one tool with params. Returns MCP-style result value.
#[must_use]
pub fn dispatch(method: &str, params: &serde_json::Value) -> serde_json::Value {
    tracing::info!(method, params = %params, "call: tools/call in");
    if !tools::has_tool(method) {
        tracing::warn!(method, "call: unknown tool");
        return serde_json::json!({"ok": false, "error": format!("unknown tool {method}"), "isError": true});
    }
    let out = match method {
        "shell.exec" if !gate::shell_allowed() => {
            serde_json::json!({"ok": false, "error": "shell blocked: set ANDROID_MCP_ALLOW_SHELL=1", "isError": true})
        }
        "file.push" | "file.pull" if !gate::write_allowed() => {
            serde_json::json!({"ok": false, "error": "write blocked: set ANDROID_MCP_ALLOW_WRITE=1", "isError": true})
        }
        "device.boot" => boot_live(),
        "device.status" => status_live(),
        "device.list" => list_live(),
        "app.start" => app_start_live(params),
        "app.stop" => app_stop_live(params),
        "app.list" => app_list_live(),
        "app.install" => app_install_live(params),
        "app.uninstall" => app_uninstall_live(params),
        "activity.current" => activity_live(),
        "input.tap" => input_live(params, "input.tap"),
        "input.swipe" => input_live(params, "input.swipe"),
        "input.key" => input_live(params, "input.key"),
        "input.text" => input_live(params, "input.text"),
        "vision.screenshot" => screenshot_live(),
        "ui.dump" => ui_dump_live(params),
        "ui.find" => ui_find_live(params),
        "logcat.dump" => logcat_live(params),
        "shell.exec" => shell_live(params),
        "file.push" => push_live(params),
        "file.pull" => pull_live(params),
        "prop.get" => prop_get_live(params),
        "prop.set" => prop_set_live(params),
        "keymap.load" => keymap_load_live(params),
        "spoof.load" => spoof_load_live(params),
        _ => {
            serde_json::json!({"ok": false, "error": format!("unhandled tool {method}"), "isError": true})
        }
    };
    tracing::info!(
        method,
        bytes = out.to_string().len(),
        "call: tools/call out"
    );
    out
}

/// Run `waydroid <argv>` with a timeout. Pure spawn helper.
fn run(argv: &[String], timeout_ms: u64) -> Result<String, wd_core::WdError> {
    let refs: Vec<&str> = argv.iter().map(String::as_str).collect();
    wd_waydroid::run_waydroid(&refs, timeout_ms)
}

/// Run an arbitrary binary (`adb` for file verbs) with timeout kill.
/// Binary + argv only, never a shell. `Io` surfaces missing binaries
/// so callers can fall back to argv preview on CI.
fn run_bin(binary: &str, argv: &[String], timeout_ms: u64) -> Result<String, wd_core::WdError> {
    use std::io::Read as _;
    use std::process::Stdio;
    use std::time::{Duration, Instant};
    tracing::info!(binary, ?argv, timeout_ms, "call: run_bin in");
    let mut child = std::process::Command::new(binary)
        .args(argv)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(wd_core::WdError::Io)?;
    let stdout_take = child.stdout.take();
    let stderr_take = child.stderr.take();
    let out_handle = stdout_take.map(|p| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = p.take(262_144).read_to_end(&mut buf);
            buf.truncate(262_144);
            buf
        })
    });
    let err_handle = stderr_take.map(|p| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = p.take(262_144).read_to_end(&mut buf);
            buf.truncate(262_144);
            buf
        })
    });
    let start = Instant::now();
    let budget = Duration::from_millis(timeout_ms);
    let status = loop {
        if let Some(exit) = child.try_wait().map_err(wd_core::WdError::Io)? {
            break exit;
        }
        if start.elapsed() >= budget {
            tracing::warn!(binary, "call: run_bin timeout, killing");
            let _ = child.kill();
            let _ = child.wait();
            return Err(wd_core::WdError::Timeout(format!("{binary} timed out")));
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    let stdout_bytes = out_handle.and_then(|h| h.join().ok()).unwrap_or_default();
    let stderr_bytes = err_handle.and_then(|h| h.join().ok()).unwrap_or_default();
    let stdout_text = String::from_utf8_lossy(&stdout_bytes).into_owned();
    if status.success() {
        Ok(stdout_text)
    } else {
        let detail = String::from_utf8_lossy(&stderr_bytes).trim().to_owned();
        Err(wd_core::WdError::Internal(format!(
            "{binary} failed: {detail}"
        )))
    }
}

/// Missing-binary fallback: argv preview (CI has no waydroid).
fn argv_fallback(tool: &str, argv: &[String]) -> serde_json::Value {
    tracing::warn!(tool, "call: spawn failed, argv fallback");
    serde_json::json!({"ok": true, "tool": tool, "argv": argv})
}

/// `shell` needs euid 0 (vendored `tools/__init__.py:actionNeedRoot`),
/// but app verbs (`app ...`, `status`, `prop get`) need the USER session
/// bus (Waydroid rejects foreign uids). Route callers: app verbs as user,
/// shell/logcat via sudo. Surfaces explicit hints for each case.
fn root_hint(argv: &[String], err: &wd_core::WdError) -> Option<serde_json::Value> {
    let msg = err.to_string();
    let refs: Vec<&str> = argv.iter().map(String::as_str).collect();
    if msg.contains("needs root access") {
        tracing::warn!("call: waydroid shell needs root (sudo)");
        Some(
            serde_json::json!({"ok": false, "error": "waydroid shell needs root: rerun via sudo", "isError": true}),
        )
    } else if msg.contains("session is stopped") && wd_waydroid::is_app_verb(&refs) {
        tracing::warn!("call: app verb under sudo sees dead session (run as user)");
        Some(
            serde_json::json!({"ok": false, "error": "app verb needs user session: run without sudo", "isError": true}),
        )
    } else {
        None
    }
}

/// Map spawn errors: hints, missing binary → argv fallback, else error.
fn spawn_out(
    tool: &str,
    argv: &[String],
    res: Result<String, wd_core::WdError>,
) -> Option<serde_json::Value> {
    match res {
        Ok(_) => None,
        Err(wd_core::WdError::Io(_)) => Some(argv_fallback(tool, argv)),
        Err(err) => {
            if let Some(hint) = root_hint(argv, &err) {
                return Some(hint);
            }
            tracing::warn!(tool, error = %err, "call: live failed");
            Some(serde_json::json!({"ok": false, "error": err.to_string(), "isError": true}))
        }
    }
}

/// One string param: object key first, then positional `args[i]`
/// (wd-ctl passes `{"args": [...]}`).
fn str_param(params: &serde_json::Value, key: &str, idx: usize) -> Option<String> {
    params
        .get(key)
        .and_then(|v| v.as_str().map(str::to_owned))
        .or_else(|| {
            params
                .get("args")
                .and_then(|a| a.as_array())
                .and_then(|a| a.get(idx))
                .and_then(|v| v.as_str().map(str::to_owned))
        })
}

/// Missing-param error JSON.
fn need(tool: &str, what: &str) -> serde_json::Value {
    serde_json::json!({"ok": false, "error": format!("{tool}: need {what}"), "isError": true})
}

/// `device.boot`: run `waydroid session start --wait`, then poll status.
/// Falls back to argv (no spawn) when the binary is missing.
fn boot_live() -> serde_json::Value {
    tracing::info!("call: device.boot live in");
    let argv = wd_waydroid::boot_args(true, true);
    let start: Vec<&str> = argv.iter().map(String::as_str).collect();
    if wd_waydroid::run_waydroid(&start, 120_000).is_err() {
        tracing::warn!("call: boot spawn failed, argv fallback");
        return serde_json::json!({"ok": true, "tool": "device.boot", "argv": argv});
    }
    status_live()
}

/// `device.status`: live `waydroid status` parse (argv fallback, no binary).
fn status_live() -> serde_json::Value {
    tracing::info!("call: device.status live in");
    match wd_waydroid::run_waydroid(&["status"], 10_000) {
        Ok(out) => {
            let parsed = wd_waydroid::parse_status(&out);
            serde_json::json!({"ok": true, "tool": "device.status", "session": parsed.session, "container": parsed.container, "frozen": parsed.frozen, "ip": parsed.ip})
        }
        Err(wd_core::WdError::Io(_)) => {
            serde_json::json!({"ok": true, "tool": "device.status", "argv": ["status"]})
        }
        Err(err) => {
            serde_json::json!({"ok": false, "error": err.to_string(), "isError": true})
        }
    }
}

/// `device.list`: live `waydroid app list` parse (argv fallback, no binary).
fn list_live() -> serde_json::Value {
    tracing::info!("call: device.list live in");
    match wd_waydroid::run_waydroid(&["app", "list"], 15_000) {
        Ok(out) => {
            let apps: Vec<serde_json::Value> = wd_waydroid::parse_app_list(&out)
                .iter()
                .map(|row| serde_json::json!({"title": row.title, "pkg": row.pkg}))
                .collect();
            serde_json::json!({"ok": true, "tool": "device.list", "devices": [{"serial": "waydroid-0"}], "apps": apps})
        }
        Err(wd_core::WdError::Io(_)) => {
            serde_json::json!({"ok": true, "tool": "device.list", "devices": [{"serial": "waydroid-0"}]})
        }
        Err(err) => {
            serde_json::json!({"ok": false, "error": err.to_string(), "isError": true})
        }
    }
}

/// `app.start`: live `waydroid app launch <pkg>`.
fn app_start_live(params: &serde_json::Value) -> serde_json::Value {
    tracing::info!("call: app.start live in");
    let Some(pkg) = str_param(params, "pkg", 0).or_else(|| str_param(params, "package", 0)) else {
        return need("app.start", "pkg");
    };
    let argv = wd_waydroid::launch_args(&wd_core::Pkg(pkg.clone()));
    match run(&argv, 30_000) {
        Ok(_) => serde_json::json!({"ok": true, "tool": "app.start", "pkg": pkg}),
        Err(e) => {
            spawn_out("app.start", &argv, Err(e)).unwrap_or_else(|| serde_json::json!({"ok": true}))
        }
    }
}

/// `app.stop`: live `waydroid shell am force-stop <pkg>`.
fn app_stop_live(params: &serde_json::Value) -> serde_json::Value {
    tracing::info!("call: app.stop live in");
    let Some(pkg) = str_param(params, "pkg", 0).or_else(|| str_param(params, "package", 0)) else {
        return need("app.stop", "pkg");
    };
    let argv = vec![
        "shell".to_owned(),
        "--".to_owned(),
        "am".to_owned(),
        "force-stop".to_owned(),
        pkg.clone(),
    ];
    match run(&argv, 15_000) {
        Ok(_) => serde_json::json!({"ok": true, "tool": "app.stop", "pkg": pkg}),
        Err(e) => {
            spawn_out("app.stop", &argv, Err(e)).unwrap_or_else(|| serde_json::json!({"ok": true}))
        }
    }
}

/// `app.list`: live `waydroid app list` parse.
fn app_list_live() -> serde_json::Value {
    tracing::info!("call: app.list live in");
    let argv = wd_waydroid::list_args();
    match run(&argv, 15_000) {
        Ok(out) => {
            let apps: Vec<serde_json::Value> = wd_waydroid::parse_app_list(&out)
                .iter()
                .map(|row| serde_json::json!({"title": row.title, "pkg": row.pkg}))
                .collect();
            serde_json::json!({"ok": true, "tool": "app.list", "apps": apps})
        }
        Err(e) => {
            spawn_out("app.list", &argv, Err(e)).unwrap_or_else(|| serde_json::json!({"ok": true}))
        }
    }
}

/// `app.install`: live `waydroid app install <apk>`.
fn app_install_live(params: &serde_json::Value) -> serde_json::Value {
    tracing::info!("call: app.install live in");
    let Some(apk) = str_param(params, "apk", 0).or_else(|| str_param(params, "path", 0)) else {
        return need("app.install", "apk path");
    };
    let argv = wd_waydroid::install_args(&apk);
    match run(&argv, 120_000) {
        Ok(out) => {
            serde_json::json!({"ok": true, "tool": "app.install", "apk": apk, "output": out.lines().last().unwrap_or("")})
        }
        Err(e) => spawn_out("app.install", &argv, Err(e))
            .unwrap_or_else(|| serde_json::json!({"ok": true})),
    }
}

/// `app.uninstall`: live `waydroid app remove <pkg>`.
fn app_uninstall_live(params: &serde_json::Value) -> serde_json::Value {
    tracing::info!("call: app.uninstall live in");
    let Some(pkg) = str_param(params, "pkg", 0).or_else(|| str_param(params, "package", 0)) else {
        return need("app.uninstall", "pkg");
    };
    let argv = wd_waydroid::remove_args(&wd_core::Pkg(pkg.clone()));
    match run(&argv, 60_000) {
        Ok(_) => serde_json::json!({"ok": true, "tool": "app.uninstall", "pkg": pkg}),
        Err(e) => spawn_out("app.uninstall", &argv, Err(e))
            .unwrap_or_else(|| serde_json::json!({"ok": true})),
    }
}

/// Parse `dumpsys activity activities` for the focused app line.
fn parse_focused(out: &str) -> Option<String> {
    out.lines().find_map(|line| {
        let t = line.trim();
        t.strip_prefix("mFocusedApp=")
            .or_else(|| t.strip_prefix("mResumedActivity:"))
            .or_else(|| t.strip_prefix("ResumedActivity:"))
            .map(|v| v.trim().to_owned())
    })
}

/// `activity.current`: live focused-app parse via dumpsys.
fn activity_live() -> serde_json::Value {
    tracing::info!("call: activity.current live in");
    let argv = vec![
        "shell".to_owned(),
        "--".to_owned(),
        "dumpsys".to_owned(),
        "activity".to_owned(),
        "activities".to_owned(),
    ];
    match run(&argv, 15_000) {
        Ok(out) => {
            serde_json::json!({"ok": true, "tool": "activity.current", "activity": parse_focused(&out)})
        }
        Err(e) => spawn_out("activity.current", &argv, Err(e))
            .unwrap_or_else(|| serde_json::json!({"ok": true})),
    }
}

/// Build `waydroid shell input ...` argv from tool + params.
fn input_argv(tool: &str, params: &serde_json::Value) -> Result<Vec<String>, serde_json::Value> {
    let arg = |key: &str, idx: usize| str_param(params, key, idx);
    match tool {
        "input.tap" => match (arg("x", 0), arg("y", 1)) {
            (Some(x), Some(y)) => Ok(vec![
                "shell".into(),
                "--".into(),
                "input".into(),
                "tap".into(),
                x,
                y,
            ]),
            _ => Err(need(tool, "x y")),
        },
        "input.swipe" => {
            let pts = [arg("x1", 0), arg("y1", 1), arg("x2", 2), arg("y2", 3)];
            let dur = arg("duration_ms", 4).or_else(|| arg("duration", 4));
            match pts {
                [Some(x1), Some(y1), Some(x2), Some(y2)] => {
                    let mut argv = vec![
                        "shell".into(),
                        "--".into(),
                        "input".into(),
                        "swipe".into(),
                        x1,
                        y1,
                        x2,
                        y2,
                    ];
                    if let Some(d) = dur {
                        argv.push(d);
                    }
                    Ok(argv)
                }
                _ => Err(need(tool, "x1 y1 x2 y2")),
            }
        }
        "input.key" => arg("key", 0).or_else(|| arg("keycode", 0)).map_or_else(
            || Err(need(tool, "key")),
            |k| {
                Ok(vec![
                    "shell".into(),
                    "--".into(),
                    "input".into(),
                    "keyevent".into(),
                    k,
                ])
            },
        ),
        _ => arg("text", 0).map_or_else(
            || Err(need(tool, "text")),
            |t| {
                Ok(vec![
                    "shell".into(),
                    "--".into(),
                    "input".into(),
                    "text".into(),
                    t.replace(' ', "%s"),
                ])
            },
        ),
    }
}

/// `input.*`: live `waydroid shell input ...`.
fn input_live(params: &serde_json::Value, tool: &str) -> serde_json::Value {
    tracing::info!(tool, "call: input live in");
    let argv = match input_argv(tool, params) {
        Ok(a) => a,
        Err(e) => return e,
    };
    match run(&argv, 15_000) {
        Ok(_) => serde_json::json!({"ok": true, "tool": tool, "argv": argv}),
        Err(e) => spawn_out(tool, &argv, Err(e))
            .unwrap_or_else(|| serde_json::json!({"ok": true, "tool": tool})),
    }
}

/// Epoch millis for unique remotes. Zero when the clock is unavailable.
fn epoch_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis().try_into().unwrap_or(0u64))
}

/// `vision.screenshot`: live `screencap -p` to a unique remote, then
/// `base64` it back (`run_waydroid` returns text, never raw PNG bytes).
fn screenshot_live() -> serde_json::Value {
    tracing::info!("call: vision.screenshot live in");
    let remote = crate::ui_dump::unique_remote(epoch_ms())
        .replace("ui_dump_", "shot_")
        .replace(".xml", ".png");
    let cap = vec![
        "shell".to_owned(),
        "--".to_owned(),
        "screencap".to_owned(),
        "-p".to_owned(),
        remote.clone(),
    ];
    match run(&cap, 30_000) {
        Ok(_) => {}
        Err(wd_core::WdError::Io(_)) => return argv_fallback("vision.screenshot", &cap),
        Err(err) => {
            return serde_json::json!({"ok": false, "error": err.to_string(), "isError": true});
        }
    }
    let b64 = vec![
        "shell".to_owned(),
        "--".to_owned(),
        "base64".to_owned(),
        remote.clone(),
    ];
    match run(&b64, 60_000) {
        Ok(out) => {
            let image: String = out.split_whitespace().collect();
            tracing::info!(b64_len = image.len(), "call: screenshot b64 joined");
            match crate::screenshot::clean_png_b64(&image) {
                Ok(clean) => {
                    serde_json::json!({"ok": true, "tool": "vision.screenshot", "source": "waydroid", "remote": remote, "image_base64": clean, "frame": crate::screenshot::FRAME_RESOURCE})
                }
                Err(msg) => {
                    serde_json::json!({"ok": false, "error": format!("vision.screenshot: {msg}"), "isError": true})
                }
            }
        }
        Err(wd_core::WdError::Io(_)) => argv_fallback("vision.screenshot", &cap),
        Err(err) => {
            serde_json::json!({"ok": false, "error": err.to_string(), "isError": true})
        }
    }
}

/// Fetch one ui dump: rm-first, `uiautomator dump`, `cat` back.
/// Returns XML text. Missing binary surfaces as `Io` (caller falls back).
fn fetch_dump(no_tree: bool) -> Result<String, wd_core::WdError> {
    let remote = crate::ui_dump::unique_remote(epoch_ms());
    let _ = run(&crate::ui_dump::rm_args(&remote), 10_000);
    let dump = if crate::ui_dump::tree_disabled(no_tree) {
        crate::ui_dump::tty_args()
    } else {
        crate::ui_dump::dump_args(&remote)
    };
    let out = run(&dump, 30_000)?;
    if !crate::ui_dump::dumped_ok(&out) {
        return Err(wd_core::WdError::Internal(
            "uiautomator dump failed".to_owned(),
        ));
    }
    let cat = vec![
        "shell".to_owned(),
        "--".to_owned(),
        "cat".to_owned(),
        remote,
    ];
    run(&cat, 15_000).map(|text| crate::ui_dump::strip_status(&text).to_owned())
}

/// `ui.dump`: live hierarchy XML (rm-first + retry per `ui_dump` docs).
fn ui_dump_live(params: &serde_json::Value) -> serde_json::Value {
    tracing::info!("call: ui.dump live in");
    let no_tree = params
        .get("no_tree")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let probe = crate::ui_dump::dump_args("/sdcard/.ui_dump_probe.xml");
    for attempt in 0..crate::ui_dump::DUMP_TRIES {
        match fetch_dump(no_tree) {
            Ok(xml) => {
                return serde_json::json!({"ok": true, "tool": "ui.dump", "xml": xml});
            }
            Err(wd_core::WdError::Io(_)) => return argv_fallback("ui.dump", &probe),
            Err(_) if attempt + 1 < crate::ui_dump::DUMP_TRIES => {
                std::thread::sleep(std::time::Duration::from_millis(
                    crate::ui_dump::DUMP_RETRY_GAP_MS,
                ));
            }
            Err(err) => {
                return serde_json::json!({"ok": false, "error": err.to_string(), "isError": true});
            }
        }
    }
    serde_json::json!({"ok": false, "error": "ui.dump: retries exhausted", "isError": true})
}

/// Byte index of `needle` in the dump, if present.
fn find_in_dump(xml: &str, needle: &str) -> Option<usize> {
    tracing::debug!(bytes = xml.len(), "call: ui.find search in");
    xml.find(needle)
}

/// `ui.find`: live dump + substring search for `text`.
fn ui_find_live(params: &serde_json::Value) -> serde_json::Value {
    tracing::info!("call: ui.find live in");
    let Some(needle) = str_param(params, "text", 0) else {
        return need("ui.find", "text");
    };
    let no_tree = params
        .get("no_tree")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let probe = crate::ui_dump::dump_args("/sdcard/.ui_dump_probe.xml");
    match fetch_dump(no_tree) {
        Ok(xml) => find_in_dump(&xml, &needle).map_or_else(
            || serde_json::json!({"ok": true, "tool": "ui.find", "found": false}),
            |at| serde_json::json!({"ok": true, "tool": "ui.find", "found": true, "at": at}),
        ),
        Err(wd_core::WdError::Io(_)) => argv_fallback("ui.find", &probe),
        Err(err) => serde_json::json!({"ok": false, "error": err.to_string(), "isError": true}),
    }
}

/// `logcat.dump`: live `waydroid logcat -d -v threadtime -t N` + parse.
fn logcat_live(params: &serde_json::Value) -> serde_json::Value {
    tracing::info!("call: logcat.dump live in");
    let tail: u32 = params
        .get("tail")
        .and_then(serde_json::Value::as_u64)
        .and_then(|n| n.try_into().ok())
        .unwrap_or(200);
    let min = params
        .get("level")
        .and_then(|v| v.as_str())
        .and_then(|s| s.chars().next())
        .unwrap_or('V');
    let tag = params.get("tag").and_then(|v| v.as_str()).unwrap_or("");
    let contains = params
        .get("message_contains")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let argv = crate::logcat::dump_args(tail);
    let waydroid_argv: Vec<String> = argv.clone();
    match run(&waydroid_argv, 20_000) {
        Ok(out) => {
            let records: Vec<serde_json::Value> = out
                .lines()
                .filter_map(crate::logcat::parse_line)
                .filter(|r| {
                    crate::logcat::at_least(r.level, min)
                        && (tag.is_empty() || r.tag == tag)
                        && (contains.is_empty() || r.message.contains(contains))
                })
                .map(|r| {
                    serde_json::json!({"time": r.time, "pid": r.pid, "tid": r.tid, "level": r.level.to_string(), "tag": r.tag, "message": r.message})
                })
                .collect();
            serde_json::json!({"ok": true, "tool": "logcat.dump", "records": records})
        }
        Err(e) => spawn_out("logcat.dump", &argv, Err(e))
            .unwrap_or_else(|| serde_json::json!({"ok": true})),
    }
}

/// Two path params: named keys first, then positional `args[0..1]`.
/// Shared by `file.push` / `file.pull` so both verbs parse one way (DRY).
fn path_pair(
    params: &serde_json::Value,
    tool: &str,
    first_keys: &[&str],
    second_keys: &[&str],
) -> Result<(String, String), serde_json::Value> {
    let first = first_keys
        .iter()
        .find_map(|k| str_param(params, k, 0))
        .filter(|s| !s.is_empty());
    let second = second_keys
        .iter()
        .find_map(|k| str_param(params, k, 1))
        .filter(|s| !s.is_empty());
    match (first, second) {
        (Some(a), Some(b)) => Ok((a, b)),
        _ => Err(need(tool, "src dst")),
    }
}

/// `shell.exec`: live `waydroid shell -- sh -c <cmd>`.
/// `{"cmd"|"command"|"script": "..."}` wins, else `args` joined.
fn shell_live(params: &serde_json::Value) -> serde_json::Value {
    tracing::info!("call: shell.exec live in");
    let cmd = str_param(params, "cmd", 0)
        .or_else(|| str_param(params, "command", 0))
        .or_else(|| str_param(params, "script", 0))
        .or_else(|| {
            params
                .get("args")
                .and_then(|a| a.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .filter(|s| !s.trim().is_empty())
        });
    let Some(cmd) = cmd.filter(|s| !s.trim().is_empty()) else {
        return need("shell.exec", "cmd");
    };
    let argv = vec![
        "shell".to_owned(),
        "--".to_owned(),
        "sh".to_owned(),
        "-c".to_owned(),
        cmd.clone(),
    ];
    match run(&argv, 30_000) {
        Ok(out) => serde_json::json!({"ok": true, "tool": "shell.exec", "output": out}),
        Err(e) => spawn_out("shell.exec", &argv, Err(e))
            .unwrap_or_else(|| serde_json::json!({"ok": true, "tool": "shell.exec"})),
    }
}

/// `file.push`: live `adb push <src> <dst>` (Waydroid shares the adb
/// transport; `waydroid shell` cannot transfer bytes).
fn push_live(params: &serde_json::Value) -> serde_json::Value {
    tracing::info!("call: file.push live in");
    let (src, dst) = match path_pair(
        params,
        "file.push",
        &["src", "local", "host", "path"],
        &["dst", "remote", "device", "dest"],
    ) {
        Ok(pair) => pair,
        Err(e) => return e,
    };
    let argv = vec!["push".to_owned(), src.clone(), dst.clone()];
    match run_bin("adb", &argv, 120_000) {
        Ok(out) => {
            serde_json::json!({"ok": true, "tool": "file.push", "src": src, "dst": dst, "output": out.lines().last().unwrap_or("")})
        }
        Err(wd_core::WdError::Io(_)) => argv_fallback("file.push", &argv),
        Err(err) => serde_json::json!({"ok": false, "error": err.to_string(), "isError": true}),
    }
}

/// `file.pull`: live `adb pull <src> <dst>`.
fn pull_live(params: &serde_json::Value) -> serde_json::Value {
    tracing::info!("call: file.pull live in");
    let (src, dst) = match path_pair(
        params,
        "file.pull",
        &["src", "remote", "device", "path"],
        &["dst", "local", "host", "dest"],
    ) {
        Ok(pair) => pair,
        Err(e) => return e,
    };
    let argv = vec!["pull".to_owned(), src.clone(), dst.clone()];
    match run_bin("adb", &argv, 120_000) {
        Ok(out) => {
            serde_json::json!({"ok": true, "tool": "file.pull", "src": src, "dst": dst, "output": out.lines().last().unwrap_or("")})
        }
        Err(wd_core::WdError::Io(_)) => argv_fallback("file.pull", &argv),
        Err(err) => serde_json::json!({"ok": false, "error": err.to_string(), "isError": true}),
    }
}

/// `prop.get`: live `waydroid prop get <key>` via `wd-waydroid` args.
fn prop_get_live(params: &serde_json::Value) -> serde_json::Value {
    tracing::info!("call: prop.get live in");
    let Some(key) = str_param(params, "key", 0).or_else(|| str_param(params, "name", 0)) else {
        return need("prop.get", "key");
    };
    let argv = wd_waydroid::get_args(&key);
    match run(&argv, 10_000) {
        Ok(out) => {
            serde_json::json!({"ok": true, "tool": "prop.get", "key": key, "value": out.trim()})
        }
        Err(e) => spawn_out("prop.get", &argv, Err(e))
            .unwrap_or_else(|| serde_json::json!({"ok": true, "tool": "prop.get"})),
    }
}

/// `prop.set`: live `waydroid prop set <key> <value>` via `wd-waydroid` args.
fn prop_set_live(params: &serde_json::Value) -> serde_json::Value {
    tracing::info!("call: prop.set live in");
    let key = str_param(params, "key", 0).or_else(|| str_param(params, "name", 0));
    let value = str_param(params, "value", 1);
    let (Some(key), Some(value)) = (key, value) else {
        return need("prop.set", "key value");
    };
    let argv = wd_waydroid::set_args(&key, &value);
    match run(&argv, 10_000) {
        Ok(_) => serde_json::json!({"ok": true, "tool": "prop.set", "key": key, "value": value}),
        Err(e) => spawn_out("prop.set", &argv, Err(e))
            .unwrap_or_else(|| serde_json::json!({"ok": true, "tool": "prop.set"})),
    }
}

/// `keymap.load` dispatch arm: `{"path": "..."}` validated via `wd-input`.
fn keymap_load_live(params: &serde_json::Value) -> serde_json::Value {
    tracing::info!("call: keymap.load live in");
    let Some(path) = str_param(params, "path", 0).or_else(|| str_param(params, "profile", 0))
    else {
        return need("keymap.load", "path");
    };
    match keymap_load(std::path::Path::new(&path)) {
        Ok(out) => out,
        Err(err) => serde_json::json!({"ok": false, "error": err.to_string(), "isError": true}),
    }
}

/// `spoof.load` dispatch arm: `{"path": "..."}` validated via `wd-spoof`.
fn spoof_load_live(params: &serde_json::Value) -> serde_json::Value {
    tracing::info!("call: spoof.load live in");
    let Some(path) = str_param(params, "path", 0).or_else(|| str_param(params, "profile", 0))
    else {
        return need("spoof.load", "path");
    };
    match spoof_load(std::path::Path::new(&path)) {
        Ok(out) => out,
        Err(err) => serde_json::json!({"ok": false, "error": err.to_string(), "isError": true}),
    }
}

/// `keymap.load` helper: validates via `wd-input`, never duplicates schema.
///
/// # Errors
/// Returns [`wd_core::WdError`] on IO/JSON/validation failure.
pub fn keymap_load(path: &std::path::Path) -> Result<serde_json::Value, wd_core::WdError> {
    tracing::info!(path = %path.display(), "call: keymap.load in");
    let profile = wd_input::validate::load(path)?;
    let out = serde_json::json!({"ok": true, "name": profile.name, "nodes": profile.nodes.len()});
    tracing::info!(bytes = out.to_string().len(), "call: keymap.load out");
    Ok(out)
}

/// `spoof.load` helper. Validates via `wd-spoof`.
///
/// # Errors
/// Returns [`wd_core::WdError`] on IO/TOML/validation failure.
pub fn spoof_load(path: &std::path::Path) -> Result<serde_json::Value, wd_core::WdError> {
    tracing::info!(path = %path.display(), "call: spoof.load in");
    let profile = wd_spoof::SpoofProfile::load(path)?;
    let rendered = wd_spoof::render(&profile);
    let leak = rendered
        .iter()
        .any(|l| l.contains("x86") || l.contains("houdini"));
    let out = serde_json::json!({"ok": true, "fingerprint": profile.fingerprint, "lines": rendered.len(), "houdini_safe": !leak});
    tracing::info!(bytes = out.to_string().len(), "call: spoof.load out");
    Ok(out)
}

/// Map a result value to CLI exit code (0 ok, 3 mismatch, 4 device).
#[must_use]
pub fn exit_code_for(result: &serde_json::Value) -> i32 {
    tracing::debug!(%result, "call: exit map in");
    if result.get("ok") == Some(&serde_json::Value::Bool(true)) {
        return 0;
    }
    let err = result.get("error").and_then(|e| e.as_str()).unwrap_or("");
    let code = if err.contains("mismatch") || err.contains("expect") {
        EXIT_MISMATCH
    } else if err.contains("device") || err.contains("frozen") || err.contains("offline") {
        EXIT_DEVICE
    } else {
        1
    };
    tracing::info!(code, err, "call: exit map out");
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_tool_is_error() {
        let r = dispatch("nope", &serde_json::Value::Null);
        assert_eq!(exit_code_for(&r), 1);
    }

    #[test]
    fn shell_gated_by_default() {
        let r = dispatch("shell.exec", &serde_json::json!({"cmd": "id"}));
        assert!(r.get("isError").is_some());
    }

    #[test]
    fn params_object_beats_args() {
        let p = serde_json::json!({"pkg": "com.a", "args": ["com.b"]});
        assert_eq!(str_param(&p, "pkg", 0).as_deref(), Some("com.a"));
        let p2 = serde_json::json!({"args": ["com.b"]});
        assert_eq!(str_param(&p2, "pkg", 0).as_deref(), Some("com.b"));
        assert!(str_param(&serde_json::Value::Null, "pkg", 0).is_none());
    }

    #[test]
    fn missing_params_error() {
        assert!(
            dispatch("app.start", &serde_json::Value::Null)
                .get("isError")
                .is_some()
        );
        assert!(
            dispatch("input.tap", &serde_json::json!({"x": "1"}))
                .get("isError")
                .is_some()
        );
        assert!(
            dispatch("ui.find", &serde_json::Value::Null)
                .get("isError")
                .is_some()
        );
    }

    #[test]
    fn input_argv_shapes() {
        let tap = input_argv("input.tap", &serde_json::json!({"x": "10", "y": "20"})).unwrap();
        assert_eq!(tap, vec!["shell", "--", "input", "tap", "10", "20"]);
        let key = input_argv("input.key", &serde_json::json!({"args": ["4"]})).unwrap();
        assert_eq!(key, vec!["shell", "--", "input", "keyevent", "4"]);
        let swipe = input_argv(
            "input.swipe",
            &serde_json::json!({"args": ["1", "2", "3", "4", "500"]}),
        )
        .unwrap();
        assert_eq!(
            swipe,
            vec!["shell", "--", "input", "swipe", "1", "2", "3", "4", "500"]
        );
        let text = input_argv("input.text", &serde_json::json!({"text": "a b"})).unwrap();
        assert_eq!(text, vec!["shell", "--", "input", "text", "a%sb"]);
    }

    #[test]
    fn focused_parse() {
        let out = "ACTIVITY MANAGER ACTIVITIES\n  mFocusedApp=ActivityRecord{com.a/.Main}\n";
        assert_eq!(
            parse_focused(out).as_deref(),
            Some("ActivityRecord{com.a/.Main}")
        );
        assert!(parse_focused("nothing here").is_none());
    }

    #[test]
    fn find_hits() {
        assert_eq!(find_in_dump("<node text=\"Play\"/>", "Play"), Some(12));
        assert!(find_in_dump("<node/>", "Play").is_none());
    }

    #[test]
    fn new_tools_live_or_param_error() {
        // prop.get/set need params; missing → isError (never panic/unknown).
        assert!(
            dispatch("prop.get", &serde_json::Value::Null)
                .get("isError")
                .is_some()
        );
        assert!(
            dispatch("prop.set", &serde_json::json!({"key": "k"}))
                .get("isError")
                .is_some()
        );
        assert!(
            dispatch("keymap.load", &serde_json::Value::Null)
                .get("isError")
                .is_some()
        );
        assert!(
            dispatch("spoof.load", &serde_json::Value::Null)
                .get("isError")
                .is_some()
        );
        // Removed stubs stay unknown.
        for stub in [
            "vision.stream_start",
            "vision.stream_stop",
            "logcat.start",
            "logcat.stop",
        ] {
            let r = dispatch(stub, &serde_json::Value::Null);
            assert_eq!(r["ok"], serde_json::Value::Bool(false), "{stub}");
        }
        // Bad paths surface isError, never ok.
        let bad = serde_json::json!({"path": "/nonexistent/keymap.json"});
        assert!(dispatch("keymap.load", &bad).get("isError").is_some());
        assert!(dispatch("spoof.load", &bad).get("isError").is_some());
    }

    #[test]
    fn spawn_err_mapping() {
        // Missing binary (Io) → argv fallback, still ok. Other errors → isError.
        let argv = vec!["app".to_owned(), "list".to_owned()];
        let fb = spawn_out(
            "app.list",
            &argv,
            Err(wd_core::WdError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no waydroid",
            ))),
        )
        .unwrap();
        assert_eq!(fb["ok"], serde_json::Value::Bool(true));
        assert!(fb.get("argv").is_some());
        let err = spawn_out(
            "app.list",
            &argv,
            Err(wd_core::WdError::Internal("boom".to_owned())),
        )
        .unwrap();
        assert!(err.get("isError").is_some());
    }
}
