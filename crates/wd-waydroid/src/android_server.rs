//! Android touch server push/run: pure argv builders plus ping parse.
//!
//! Pure builders + parsers only. No spawn, no sleep, no socket, no shell.
//! Every builder returns an argv [`Vec`] (binary + args); callers spawn via
//! `wd_waydroid::exec::run_waydroid` (binary + argv array only, never a
//! shell). Port re-uses [`wd_inject::SERVER_PORT`].
//! Refs: `.devdocs/phantom/phantom/src/waydroid.rs:101-154` (vendor uses
//! `sh -c` there; this port keeps the same launch semantics as pure argv),
//! `.devdocs/phantom/phantom/src/android_inject.rs:130-148`,
//! `java/PhantomServer.java`.

/// Device-side jar path (`adb push <host-jar>` target).
pub const SERVER_JAR_DEVICE_PATH: &str = "/data/local/tmp/ghostdroid-server.jar";

/// Device-side log path (log excerpt reads this; launch does not redirect —
/// no shell here, callers capture stdout instead).
pub const SERVER_LOG_DEVICE_PATH: &str = "/data/local/tmp/ghostdroid-server.log";

/// Server class (matches `java/PhantomServer.java` package).
pub const SERVER_CLASS: &str = "com.ghostdroid.server.PhantomServer";

/// Container bind host (server listens on all interfaces).
pub const SERVER_BIND_HOST: &str = "0.0.0.0";

/// `app_process` system binary inside the container.
pub const APP_PROCESS_BIN: &str = "app_process";

/// `app_process` base path arg.
pub const APP_PROCESS_BASE: &str = "/system/bin";

/// Build `adb push <host-jar> <device-path>` args (no `adb` prefix).
#[must_use]
pub fn push_args(host_jar: &str) -> Vec<String> {
    tracing::info!(host_jar, "android-server: push args");
    let args = vec![
        "push".to_owned(),
        host_jar.to_owned(),
        SERVER_JAR_DEVICE_PATH.to_owned(),
    ];
    tracing::debug!(?args, "android-server: push args built");
    args
}

/// Build `waydroid shell -- env CLASSPATH=<jar> app_process /system/bin
/// <class> --host <bind> --port <port>` args (no `waydroid` prefix).
///
/// Pure argv: no `sh -c`, no string concat. `CLASSPATH` rides via `env`
/// (direct exec has no shell to expand `VAR=...` prefixes).
#[must_use]
pub fn launch_args(bind_host: &str, port: u16) -> Vec<String> {
    tracing::info!(bind_host, port, "android-server: launch args");
    let args = vec![
        "shell".to_owned(),
        "--".to_owned(),
        "env".to_owned(),
        format!("CLASSPATH={SERVER_JAR_DEVICE_PATH}"),
        APP_PROCESS_BIN.to_owned(),
        APP_PROCESS_BASE.to_owned(),
        SERVER_CLASS.to_owned(),
        "--host".to_owned(),
        bind_host.to_owned(),
        "--port".to_owned(),
        port.to_string(),
    ];
    tracing::debug!(?args, "android-server: launch args built");
    args
}

/// Build `waydroid shell -- tail -n 80 <log>` args (log excerpt, pure argv).
///
/// Reads the file the detached launcher's captured stdout creates
/// (see [`launch_detached_args`]). Use [`log_args_for`] for a custom path.
#[must_use]
pub fn log_args() -> Vec<String> {
    log_args_for(SERVER_LOG_DEVICE_PATH)
}

/// Build `waydroid shell -- tail -n 80 <log>` for an explicit log path.
#[must_use]
pub fn log_args_for(log_path: &str) -> Vec<String> {
    tracing::info!(log_path, "android-server: log args");
    vec![
        "shell".to_owned(),
        "--".to_owned(),
        "tail".to_owned(),
        "-n".to_owned(),
        "80".to_owned(),
        log_path.to_owned(),
    ]
}

