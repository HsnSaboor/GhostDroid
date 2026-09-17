//! Template renderer: profile -> `waydroid_base.prop` lines.
//!
//! Refs: `.devdocs/waydroid-settings/usr/lib/waydroid-settings/utils.py:18,80`
//! (`BASE_PROP_LOC` + `tee -a` append),
//! `.devdocs/Waydroid-total-spoof/waydroid.sh:78` (append flow),
//! `.devdocs/DeviceSpoofLab-Hooks/device_profile.conf:138-148` (arm64/hw),
//! `.research/03-tech-stack.md:117` (canonical path).
//!
//! DRY: keys via `wd_waydroid::{BASE_PROP, PROP_KEYS, is_managed}`.

use crate::profile::SpoofProfile;

pub use wd_waydroid::BASE_PROP;

/// Verified-boot green tail (never userdebug/test-keys).
const GREEN: &[(&str, &str)] = &[
    ("ro.boot.verifiedbootstate", "green"),
    ("ro.boot.flash.locked", "1"),
    ("ro.boot.vbmeta.device_state", "locked"),
];

/// Push `key=value` when value non-empty.
fn push_if(lines: &mut Vec<String>, key: &str, value: &str) {
    if !value.trim().is_empty() {
        lines.push(format!("{key}={value}"));
    }
}

/// Render profile into append-ready `key=value` lines.
#[must_use]
pub fn render(profile: &SpoofProfile) -> Vec<String> {
    tracing::info!(fingerprint = %profile.fingerprint, "spoof: render start");
    let touch = profile.fake_touch.join(",");
    let wifi = profile.fake_wifi.join(",");
    tracing::debug!(touch = %touch, wifi = %wifi, "spoof: globs joined");
    let mut lines = vec![
        format!("ro.build.fingerprint={}", profile.fingerprint),
        format!("persist.waydroid.fake_touch={touch}"),
        format!("persist.waydroid.fake_wifi={wifi}"),
    ];
    // Partition fingerprints must match ro.build.fingerprint or
    // Build.isBuildConsistent() fires the "internal problem with your
    // device" popup (2026-09-17: main fp Samsung, partitions Waydroid).
    for key in [
        "ro.system.build.fingerprint",
        "ro.system_ext.build.fingerprint",
        "ro.vendor.build.fingerprint",
        "ro.vendor_dlkm.build.fingerprint",
        "ro.odm.build.fingerprint",
    ] {
        lines.push(format!("{key}={}", profile.fingerprint));
    }
    push_if(&mut lines, "ro.product.model", &profile.model);
    push_if(&mut lines, "ro.product.name", &profile.product);
    push_if(&mut lines, "ro.product.device", &profile.device);
    // NEVER render `ro.hardware`: the graphics HAL reads it at boot and
    // `m3q` (a phone SoC string) kills hwcomposer+surfaceflinger (SIGABRT
    // bootloop, 2026-09-16). Host keeps `unknown`; Build.HARDWARE spoofing
    // belongs in a Vector/Zygisk module, not base.prop.
    push_if(&mut lines, "ro.product.cpu.abi", &profile.cpu_abi);
    push_if(&mut lines, "ro.product.cpu.abilist", &profile.cpu_abilist);
    push_if(&mut lines, "ro.build.tags", &profile.build_tags);
    push_if(&mut lines, "ro.build.type", &profile.build_type);
    for (key, value) in GREEN {
        lines.push(format!("{key}={value}"));
    }
    for line in &lines {
        let key = line.split('=').next().unwrap_or("");
        // ponytail: reuse managed-key check; build props warn ok.
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

    fn load(name: &str) -> SpoofProfile {
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../profiles/spoof");
        SpoofProfile::load(&base.join(name)).expect("profile loads")
    }

    #[test]
    fn renders_three_lines() {
        let lines = render(&load("example.toml"));
        assert_eq!(lines.len(), 11);
        assert!(lines[0].starts_with("ro.build.fingerprint=google/cheetah"));
        assert!(lines[1].starts_with("persist.waydroid.fake_touch="));
        assert!(lines[2].starts_with("persist.waydroid.fake_wifi="));
        assert!(lines[3..].contains(&"ro.boot.verifiedbootstate=green".to_owned()));
    }

    #[test]
    fn s26_renders_no_x86_leak() {
        let lines = render(&load("s26-ultra.toml"));
        assert_eq!(lines.len(), 18);
        // Partition fingerprints carry the Samsung fp (no x86/houdini).
        for key in [
            "ro.system.build.fingerprint",
            "ro.system_ext.build.fingerprint",
            "ro.vendor.build.fingerprint",
            "ro.vendor_dlkm.build.fingerprint",
            "ro.odm.build.fingerprint",
        ] {
            let line = lines.iter().find(|l| l.starts_with(key)).expect(key);
            assert!(line.contains("m3qxeea/m3q"), "{line}");
        }
        assert!(lines.contains(&"ro.product.model=SM-S948B".to_owned()));
        assert!(lines.contains(&"ro.product.cpu.abi=arm64-v8a".to_owned()));
        assert!(lines.contains(&"ro.build.tags=release-keys".to_owned()));
        assert!(!lines.iter().any(|l| l.starts_with("ro.hardware=")));
        assert!(
            !lines
                .iter()
                .any(|l| l.contains("x86") || l.contains("houdini"))
        );
    }

    #[test]
    fn diff_finds_new_lines() {
        let lines = render(&load("example.toml"));
        let diff = swap_diff("", &lines);
        assert_eq!(diff.len(), 11);
        let body = lines.join("\n");
        assert!(swap_diff(&body, &lines).is_empty());
    }
}
