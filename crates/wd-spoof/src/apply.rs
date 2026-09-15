//! Houdini-safe apply preview.
//!
//! Pure snapshot + render + merge + verify, no spawn, no write. Caller owns
//! `pkexec` write + session restart. Live bridge `libhoudini.so` survives.

use std::collections::HashSet;

/// Host keys never dropped unless rendered sets the same key.
const PRESERVE: &[&str] = &[
    "houdini",
    "native.bridge",
    "neural",
    "vulkan",
    "gralloc",
    "egl",
];

/// Split `key=value`; `None` for blank/`#`/keyless lines.
fn split_kv(line: &str) -> Option<(&str, &str)> {
    let t = line.trim();
    if t.is_empty() || t.starts_with('#') {
        return None;
    }
    t.split_once('=').map(|(k, v)| (k.trim(), v.trim()))
}

/// True when key or value needs host preservation (bridge/abilist lines).
fn preserved(key: &str, value: &str) -> bool {
    let low = format!("{key}={value}").to_ascii_lowercase();
    let hit =
        PRESERVE.iter().any(|p| low.contains(p)) || low.contains("houdini") || low.contains("x86");
    tracing::debug!(key, hit, "spoof/apply: preserve check");
    hit
}

/// Union comma tokens, rendered first, host extras appended.
fn union_abilist(rendered_val: &str, host_val: &str) -> String {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<String> = Vec::new();
    for tok in rendered_val
        .split(',')
        .chain(host_val.split(','))
        .map(str::trim)
    {
        if !tok.is_empty() && seen.insert(tok.to_owned()) {
            out.push(tok.to_owned());
        }
    }
    tracing::info!(merged = %out.join(","), "spoof/apply: abilist union");
    out.join(",")
}

/// Merge rendered over snapshot body, keeping host bridge lines.
/// Rendered keys win except `ro.product.cpu.abilist`, unioned with host
/// (live `x86_64,x86` kept). Preserved host keys append when absent.
#[must_use]
pub fn merge_lines(snapshot_body: &str, rendered: &[String]) -> Vec<String> {
    tracing::info!("spoof/apply: merge start");
    let host: Vec<(String, String)> = snapshot_body
        .lines()
        .filter_map(|l| split_kv(l).map(|(k, v)| (k.to_owned(), v.to_owned())))
        .collect();
    let mut out: Vec<String> = rendered
        .iter()
        .map(|line| {
            let Some((key, val)) = split_kv(line) else {
                return line.clone();
            };
            if key == "ro.product.cpu.abilist"
                && let Some((_, host_val)) = host.iter().find(|(k, _)| k == key)
            {
                let merged = union_abilist(val, host_val);
                return format!("{key}={merged}");
            }
            line.clone()
        })
        .collect();
    let have: HashSet<String> = out
        .iter()
        .filter_map(|l| split_kv(l).map(|(k, _)| k.to_owned()))
        .collect();
    for (key, value) in &host {
        if preserved(key, value) && !have.contains(key) {
            let line = format!("{key}={value}");
            tracing::info!(key, "spoof/apply: keep host line");
            out.push(line);
        }
    }
    tracing::info!(count = out.len(), "spoof/apply: merge done");
    out
}

/// Verify `getprop` output contains the wanted fingerprint. Pure.
#[must_use]
pub fn verify_fingerprint(getprop_out: &str, want: &str) -> bool {
    tracing::info!("spoof/apply: verify start");
    let want = want.trim();
    let ok = !want.is_empty() && getprop_out.contains(want);
    tracing::info!(ok, "spoof/apply: verify done");
    ok
}

#[cfg(test)]
mod tests {
    use super::*;

    const SNAP: &str = "ro.build.fingerprint=old\n\
        ro.product.cpu.abilist=x86_64,x86,arm64-v8a\n\
        ro.dalvik.vm.native.bridge=libhoudini.so\n\
        ro.hardware.vulkan=default\n";

    #[test]
    fn keeps_bridge_lines() {
        let rendered = vec!["ro.build.fingerprint=new".to_owned()];
        let merged = merge_lines(SNAP, &rendered);
        assert_eq!(merged.len(), 4);
        assert!(merged.iter().any(|l| l.contains("libhoudini.so")));
        assert!(merged.iter().any(|l| l.contains("x86_64,x86")));
        assert!(merged.iter().any(|l| l.contains("vulkan")));
    }

    #[test]
    fn rendered_wins_same_key() {
        let rendered = vec!["ro.dalvik.vm.native.bridge=libnb.so".to_owned()];
        let merged = merge_lines(SNAP, &rendered);
        assert_eq!(merged.len(), 3);
        assert!(!merged.iter().any(|l| l.contains("libhoudini.so")));
        assert!(merged.iter().any(|l| l.contains("x86_64,x86")));
    }

    #[test]
    fn abilist_unions_host_x86() {
        let rendered = vec!["ro.product.cpu.abilist=arm64-v8a,armeabi-v7a".to_owned()];
        let merged = merge_lines(SNAP, &rendered);
        assert_eq!(merged.len(), 3);
        let abi = merged
            .iter()
            .find(|l| l.starts_with("ro.product.cpu.abilist="))
            .expect("abilist kept");
        assert!(abi.contains("arm64-v8a"));
        assert!(abi.contains("x86_64,x86"));
    }

    #[test]
    fn verify_contains() {
        assert!(verify_fingerprint("samsung/m3q:user\n", "samsung/m3q:user"));
        assert!(!verify_fingerprint("old\n", "new"));
        assert!(!verify_fingerprint("anything", ""));
    }
}
