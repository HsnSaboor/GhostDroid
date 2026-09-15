//! Self-boot + auto-load: Waydroid session boot, app list, device snapshot.
//!
//! All blocking work; call off the UI thread (`ShellView::new` runs this on
//! a background executor). Missing `waydroid` binary → `Err`, caller keeps
//! the seeded fallback.

use std::time::{Duration, Instant};

use wd_shell::{DeviceState, GameRow};
use wd_waydroid::{AppRow, Status, boot_args, parse_app_list, parse_status, run_waydroid};

/// Per-call timeout for `status` / `app list`.
const QUICK_MS: u64 = 10_000;
/// Timeout for `session start` (first boot can be slow).
const BOOT_MS: u64 = 90_000;
/// Budget for polling `status` after boot.
const POLL_BUDGET: Duration = Duration::from_secs(60);
/// Gap between status polls.
const POLL_GAP: Duration = Duration::from_millis(500);

/// Map a parsed [`Status`] to shell [`DeviceState`].
#[must_use]
pub fn status_to_device(status: &Status) -> DeviceState {
    tracing::debug!(?status, "sync: map status to device");
    DeviceState {
        ready: status.session && status.container,
        frozen: status.frozen,
    }
}

/// Map one [`AppRow`] to a shell [`GameRow`] (`genre` "App", `state` "ready").
#[must_use]
pub fn row_to_game(row: &AppRow) -> GameRow {
    tracing::debug!(pkg = %row.pkg, "sync: map row to game");
    GameRow {
        title: row.title.clone(),
        pkg: row.pkg.clone(),
        genre: "App".to_owned(),
        state: "ready".to_owned(),
    }
}

/// Ensure session + container report `RUNNING`, booting first if needed.
///
/// Polls `status` after boot until both are up or the budget lapses.
///
/// # Errors
///
/// Returns the spawn error when `waydroid` is missing, boot fails, the
/// container reports frozen, or the poll budget lapses.
pub fn ensure_booted() -> wd_core::Result<()> {
    tracing::info!("sync: ensure booted");
    let live = parse_status(&run_waydroid(&["status"], QUICK_MS)?);
    if status_to_device(&live).ready {
        tracing::info!("sync: already running");
        return Ok(());
    }
    tracing::info!("sync: session down, starting");
    let boot = boot_args(false, false);
    let boot_ref: Vec<&str> = boot.iter().map(String::as_str).collect();
    run_waydroid(&boot_ref, BOOT_MS)?;
    let start = Instant::now();
    loop {
        let status = parse_status(&run_waydroid(&["status"], QUICK_MS)?);
        tracing::debug!(?status, "sync: boot poll");
        if status.frozen {
            tracing::warn!("sync: frozen after boot");
            return Err(wd_core::WdError::Frozen(
                "container frozen after boot".to_owned(),
            ));
        }
        if status.session && status.container {
            tracing::info!("sync: booted");
            return Ok(());
        }
        if start.elapsed() >= POLL_BUDGET {
            tracing::warn!("sync: boot poll timed out");
            return Err(wd_core::WdError::Timeout(
                "waydroid boot poll timed out".to_owned(),
            ));
        }
        std::thread::sleep(POLL_GAP);
    }
}

/// Load installed apps as shell game rows.
///
/// # Errors
///
/// Returns the spawn error when `waydroid` is missing or `app list` fails.
pub fn fetch_games() -> wd_core::Result<Vec<GameRow>> {
    tracing::info!("sync: fetch games");
    let rows = parse_app_list(&run_waydroid(&["app", "list"], QUICK_MS)?);
    tracing::info!(count = rows.len(), "sync: games loaded");
    Ok(rows.iter().map(row_to_game).collect())
}

/// Load the device snapshot plus container IP.
///
/// # Errors
///
/// Returns the spawn error when `waydroid` is missing or `status` fails.
pub fn fetch_device() -> wd_core::Result<(DeviceState, String)> {
    tracing::info!("sync: fetch device");
    let status = parse_status(&run_waydroid(&["status"], QUICK_MS)?);
    let ip = status.ip.clone().unwrap_or_default();
    tracing::info!(ready = status.session && status.container, %ip, "sync: device loaded");
    Ok((status_to_device(&status), ip))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn running_maps_ready() {
        let out = "Session:\tRUNNING\nContainer:\tRUNNING\nIP address:\t192.168.240.112\n";
        let device = status_to_device(&parse_status(out));
        assert!(device.ready);
        assert!(!device.frozen);
    }

    #[test]
    fn frozen_maps_frozen_not_ready() {
        let out = "Session:\tRUNNING\nContainer:\tFROZEN\n";
        let device = status_to_device(&parse_status(out));
        assert!(device.frozen);
        assert!(!device.ready);
    }

    #[test]
    fn stopped_maps_not_ready() {
        let out = "Session:\tSTOPPED\nContainer:\tSTOPPED\n";
        assert!(!status_to_device(&parse_status(out)).ready);
    }

    #[test]
    fn row_maps_game() {
        let row = AppRow {
            title: "Clash Royale".to_owned(),
            pkg: "com.supercell.clashroyale".to_owned(),
        };
        let game = row_to_game(&row);
        assert_eq!(game.title, "Clash Royale");
        assert_eq!(game.pkg, "com.supercell.clashroyale");
        assert_eq!(game.genre, "App");
        assert_eq!(game.state, "ready");
    }
}
