//! Config load: `~/.config/wd/config.toml`, trimmed YAGNI fields.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Minimal runtime config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WdConfig {
    /// Screen size.
    #[serde(default = "default_screen")]
    pub screen: (u32, u32),
    /// Working dir.
    #[serde(default = "default_work_dir")]
    pub work_dir: PathBuf,
    /// Daemon socket path.
    #[serde(default = "default_socket")]
    pub socket: PathBuf,
    /// Log level string.
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

const fn default_screen() -> (u32, u32) {
    (1920, 1080)
}

fn default_work_dir() -> PathBuf {
    PathBuf::from("/tmp/wd")
}

fn default_socket() -> PathBuf {
    PathBuf::from("/tmp/wd-daemon.sock")
}

fn default_log_level() -> String {
    String::from("info")
}

impl Default for WdConfig {
    fn default() -> Self {
        Self {
            screen: default_screen(),
            work_dir: default_work_dir(),
            socket: default_socket(),
            log_level: default_log_level(),
        }
    }
}

impl WdConfig {
    /// Config file path.
    #[must_use]
    pub fn path() -> PathBuf {
        std::env::var("XDG_CONFIG_HOME")
            .map_or_else(
                |_| {
                    std::env::var("HOME").map_or_else(
                        |_| PathBuf::from("/tmp"),
                        |h| PathBuf::from(h).join(".config"),
                    )
                },
                PathBuf::from,
            )
            .join("wd/config.toml")
    }

    /// Load from disk or default when missing.
    ///
    /// # Errors
    /// Returns [`crate::err::WdError`] on IO / parse failure.
    pub fn load() -> crate::err::Result<Self> {
        let path = Self::path();
        tracing::info!(path = %path.display(), "loading config");
        if !path.exists() {
            tracing::info!("config missing, using defaults");
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(&path)?;
        let cfg: Self = toml::from_str(&text).map_err(crate::err::WdError::Toml)?;
        tracing::info!(path = %path.display(), "config loaded");
        Ok(cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let cfg = WdConfig::default();
        let json = serde_json::to_string(&cfg).expect("serde");
        let back: WdConfig = serde_json::from_str(&json).expect("deser");
        assert_eq!(back.screen, (1920, 1080));
    }
}
