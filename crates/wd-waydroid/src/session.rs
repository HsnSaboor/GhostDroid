//! Session lifecycle: arg builders plus status parse.
//!
//! Refs: `.devdocs/waydroid/tools/__init__.py:62-72`,
//! `.devdocs/waydroid-mcp/src/waydroid_mcp/adb.py:39-51`.

use wd_core::WdError;

/// Session plus container state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Status {
    /// Session up.
    pub session: bool,
    /// Container up.
    pub container: bool,
    /// Frozen (show session UI).
    pub frozen: bool,
}

/// Build `waydroid session start` args.
#[must_use]
pub fn boot_args(wait: bool, frozen_check: bool) -> Vec<String> {
    tracing::info!(wait, frozen_check, "session: boot args");
    let mut args = vec!["session".to_owned(), "start".to_owned()];
    if wait {
        args.push("--wait".to_owned());
    }
    if frozen_check {
        args.push("--frozen-check".to_owned());
    }
    tracing::debug!(?args, "session: boot args built");
    args
}

/// Build `waydroid session stop` args.
#[must_use]
pub fn shutdown_args() -> Vec<String> {
    tracing::info!("session: shutdown args");
    vec!["session".to_owned(), "stop".to_owned()]
}

/// Build `waydroid container freeze` args.
#[must_use]
pub fn freeze_args() -> Vec<String> {
    tracing::info!("session: freeze args");
    vec!["container".to_owned(), "freeze".to_owned()]
}

/// Build `waydroid container unfreeze` args.
#[must_use]
pub fn unfreeze_args() -> Vec<String> {
    tracing::info!("session: unfreeze args");
    vec!["container".to_owned(), "unfreeze".to_owned()]
}

/// Parse `waydroid status` output into [`Status`].
#[must_use]
pub fn parse_status(out: &str) -> Status {
    tracing::debug!(len = out.len(), "session: parse status");
    let status = Status {
        session: out.contains("Session: RUNNING") || out.contains("RUNNING"),
        container: out.contains("Container: RUNNING"),
        frozen: out.contains("FROZEN"),
    };
    if status.frozen {
        tracing::warn!("session: frozen detected in status output");
    } else {
        tracing::debug!(?status, "session: status parsed");
    }
    status
}

/// Classify a timeout as frozen or plain timeout.
/// Pure port of `waydroid-mcp adb.py _timeout_error`.
#[must_use]
pub fn classify_timeout(status_output: &str, what: &str) -> WdError {
    tracing::info!(what, "session: classify timeout");
    if status_output.contains("FROZEN") {
        tracing::warn!("session: frozen detected, mapping timeout to frozen");
        WdError::Frozen("container frozen - show session UI".to_owned())
    } else {
        tracing::debug!(what, "session: plain timeout");
        WdError::Timeout(what.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_shutdown_freeze_shapes() {
        assert_eq!(boot_args(true, true).len(), 4);
        assert_eq!(shutdown_args(), vec!["session", "stop"]);
        assert_eq!(freeze_args(), vec!["container", "freeze"]);
        assert_eq!(unfreeze_args(), vec!["container", "unfreeze"]);
    }

    #[test]
    fn frozen_beats_timeout() {
        let status = parse_status("Session: RUNNING\nContainer: FROZEN");
        assert!(status.frozen);
        let err = classify_timeout("FROZEN", "adb shell");
        assert!(matches!(err, WdError::Frozen(_)));
        let err2 = classify_timeout("STOPPED", "adb shell");
        assert!(matches!(err2, WdError::Timeout(_)));
    }
}
