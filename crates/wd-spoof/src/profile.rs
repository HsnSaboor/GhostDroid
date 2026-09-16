//! Per-game spoof profile (TOML).
//!
//! Ref: `.devdocs/DeviceSpoofLab-Hooks/device_profile.conf`
//! (cheetah canonical 260, arm64/hw block 138-148).

use std::path::Path;

use serde::{Deserialize, Serialize};

/// Stack preset: minimal vs full hide stack. Traced on load.
/// Default is [`StackPreset::Full`] — full is the default root/spoof profile.
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
    /// `ro.build.fingerprint` value.
    pub fingerprint: String,
    /// `persist.waydroid.fake_touch` package globs.
    pub fake_touch: Vec<String>,
    /// `persist.waydroid.fake_wifi` package globs.
    pub fake_wifi: Vec<String>,
    /// Denylist packages (applist-hide ref, 05 owns policy).
    pub denylist: Vec<String>,
    /// Stack preset; defaults `full`.
    #[serde(default)]
    pub stack: StackPreset,
    /// `ro.product.model` (e.g. `SM-S948B`). Empty = line skipped.
    #[serde(default, rename = "ro.product.model")]
    pub model: String,
    /// `ro.product.name` (e.g. `m3qxeea`). Empty = line skipped.
    #[serde(default, rename = "ro.product.name")]
    pub product: String,
    /// `ro.product.device` (e.g. `m3q`). Empty = line skipped.
    #[serde(default, rename = "ro.product.device")]
    pub device: String,
    /// `ro.hardware` (NEVER rendered: graphics HAL reads it at boot, phone
    /// SoC strings kill hwcomposer+surfaceflinger; kept for compat, ignored).
    #[serde(default, rename = "ro.hardware")]
    pub hardware: String,
    /// `ro.product.cpu.abi` (always `arm64-v8a`). Empty = line skipped.
    #[serde(default, rename = "ro.product.cpu.abi")]
    pub cpu_abi: String,
    /// `ro.product.cpu.abilist` (never x86). Empty = line skipped.
    #[serde(default, rename = "ro.product.cpu.abilist")]
    pub cpu_abilist: String,
    /// `ro.build.tags` (always `release-keys`). Empty = line skipped.
    #[serde(default, rename = "ro.build.tags")]
    pub build_tags: String,
    /// `ro.build.type` (always `user`). Empty = line skipped.
    #[serde(default, rename = "ro.build.type")]
    pub build_type: String,
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
        tracing::info!(fingerprint = %profile.fingerprint, stack = ?profile.stack, "spoof: loaded");
        Ok(profile)
    }

    /// Validate: non-empty fingerprint + globs, never x86 in abilist.
    ///
    /// # Errors
    /// Returns [`wd_core::WdError::Validation`] when empty or leaking x86.
    pub fn validate(&self) -> wd_core::Result<()> {
        tracing::debug!(fingerprint = %self.fingerprint, "spoof: validate");
        if self.fingerprint.trim().is_empty() {
            return Err(wd_core::WdError::Validation("empty fingerprint".to_owned()));
        }
        if self.fake_touch.is_empty() || self.fake_wifi.is_empty() {
            return Err(wd_core::WdError::Validation(
                "empty fake_* globs".to_owned(),
            ));
        }
        if self.cpu_abilist.contains("x86") {
            return Err(wd_core::WdError::Validation(
                "x86 leak in abilist".to_owned(),
            ));
        }
        if !self.hardware.trim().is_empty() {
            return Err(wd_core::WdError::Validation(
                "ro.hardware banned in base.prop (kills hwcomposer/surfaceflinger)".to_owned(),
            ));
        }
        tracing::debug!("spoof: valid");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load(name: &str) -> SpoofProfile {
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../profiles/spoof");
        SpoofProfile::load(&base.join(name)).expect("profile loads")
    }

    #[test]
    fn golden_example() {
        let profile = load("example.toml");
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
            let profile = load(name);
            assert_eq!(profile.stack, want, "{name}");
            assert!(profile.validate().is_ok());
        }
    }

    #[test]
    fn s26_ultra_spoof() {
        let p = load("s26-ultra.toml");
        assert_eq!(p.stack, StackPreset::Full);
        assert!(p.fingerprint.contains("m3qxeea/m3q"));
        assert_eq!(
            (p.model.as_str(), p.product.as_str()),
            ("SM-S948B", "m3qxeea")
        );
        assert_eq!(p.device.as_str(), "m3q");
        assert!(
            p.hardware.trim().is_empty(),
            "ro.hardware banned in base.prop"
        );
        assert_eq!(p.cpu_abi, "arm64-v8a");
        assert!(!p.cpu_abilist.contains("x86"));
        assert_eq!(
            (p.build_tags.as_str(), p.build_type.as_str()),
            ("release-keys", "user")
        );
        assert!(p.validate().is_ok());
    }

    #[test]
    fn rejects_empty() {
        let mut bad = load("example.toml");
        bad.fingerprint.clear();
        bad.fake_touch.clear();
        bad.fake_wifi.clear();
        assert!(bad.validate().is_err());
        assert!(!has_keys(&["fingerprint"]));
    }
}
