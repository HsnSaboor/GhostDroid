//! Prop get/set arg builders over stock `waydroid prop`.
//!
//! Refs: `.devdocs/waydroid/tools/helpers/arguments.py:93-101`,
//! `.devdocs/waydroid/tools/actions/prop.py`.

/// Prop keys managed per game profile.
pub const PROP_KEYS: &[&str] = &[
    "persist.waydroid.fake_touch",
    "persist.waydroid.fake_wifi",
    "persist.waydroid.multi_windows",
    "persist.waydroid.width",
    "persist.waydroid.height",
    "persist.waydroid.suspend",
    "persist.waydroid.uevent",
];

/// Canonical base prop file swapped per game (04 owns swap).
pub const BASE_PROP: &str = "/var/lib/waydroid/waydroid_base.prop";

/// Managed-key check. Warns on unmanaged keys for fast debug.
#[must_use]
pub fn is_managed(key: &str) -> bool {
    tracing::debug!(key, "props: managed check");
    let hit = PROP_KEYS.contains(&key);
    if hit {
        tracing::debug!(key, "props: managed key");
    } else {
        tracing::warn!(key, "props: unmanaged key");
    }
    hit
}

/// Build `waydroid prop get <key>` args.
#[must_use]
pub fn get_args(key: &str) -> Vec<String> {
    tracing::info!(key, "props: get args");
    vec!["prop".to_owned(), "get".to_owned(), key.to_owned()]
}

/// Build `waydroid prop set <key> <value>` args.
#[must_use]
pub fn set_args(key: &str, value: &str) -> Vec<String> {
    tracing::info!(key, value, "props: set args");
    vec![
        "prop".to_owned(),
        "set".to_owned(),
        key.to_owned(),
        value.to_owned(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shapes_and_managed() {
        assert_eq!(
            get_args("persist.waydroid.width"),
            vec!["prop", "get", "persist.waydroid.width"]
        );
        assert_eq!(
            set_args("persist.waydroid.width", "1920"),
            vec!["prop", "set", "persist.waydroid.width", "1920"]
        );
        assert!(is_managed("persist.waydroid.fake_touch"));
        assert!(!is_managed("ro.build.fingerprint"));
    }
}