/// Detached launch descriptor: pure argv plus explicit log + detach flag.
///
/// Vendor backgrounds via `sh -c 'rm -f log; CLASSPATH=.. app_process … >log
/// 2>&1 &'`. This port keeps zero shell: `argv` is the same pure argv as
/// [`launch_args`], and callers detach (spawn without waiting) and capture
/// the child stdout into `log_path` instead of shell `>` redirect. No `;`,
/// `&&`, `|` or `$()` ever appears, so argv injection is impossible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetachedLaunch {
    /// Pure argv (no `waydroid` prefix), same shape as [`launch_args`].
    pub argv: Vec<String>,
    /// Device log path the caller captures stdout into.
    pub log_path: String,
    /// Always true: caller must spawn detached (no wait).
    pub detached: bool,
}

/// Build a detached launch (pure argv + explicit log path + detach flag).
#[must_use]
pub fn launch_detached_args(bind_host: &str, port: u16) -> DetachedLaunch {
    tracing::info!(bind_host, port, "android-server: detached launch args");
    DetachedLaunch {
        argv: launch_args(bind_host, port),
        log_path: SERVER_LOG_DEVICE_PATH.to_owned(),
        detached: true,
    }
}

/// Parse a ping reply: up iff the server echoed the single `0x7f` byte.
///
/// Pure port of `android_inject.rs` ping check (`reply[0] != CMD_PING`).
#[must_use]
pub fn is_server_up(reply: &[u8]) -> bool {
    tracing::debug!(len = reply.len(), "android-server: parse ping reply");
    let up = reply == [wd_inject::TAG_PING];
    if !up {
        tracing::warn!(len = reply.len(), "android-server: bad ping reply");
    }
    up
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_shape() {
        assert_eq!(
            push_args("/tmp/ghostdroid-server.jar"),
            vec!["push", "/tmp/ghostdroid-server.jar", SERVER_JAR_DEVICE_PATH]
        );
    }

    #[test]
    fn launch_shape_is_pure_argv() {
        let args = launch_args(SERVER_BIND_HOST, wd_inject::SERVER_PORT);
        assert_eq!(
            args,
            vec![
                "shell",
                "--",
                "env",
                "CLASSPATH=/data/local/tmp/ghostdroid-server.jar",
                "app_process",
                "/system/bin",
                SERVER_CLASS,
                "--host",
                SERVER_BIND_HOST,
                "--port",
                "27183",
            ]
        );
        assert!(args.iter().all(|a| !a.contains("&&") && !a.contains(';')));
    }

    #[test]
    fn launch_port_param_flows() {
        let args = launch_args(SERVER_BIND_HOST, 1234);
        assert_eq!(args.last().map(String::as_str), Some("1234"));
    }

    #[test]
    fn log_shape() {
        assert_eq!(
            log_args(),
            vec!["shell", "--", "tail", "-n", "80", SERVER_LOG_DEVICE_PATH]
        );
        assert_eq!(
            log_args_for("/tmp/x.log"),
            vec!["shell", "--", "tail", "-n", "80", "/tmp/x.log"]
        );
    }

    #[test]
    fn detached_is_pure_argv_with_log_and_flag() {
        let d = launch_detached_args(SERVER_BIND_HOST, wd_inject::SERVER_PORT);
        assert!(d.detached);
        assert_eq!(d.log_path, SERVER_LOG_DEVICE_PATH);
        assert_eq!(
            d.argv,
            launch_args(SERVER_BIND_HOST, wd_inject::SERVER_PORT)
        );
        assert!(d.argv.iter().all(|a| {
            !a.contains("&&") && !a.contains(';') && !a.contains('|') && !a.contains("$(")
        }));
        assert_eq!(log_args_for(&d.log_path), log_args());
    }

    #[test]
    fn ping_parse() {
        assert!(is_server_up(&[wd_inject::TAG_PING]));
        assert!(!is_server_up(&[]));
        assert!(!is_server_up(&[0x00]));
        assert!(!is_server_up(&[0x7f, 0x00]));
    }
}
