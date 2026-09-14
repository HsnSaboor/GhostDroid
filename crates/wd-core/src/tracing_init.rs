//! Tracing init, called once per binary.

/// Init tracing subscriber with env filter.
///
/// # Errors
/// Currently infallible; keeps `Result` for future fallible layers.
pub fn init_tracing(default_level: &str) -> crate::err::Result<()> {
    use tracing_subscriber::{EnvFilter, fmt};
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| default_level.to_owned());
    tracing::info!(filter = %filter, "init tracing");
    let _ = fmt().with_env_filter(EnvFilter::new(filter)).try_init();
    Ok(())
}
