//! Logcat pattern doc + threadtime parser.
//!
//! Best port: `adb-mcp/tools/logs.py`: `logcat_start` Popen
//! `adb -s logcat -v threadtime` → temp file; `dump{tags,
//! priority,message_contains,tail}` → `{time,pid,tid,level,
//! tag,message}` VDIWEF order; `get_crash_log(-b crash)`;
//! `get_anr_traces`; `capture_bugreport`. One-shot fallback:
//! `us/tools/logcat.ts` (`-d -t N`, `tag:V *:S`), `mg`
//! `get_logs{package,level,lines=200,since}`. Redact tokens opt-in.

/// Priority order string (VDIWEF).
pub const PRIORITY_ORDER: &str = "VDIWEF";

/// One parsed threadtime record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRecord {
    /// `MM-DD HH:MM:SS.mmm`.
    pub time: String,
    /// PID.
    pub pid: u32,
    /// TID.
    pub tid: u32,
    /// Level char.
    pub level: char,
    /// Tag.
    pub tag: String,
    /// Message.
    pub message: String,
}

/// One-shot dump argv (`-d -t N`, filters appended by caller).
#[must_use]
pub fn dump_args(lines: u32) -> Vec<String> {
    tracing::debug!(lines, "logcat: dump args");
    vec![
        "logcat".into(),
        "-d".into(),
        "-v".into(),
        "threadtime".into(),
        "-t".into(),
        lines.to_string(),
    ]
}

/// Bg capture argv (`logcat_start` port — caller spawns).
#[must_use]
pub fn start_args(buffers: Option<&str>) -> Vec<String> {
    tracing::info!(?buffers, "logcat: start args");
    let mut a = vec!["logcat".into(), "-v".into(), "threadtime".into()];
    if let Some(b) = buffers.filter(|b| !b.is_empty()) {
        a.push("-b".into());
        a.push(b.into());
    }
    a
}

/// Parse one threadtime line; `None` on separators/short lines.
#[must_use]
pub fn parse_line(line: &str) -> Option<LogRecord> {
    let line = line.trim_end();
    if line.is_empty() || line.starts_with("---------") {
        return None;
    }
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 6 {
        return None;
    }
    let (date, tm, pid, tid, level) = (parts[0], parts[1], parts[2], parts[3], parts[4]);
    if !pid.bytes().all(|b| b.is_ascii_digit())
        || !tid.bytes().all(|b| b.is_ascii_digit())
        || level.len() != 1
    {
        return None;
    }
    let rest = parts[5..].join(" ");
    let (tag, message) = rest.split_once(':').unwrap_or((rest.as_str(), ""));
    Some(LogRecord {
        time: format!("{date} {tm}"),
        pid: pid.parse().ok()?,
        tid: tid.parse().ok()?,
        tag: tag.trim().to_owned(),
        message: message.trim().to_owned(),
        level: level.chars().next()?,
    })
}

/// Min-priority filter (VDIWEF order).
#[must_use]
pub fn at_least(level: char, min: char) -> bool {
    match (PRIORITY_ORDER.find(min), PRIORITY_ORDER.find(level)) {
        (Some(w), Some(h)) => h >= w,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn threadtime_roundtrip() {
        let r = parse_line("09-14 10:00:00.123  1234  5678 I MyTag: hello").unwrap();
        assert_eq!((r.pid, r.tid, r.level), (1234, 5678, 'I'));
        assert!(parse_line("--------- beginning").is_none());
        assert!(at_least('E', 'W') && !at_least('D', 'W'));
    }
}
