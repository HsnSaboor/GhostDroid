//! Supervised child specs. Data plus log lines only, no spawn.
//!
//! Daemon owns 4 children: logcat tail, screencap, py-sidecar,
//! inject socket. Ref: `.plans/02-core-daemon.md`.

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

/// The 4 daemon-owned children. Pure argv specs.
#[must_use]
pub fn supervised_children() -> Vec<Child> {
    tracing::info!("supervisor: building child set");
    let kids = vec![
        Child::new("logcat", vec!["waydroid".to_owned(), "logcat".to_owned()]),
        Child::new(
            "screencap",
            vec!["waydroid".to_owned(), "screencap".to_owned()],
        ),
        Child::new(
            "py-sidecar",
            vec!["python3".to_owned(), "py-sidecar/wd_sidecar.py".to_owned()],
        ),
        Child::new(
            "inject-socket",
            vec!["wd-inject".to_owned(), "serve".to_owned()],
        ),
    ];
    tracing::info!(count = kids.len(), "supervisor: child set ready");
    kids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_children_logged() {
        let kids = supervised_children();
        assert_eq!(kids.len(), 4);
        assert!(kids[0].spawn_line().contains("logcat"));
    }
}
