//! Android touch server push/run: arg builders plus ping parse.
//!
//! Pure builders + parsers only. No spawn, no sleep, no socket.
//! Spawn lives in sync/exec callers (`wd_waydroid::exec::run_waydroid`,
//! adb/waydroid-shell callers). Port re-uses [`wd_inject::SERVER_PORT`].
//! Refs: `.devdocs/phantom/phantom/src/waydroid.rs:101-154`,
//! `.devdocs/phantom/phantom/src/android_inject.rs:130-148`,
//! `java/PhantomServer.java`.

/// Device-side jar path (`adb push <host-jar>` target).
pub const SERVER_JAR_DEVICE_PATH: &str = "/data/local/tmp/ghostdroid-server.jar";

/// Device-side log path (launch redirects stdout/stderr here).
pub const SERVER_LOG_DEVICE_PATH: &str = "/data/local/tmp/ghostdroid-server.log";

/// Server class (matches `java/PhantomServer.java` package).
pub const SERVER_CLASS: &str = "com.ghostdroid.server.PhantomServer";

/// Container bind host (server listens on all interfaces).
pub const SERVER_BIND_HOST: &str = "0.0.0.0";

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

/// Build `waydroid shell -- sh -c "<launch line>"` args (no `waydroid` prefix).
///
/// Launch line mirrors vendor: `rm -f log; CLASSPATH=jar app_process /
/// <class> --host <bind> --port <port> </dev/null >log 2>&1 &`.
#[must_use]
pub fn launch_args(bind_host: &str, port: u16) -> Vec<String> {
    tracing::info!(bind_host, port, "android-server: launch args");
    let shell = format!(
        "rm -f {log}; CLASSPATH={jar} app_process / {class} --host {host} --port {port} </dev/null >{log} 2>&1 &",
        jar = sh_quote(SERVER_JAR_DEVICE_PATH),
        class = sh_quote(SERVER_CLASS),
        host = sh_quote(bind_host),
        log = sh_quote(SERVER_LOG_DEVICE_PATH),
    );
    let args = vec![
        "shell".to_owned(),
        "--".to_owned(),
        "sh".to_owned(),
        "-c".to_owned(),
        shell,
    ];
    tracing::debug!("android-server: launch args built");
    args
}

/// Build `waydroid shell -- sh -c "tail -n 80 <log>"` args (log excerpt).
#[must_use]
pub fn log_args() -> Vec<String> {
    tracing::info!("android-server: log args");
    vec![
        "shell".to_owned(),
        "--".to_owned(),
        "sh".to_owned(),
        "-c".to_owned(),
        format!("tail -n 80 {}", sh_quote(SERVER_LOG_DEVICE_PATH)),
    ]
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

/// Quote a shell word with single quotes (pure port of vendor `sh_quote`).
#[must_use]
pub fn sh_quote(value: &str) -> String {
    let mut out = String::from("'");
    for ch in value.chars() {
        if ch == '\'' {
            out.push_str("'\"'\"'");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
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
    fn launch_shape_uses_vendor_line() {
        let args = launch_args(SERVER_BIND_HOST, wd_inject::SERVER_PORT);
        let head: Vec<&str> = args.iter().take(4).map(String::as_str).collect();
        assert_eq!(head, ["shell", "--", "sh", "-c"]);
        assert!(args[4].contains("CLASSPATH="));
        assert!(args[4].contains("app_process /"));
        assert!(args[4].contains(SERVER_CLASS));
        assert!(args[4].contains("--port 27183"));
    }

    #[test]
    fn log_shape() {
        let args = log_args();
        let head: Vec<&str> = args.iter().take(4).map(String::as_str).collect();
        assert_eq!(head, ["shell", "--", "sh", "-c"]);
        assert!(args[4].starts_with("tail -n 80 "));
    }

    #[test]
    fn ping_parse() {
        assert!(is_server_up(&[wd_inject::TAG_PING]));
        assert!(!is_server_up(&[]));
        assert!(!is_server_up(&[0x00]));
        assert!(!is_server_up(&[0x7f, 0x00]));
    }

    #[test]
    fn quote_escapes_tick() {
        assert_eq!(sh_quote("a'b"), "'a'\"'\"'b'");
        assert_eq!(
            sh_quote(SERVER_JAR_DEVICE_PATH),
            format!("'{SERVER_JAR_DEVICE_PATH}'")
        );
    }
}
