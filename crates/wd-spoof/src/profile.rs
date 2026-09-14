//! Per-game spoof profile (TOML).
//!
//! Ref: `.devdocs/DeviceSpoofLab-Hooks/device_profile.conf:260`
//! (cheetah canonical), `.research/04-root-hide-spoof-2026.md:40`
//! (discard redfin print, broken sed).

use std::path::Path;

use serde::{Deserialize, Serialize};

/// Stack preset: minimal vs full hide stack. Traced on load.
/// Default is [`StackPreset::Full`] — gaming-full is the default root/spoof profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum StackPreset {
    /// Built-in `Zygisk` + `Shamiko` + `PIFork` only.
    Basic,
    /// Full stack: `TrickyStore` + Vector shims + `HMA`-on-kick. DEFAULT.
    #[default]
    Full,
}

/// Canonical per-game spoof profile. One schema only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpoofProfile {
    /// `ro.build.fingerprint` value (cheetah A16).
    pub fingerprint: String,
    /// `persist.waydroid.fake_touch` package globs.
    pub fake_touch: Vec<String>,
    /// `persist.waydroid.fake_wifi` package globs.
    pub fake_wifi: Vec<String>,
    /// Denylist packages (applist-hide ref, 05 owns policy).
    pub denylist: Vec<String>,
    /// Stack preset; defaults `full` (gaming-full is the default profile).
    #[serde(default)]
    pub stack: StackPreset,
}

/// Template keys required per game profile.
pub const REQUIRED_KEYS: &[&str] = &[
    "fingerprint",
    "fake_touch",
    "fake_wifi",
    "denylist",
    "stack",
];

/// Check a key list covers [`REQUIRED_KEYS`].
#[must_use]
pub fn has_keys(keys: &[&str]) -> bool {
    tracing::debug!(?keys, "spoof: keys check");
    let ok = REQUIRED_KEYS.iter().all(|k| keys.contains(k));
    tracing::debug!(ok, "spoof: keys result");
    ok
}

impl SpoofProfile {
    /// Load + validate from TOML path.
    ///
    /// # Errors
    /// Returns [`wd_core::WdError`] on IO, TOML, or validation failure.
    pub fn load(path: &Path) -> wd_core::Result<Self> {
        tracing::info!(path = %path.display(), "spoof: load profile");
        let raw = std::fs::read_to_string(path)?;
        let profile: Self = toml::from_str(&raw).map_err(wd_core::WdError::Toml)?;
        profile.validate()?;
        tracing::info!(
            fingerprint = %profile.fingerprint,
            touch = profile.fake_touch.len(),
            wifi = profile.fake_wifi.len(),
            stack = ?profile.stack,
            "spoof: loaded"
        );
        Ok(profile)
    }

    /// Validate: non-empty fingerprint, non-empty glob lists.
    ///
    /// # Errors
    /// Returns [`wd_core::WdError::Validation`] when empty.
    pub fn validate(&self) -> wd_core::Result<()> {
        tracing::debug!(fingerprint = %self.fingerprint, "spoof: validate");
        if self.fingerprint.trim().is_empty() {
            tracing::warn!("spoof: empty fingerprint");
            return Err(wd_core::WdError::Validation("empty fingerprint".to_owned()));
        }
        if self.fake_touch.is_empty() || self.fake_wifi.is_empty() {
            tracing::warn!("spoof: empty fake_* globs");
            return Err(wd_core::WdError::Validation(
                "empty fake_* globs".to_owned(),
            ));
        }
        tracing::debug!("spoof: valid");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn golden_example() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../profiles/spoof/example.toml");
        let profile = SpoofProfile::load(&path).expect("example loads");
        assert!(has_keys(REQUIRED_KEYS));
        assert!(profile.fingerprint.contains("cheetah"));
        assert!(profile.validate().is_ok());
    }

    #[test]
    fn gaming_presets_load() {
        for (name, want) in [
            ("gaming-basic.toml", StackPreset::Basic),
            ("gaming-full.toml", StackPreset::Full),
        ] {
            let path =
                Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../profiles/spoof/{name}"));
            let profile = SpoofProfile::load(&path).expect("gaming preset loads");
            assert_eq!(profile.stack, want, "{name}");
            assert!(profile.validate().is_ok());
        }
    }

    #[test]
    fn rejects_empty() {
        let bad = SpoofProfile {
            fingerprint: String::new(),
            fake_touch: vec![],
            fake_wifi: vec![],
            denylist: vec![],
            stack: StackPreset::Basic,
        };
        assert!(bad.validate().is_err());
        assert!(!has_keys(&["fingerprint"]));
    }
}
