//! Supervised child specs: ghostdroid-server.jar plus keymap bridges.
//!
//! Daemon owns 4 children: ghostdroid jar push, ghostdroid server
//! launch, keymap bridge (`wd-inject serve`), logcat tail. Pure argv
//! specs plus timeout/health helpers; bins own real spawns.
//! Frozen-container detection reuses `wd-waydroid::parse_status`.

/// One supervised child: name plus argv.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Child {
    /// Short name for logs.
    pub name: String,
    /// Full argv (`argv[0]` is the program).
    pub argv: Vec<String>,
}

impl Child {
    /// New spec. Logs every spec build for spawn/perf triage.
    #[must_use]
    pub fn new(name: &str, argv: Vec<String>) -> Self {
        tracing::info!(name, ?argv, "supervisor: child spec");
        Self {
            name: name.to_owned(),
            argv,
        }
    }

    /// One-line spawn preview for logs. No execution here.
    #[must_use]
    pub fn spawn_line(&self) -> String {
        let line = self.argv.join(" ");
        tracing::info!(name = %self.name, line = %line, "supervisor: spawn");
        line
    }
}

/// Per-child spawn timeout, ms (timeout protection: hung children die).
pub const CHILD_TIMEOUT_MS: u64 = 30_000;

/// The 4 daemon-owned children. Pure argv specs.
#[must_use]
pub fn supervised_children() -> Vec<Child> {
    tracing::info!("supervisor: building child set");
    let mut push = vec!["adb".to_owned()];
    push.extend(wd_waydroid::server_push_args("/tmp/ghostdroid-server.jar"));
    let mut launch = vec!["waydroid".to_owned()];
    launch.extend(wd_waydroid::server_launch_args(
        wd_waydroid::SERVER_BIND_HOST,
        wd_inject::SERVER_PORT,
    ));
    let kids = vec![
        Child::new("ghostdroid-push", push),
        Child::new("ghostdroid-server", launch),
        Child::new(
            "keymap-bridge",
            vec!["wd-inject".to_owned(), "serve".to_owned()],
        ),
        Child::new("logcat", vec!["waydroid".to_owned(), "logcat".to_owned()]),
    ];
    tracing::info!(count = kids.len(), "supervisor: child set ready");
    kids
}

/// Timeout for one child spawn, ms (single source for bins).
#[must_use]
pub const fn child_timeout_ms() -> u64 {
    CHILD_TIMEOUT_MS
}

/// True when `waydroid status` output reports a frozen container.
/// Thin wrapper over [`wd_waydroid::parse_status`] so daemon + MCP
/// share one frozen definition (no drift).
#[must_use]
pub fn is_frozen_status(output: &str) -> bool {
    let frozen = wd_waydroid::parse_status(output).frozen;
    if frozen {
        tracing::warn!("supervisor: frozen container detected");
    }
    frozen
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_children_logged() {
        let kids = supervised_children();
        assert_eq!(kids.len(), 4);
        assert_eq!(kids[0].name, "ghostdroid-push");
        assert!(kids[0].spawn_line().contains("ghostdroid-server.jar"));
        assert!(kids[1].spawn_line().contains("ghostdroid"));
        assert!(kids[2].spawn_line().contains("wd-inject"));
        assert!(kids[3].spawn_line().contains("logcat"));
    }

    #[test]
    fn frozen_and_timeout_helpers() {
        assert!(is_frozen_status("Container: FROZEN"));
        assert!(!is_frozen_status("Container: RUNNING"));
        assert!(child_timeout_ms() > 0);
    }
}
