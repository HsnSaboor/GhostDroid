//! Fake `persist.waydroid.fake_touch / fake_wifi` glob match.
//!
//! Refs: `.research/03-tech-stack.md:87,97`
//! (per-package fake_* forward),
//! `.devdocs/cage-xtmapper/README.md:35-38`
//! (`fake_touch ""` to disable),
//! `.devdocs/waydroid_stuff/weston/README.md:86`
//! (comma globs e.g. `com.HoYoverse.*,com.miHoYo.*`).

/// Match `*`-only glob (`prefix*` or exact). No regex dep.
#[must_use]
pub fn glob_match(pattern: &str, pkg: &str) -> bool {
    tracing::debug!(pattern, pkg, "spoof: glob check");
    let hit = pattern == "*"
        || pattern == pkg
        || pattern
            .strip_suffix('*')
            .is_some_and(|pre| pkg.starts_with(pre));
    tracing::debug!(hit, "spoof: glob result");
    hit
}

/// True when any glob in list matches pkg.
#[must_use]
pub fn list_match(globs: &[String], pkg: &str) -> bool {
    tracing::info!(pkg, count = globs.len(), "spoof: list match start");
    let hit = globs.iter().any(|glob| glob_match(glob, pkg));
    tracing::info!(hit, "spoof: list match done");
    hit
}

/// Build `waydroid prop set persist.waydroid.fake_touch <csv>` args.
///
/// DRY: key string from `wd_waydroid::PROP_KEYS`, args via `set_args`.
#[must_use]
pub fn touch_set_args(globs: &[String]) -> Vec<String> {
    let key = wd_waydroid::PROP_KEYS[0];
    tracing::info!(key, "spoof: touch set args");
    wd_waydroid::set_args(key, &globs.join(","))
}

/// Build `waydroid prop set persist.waydroid.fake_wifi <csv>` args.
#[must_use]
pub fn wifi_set_args(globs: &[String]) -> Vec<String> {
    let key = wd_waydroid::PROP_KEYS[1];
    tracing::info!(key, "spoof: wifi set args");
    wd_waydroid::set_args(key, &globs.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_shapes() {
        assert!(glob_match("*", "com.example.game"));
        assert!(glob_match("com.example.*", "com.example.game"));
        assert!(glob_match("com.example.game", "com.example.game"));
        assert!(!glob_match("com.other.*", "com.example.game"));
    }

    #[test]
    fn list_and_args() {
        let globs = vec!["com.example.game*".to_owned()];
        assert!(list_match(&globs, "com.example.game2"));
        assert!(!list_match(&globs, "com.other"));
        assert_eq!(
            touch_set_args(&globs),
            vec![
                "prop",
                "set",
                "persist.waydroid.fake_touch",
                "com.example.game*"
            ]
        );
        assert_eq!(
            wifi_set_args(&globs),
            vec![
                "prop",
                "set",
                "persist.waydroid.fake_wifi",
                "com.example.game*"
            ]
        );
    }
}
