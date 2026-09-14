//! `tools/call` dispatch: ONE core CLI+MCP share.
//!
//! Every call logs in/params/out for debug/perf. No spawn here —
//! pure argv + validation; bins execute. Gates live in `gate.rs`.

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
        "device.boot" => {
            serde_json::json!({"ok": true, "tool": method, "argv": wd_waydroid::boot_args(true, true)})
        }
        "device.status" => serde_json::json!({"ok": true, "tool": method, "argv": ["status"]}),
        "device.list" => {
            serde_json::json!({"ok": true, "tool": method, "devices": [{"serial": "waydroid-0"}]})
        }
        _ => {
            serde_json::json!({"ok": true, "tool": method, "params": params, "structuredContent": {"source": "adb"}})
        }
    };
    tracing::info!(
        method,
        bytes = out.to_string().len(),
        "call: tools/call out"
    );
    out
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
    let out = serde_json::json!({"ok": true, "fingerprint": profile.fingerprint});
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
}
