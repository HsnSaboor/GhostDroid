//! Shared error type.

/// All errors flow through here.
#[derive(Debug, thiserror::Error)]
pub enum WdError {
    /// IO failure.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// JSON failure.
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    /// TOML decode failure.
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    /// Operation timed out.
    #[error("timeout: {0}")]
    Timeout(String),
    /// Waydroid frozen / not ready.
    #[error("frozen: {0}")]
    Frozen(String),
    /// Validation failure.
    #[error("validation: {0}")]
    Validation(String),
    /// Internal catch-all.
    #[error("internal: {0}")]
    Internal(String),
}

/// Shared result alias.
pub type Result<T> = std::result::Result<T, WdError>;
