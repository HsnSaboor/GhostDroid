//! `ui.dump` guards: rm-first + retry + `--no-tree`.
//!
//! Ports `waydroid-mcp core.py:43-95` (`Core(tree)`; dump rebinds
//! a11y ~0.8s so canvas apps skip via `--no-tree` / `WM_NO_TREE=1`)
//! with rm-first and retry 3x1.0s; tmp shape `scrcpy-mcp ui.ts:52-60`
//! unique `/sdcard/.ui_dump_${ms}_${rand}.xml` plus strip dumped-to
//! line; `adb-mcp ui.py:35-44` `/dev/tty` fast-path optional.
//! NEVER fixed no-rm path (us stale-prone: silent wrong screen).

/// Dump attempts before giving up.
pub const DUMP_TRIES: u32 = 3;
/// Gap between dump retries, ms.
pub const DUMP_RETRY_GAP_MS: u64 = 1_000;
/// Remote tmp prefix (unique suffix appended by caller).
pub const REMOTE_PREFIX: &str = "/sdcard/.ui_dump_";

/// True when the tree should be skipped (canvas app / a11y busy).
/// CLI `--no-tree` or `WM_NO_TREE=1`.
#[must_use]
pub fn tree_disabled(no_tree_flag: bool) -> bool {
    let env = std::env::var("WM_NO_TREE").is_ok_and(|v| v == "1");
    let off = no_tree_flag || env;
    tracing::debug!(off, "ui_dump: tree gate");
    off
}

/// Unique remote path (ms + pid — no rand dep on weak i5).
#[must_use]
pub fn unique_remote(now_ms: u64) -> String {
    let path = format!("{REMOTE_PREFIX}{now_ms}_{}.xml", std::process::id());
    tracing::debug!(path, "ui_dump: remote path");
    path
}

/// rm-first argv. Always run before dump (stale = wrong screen).
/// `--` separators stop waydroid's own argparse eating flags like `-f`.
#[must_use]
pub fn rm_args(remote: &str) -> Vec<String> {
    tracing::debug!(remote, "ui_dump: rm-first");
    vec![
        "shell".into(),
        "--".into(),
        "rm".into(),
        "-f".into(),
        remote.into(),
    ]
}

/// `uiautomator dump <remote>` argv.
#[must_use]
pub fn dump_args(remote: &str) -> Vec<String> {
    tracing::info!(remote, "ui_dump: dump args");
    vec![
        "shell".into(),
        "--".into(),
        "uiautomator".into(),
        "dump".into(),
        remote.into(),
    ]
}

/// `/dev/tty` fast-path argv (`adb-mcp ui.py` optional first try).
#[must_use]
pub fn tty_args() -> Vec<String> {
    tracing::debug!("ui_dump: tty fast-path");
    vec![
        "shell".into(),
        "--".into(),
        "uiautomator".into(),
        "dump".into(),
        "/dev/tty".into(),
    ]
}

/// True when `uiautomator` output means success.
#[must_use]
pub fn dumped_ok(out: &str) -> bool {
    let ok = out.contains("dumped to") || out.contains("<hierarchy");
    tracing::debug!(ok, "ui_dump: dumped check");
    ok
}

/// Strip the `UI hierchay dumped to: ...` status line, keep XML.
#[must_use]
pub fn strip_status(out: &str) -> &str {
    tracing::debug!(bytes = out.len(), "ui_dump: strip in");
    out.find('<').map_or(out, |i| &out[i..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rm_first_and_retry_consts() {
        assert_eq!(DUMP_TRIES, 3);
        let r = unique_remote(1);
        assert!(rm_args(&r).contains(&"rm".to_owned()));
        assert!(dumped_ok("UI hierchay dumped to: /sdcard/x.xml"));
        assert!(strip_status("dumped to: x\n<hier ok").starts_with('<'));
    }
}
