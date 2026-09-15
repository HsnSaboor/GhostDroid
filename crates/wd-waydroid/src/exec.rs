//! Supervised `waydroid` spawn: timeout + output caps.
//!
//! Argv only, no shell. Entry logs at info, exit at debug,
//! timeout-kill at warn. Refs: `.plans/10-ghostdroid-1.0.md:20-23`.

use std::io::Read as _;
use std::process::Stdio;
use std::time::{Duration, Instant};

/// Per-stream output cap (256KB each for stdout/stderr).
const CAP_U64: u64 = 262_144;
/// Same cap as `usize` for [`Vec::truncate`] (no casts, clippy-clean).
const CAP_LEN: usize = 262_144;
/// Poll gap while waiting for the child.
const POLL_MS: u64 = 5;

/// True for USER-session-bus verbs.
///
/// These MUST run as the session user — sudo'd they see "session stopped"
/// (bus rejects foreign uids). `shell`/`logcat` need euid 0, keep sudo.
#[must_use]
pub fn is_app_verb(args: &[&str]) -> bool {
    let verb = args.first().copied().unwrap_or("");
    tracing::debug!(verb, "exec: app verb check");
    // `app ...` needs the session service; `status`/`prop get` also read
    // session state. `shell`/`logcat`/`upgrade` etc stay elevated.
    let app = matches!(verb, "app" | "status" | "show-full-ui" | "first-launch")
        || (verb == "prop" && args.get(1).copied() == Some("get"));
    tracing::debug!(verb, app, "exec: app verb result");
    app
}
fn drain_capped(pipe: impl std::io::Read + Send + 'static) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = pipe.take(CAP_U64).read_to_end(&mut buf);
        buf.truncate(CAP_LEN);
        buf
    })
}

/// Run `waydroid <args>` with a timeout, return stdout.
///
/// Kills the child on expiry. Stderr lands in the error on failure.
/// Never a shell: binary + argv array only.
///
/// # Errors
///
/// Returns [`wd_core::WdError::Io`] when spawn/wait fails (incl missing
/// binary), [`wd_core::WdError::Timeout`] on expiry, or
/// [`wd_core::WdError::Internal`] on non-zero exit.
pub fn run_waydroid(args: &[&str], timeout_ms: u64) -> wd_core::Result<String> {
    tracing::info!(?args, timeout_ms, "exec: run in");
    let mut cmd = std::process::Command::new("waydroid");
    cmd.args(args);
    // No bus forwarding: Waydroid's session bus rejects foreign uids, so
    // sudo'd app verbs can never see the user session. Split instead:
    // app verbs run as the user, shell/logcat run sudo (needs euid 0).
    // See `is_app_verb` + SKILL routing (`waydroid-control`).
    let mut child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(wd_core::WdError::Io)?;
    let stdout_handle = child.stdout.take().map(drain_capped);
    let stderr_handle = child.stderr.take().map(drain_capped);
    let start = Instant::now();
    let budget = Duration::from_millis(timeout_ms);
    let status = loop {
        if let Some(exit) = child.try_wait().map_err(wd_core::WdError::Io)? {
            break exit;
        }
        if start.elapsed() >= budget {
            tracing::warn!(?args, timeout_ms, "exec: timeout, killing waydroid");
            let _ = child.kill();
            let _ = child.wait();
            return Err(wd_core::WdError::Timeout(format!(
                "waydroid {} timed out",
                args.join(" ")
            )));
        }
        std::thread::sleep(Duration::from_millis(POLL_MS));
    };
    let stdout_bytes = stdout_handle
        .and_then(|h| h.join().ok())
        .unwrap_or_default();
    let stderr_bytes = stderr_handle
        .and_then(|h| h.join().ok())
        .unwrap_or_default();
    let stdout_text = String::from_utf8_lossy(&stdout_bytes).into_owned();
    if status.success() {
        tracing::debug!(len = stdout_text.len(), "exec: run out");
        Ok(stdout_text)
    } else {
        let detail = String::from_utf8_lossy(&stderr_bytes).trim().to_owned();
        tracing::warn!(?args, %detail, "exec: non-zero exit");
        Err(wd_core::WdError::Internal(format!(
            "waydroid {} failed: {detail}",
            args.join(" ")
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verb_split() {
        assert!(is_app_verb(&["app", "list"]));
        assert!(is_app_verb(&["app", "launch", "x"]));
        assert!(is_app_verb(&["status"]));
        assert!(is_app_verb(&["prop", "get", "k"]));
        assert!(!is_app_verb(&["shell", "--", "input"]));
        assert!(!is_app_verb(&["logcat", "--", "-d"]));
        assert!(!is_app_verb(&["prop", "set", "k", "v"]));
        assert!(!is_app_verb(&[]));
    }
}
