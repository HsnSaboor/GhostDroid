//! Template renderer: profile -> `waydroid_base.prop` lines.
//!
//! Refs: `.devdocs/waydroid-settings/usr/lib/waydroid-settings/utils.py:18,80`
//! (`BASE_PROP_LOC` + `tee -a` append),
//! `.devdocs/Waydroid-total-spoof/waydroid.sh:78` (append flow),
//! `.research/03-tech-stack.md:117` (canonical path).
//!
//! DRY: keys via `wd_waydroid::{BASE_PROP, PROP_KEYS, is_managed}`.

use crate::profile::SpoofProfile;

pub use wd_waydroid::BASE_PROP;

/// Render profile into append-ready `key=value` lines.
#[must_use]
pub fn render(profile: &SpoofProfile) -> Vec<String> {
    tracing::info!(
        fingerprint = %profile.fingerprint,
        "spoof: render start"
    );
    let touch = profile.fake_touch.join(",");
    let wifi = profile.fake_wifi.join(",");
    tracing::debug!(touch = %touch, wifi = %wifi, "spoof: globs joined");
    let lines = vec![
        format!("ro.build.fingerprint={}", profile.fingerprint),
        format!("persist.waydroid.fake_touch={touch}"),
        format!("persist.waydroid.fake_wifi={wifi}"),
    ];
    for line in &lines {
        let key = line.split('=').next().unwrap_or("");
        // ponytail: reuse managed-key check; fingerprint is build prop (warn ok).
        let managed = wd_waydroid::is_managed(key);
        tracing::info!(line = %line, managed, "spoof: render line");
    }
    tracing::info!(count = lines.len(), path = BASE_PROP, "spoof: render done");
    lines
}

/// Diff rendered lines vs snapshot body. Pure, no IO.
#[must_use]
pub fn swap_diff(snapshot_body: &str, rendered: &[String]) -> Vec<String> {
    tracing::info!("spoof: swap diff start");
    let diff: Vec<String> = rendered
        .iter()
        .filter(|line| {
            let hit = !snapshot_body.lines().any(|old| old.trim() == line.trim());
            tracing::debug!(line = %line, new = hit, "spoof: diff line");
            hit
        })
        .cloned()
        .collect();
    tracing::info!(count = diff.len(), "spoof: swap diff done");
    diff
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn example() -> SpoofProfile {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../profiles/spoof/example.toml");
        SpoofProfile::load(&path).expect("example loads")
    }

    #[test]
    fn renders_three_lines() {
        let lines = render(&example());
        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with("ro.build.fingerprint=google/cheetah"));
        assert!(lines[1].starts_with("persist.waydroid.fake_touch="));
        assert!(lines[2].starts_with("persist.waydroid.fake_wifi="));
    }

    #[test]
    fn diff_finds_new_lines() {
        let lines = render(&example());
        let diff = swap_diff("", &lines);
        assert_eq!(diff.len(), 3);
        let body = lines.join("\n");
        assert!(swap_diff(&body, &lines).is_empty());
    }
}
