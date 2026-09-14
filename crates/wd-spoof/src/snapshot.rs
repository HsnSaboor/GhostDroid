//! Snapshot/restore of `waydroid_base.prop`. No spawn.
//!
//! Ref: `.devdocs/waydroid/tools/helpers/props.py:file_get`
//! (build.prop parser skip `#`/blank),
//! plan 04: snapshot before swap, restore on rollback.

use std::path::Path;

/// Snapshot of base prop for rollback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropSnapshot {
    /// Source path.
    pub path: String,
    /// Raw body.
    pub body: String,
}

/// Read file into snapshot. Pure IO, no spawn.
///
/// # Errors
/// Returns [`wd_core::WdError`] on IO failure.
pub fn snapshot(path: &Path) -> wd_core::Result<PropSnapshot> {
    tracing::info!(path = %path.display(), "spoof: snapshot start");
    let body = std::fs::read_to_string(path)?;
    tracing::info!(bytes = body.len(), "spoof: snapshot done");
    Ok(PropSnapshot {
        path: path.display().to_string(),
        body,
    })
}

/// Write snapshot body back. Rollback path.
///
/// # Errors
/// Returns [`wd_core::WdError`] on IO failure.
pub fn restore(snapshot: &PropSnapshot) -> wd_core::Result<()> {
    tracing::info!(path = %snapshot.path, bytes = snapshot.body.len(), "spoof: restore start");
    std::fs::write(&snapshot.path, &snapshot.body)?;
    tracing::info!("spoof: restore done");
    Ok(())
}

/// Parse `key=value` lines, skipping blanks + `#` (props.py `file_get`).
#[must_use]
pub fn parse_kv(body: &str) -> Vec<(String, String)> {
    tracing::debug!("spoof: parse kv start");
    let pairs: Vec<(String, String)> = body
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| {
            let (key, value) = line.split_once('=')?;
            tracing::debug!(key, "spoof: parse kv line");
            Some((key.trim().to_owned(), value.trim().to_owned()))
        })
        .collect();
    tracing::debug!(count = pairs.len(), "spoof: parse kv done");
    pairs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_skips_comments() {
        let body = "# c\n\npersist.waydroid.width=1920\nro.build.fingerprint=x\n";
        let pairs = parse_kv(body);
        assert_eq!(pairs.len(), 2);
        assert_eq!(
            pairs[0],
            ("persist.waydroid.width".to_owned(), "1920".to_owned())
        );
    }

    #[test]
    fn snapshot_restore_tmp() {
        let dir = std::env::temp_dir().join("wd-spoof-test");
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("waydroid_base.prop");
        std::fs::write(&path, "a=1\n").expect("write");
        let snap = snapshot(&path).expect("snapshot");
        assert_eq!(snap.body, "a=1\n");
        std::fs::write(&path, "b=2\n").expect("overwrite");
        restore(&snap).expect("restore");
        assert_eq!(std::fs::read_to_string(&path).expect("read"), "a=1\n");
        std::fs::remove_dir_all(&dir).ok();
    }
}
