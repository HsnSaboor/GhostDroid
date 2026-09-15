//! wd-gpui: GPUI desktop shell scaffold. Pure view layer, no Waydroid logic.
//!
//! Page bodies live in [`pages`]; page agents own their content.

/// Shell root view.
pub mod app;
/// Page dispatch + bodies (per-page modules own content).
pub mod pages;
/// Shared row/card compositions.
pub mod shared;
/// Pure app state + mutators.
pub mod state;
/// Self-boot + auto-load: Waydroid session boot, app list, device snapshot.
pub mod sync;
