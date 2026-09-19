//! Self-boot + auto-load: Waydroid session boot, app list, device snapshot.
//!
//! All blocking work; call off the UI thread (`ShellView::new` runs this on
//! a background executor). Missing `waydroid` binary → `Err`, caller keeps
//! the seeded fallback.

use std::time::{Duration, Instant};

use wd_core::Pkg;
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

/// Container power action, mirroring Devices page buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceAction {
    /// `waydroid session start` + poll to RUNNING.
    Start,
    /// `waydroid session stop`.
    Stop,
    /// Stop then boot again.
    Restart,
}

impl DeviceAction {
    /// Button label (kit `Button` text, dark theme inherits).
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Start => "Start",
            Self::Stop => "Stop",
            Self::Restart => "Restart",
        }
    }
}

/// Run one container power action, then return the fresh snapshot.
///
/// Start/Restart poll via [`ensure_booted`]; Stop returns the post-stop
/// snapshot so the page flips to STOPPED without a second click.
///
/// # Errors
///
/// Returns the spawn error when `waydroid` is missing or the action fails.
pub fn device_action(action: DeviceAction) -> wd_core::Result<(DeviceState, String)> {
    tracing::info!(?action, "sync: device action");
    match action {
        DeviceAction::Start | DeviceAction::Restart => {
            if action == DeviceAction::Restart {
                let stop = wd_waydroid::session::shutdown_args();
                let stop_ref: Vec<&str> = stop.iter().map(String::as_str).collect();
                let _ = run_waydroid(&stop_ref, QUICK_MS);
            }
            ensure_booted()?;
            fetch_device()
        }
        DeviceAction::Stop => {
            let stop = wd_waydroid::session::shutdown_args();
            let stop_ref: Vec<&str> = stop.iter().map(String::as_str).collect();
            run_waydroid(&stop_ref, QUICK_MS)?;
            fetch_device()
        }
    }
}

/// Launch one app via `waydroid app launch <pkg>` (Library card button).
///
/// # Errors
///
/// Returns the spawn error when `waydroid` is missing or launch fails.
pub fn launch_game(pkg: &str) -> wd_core::Result<()> {
    tracing::info!(pkg, "sync: launch game");
    let args = wd_waydroid::launch_args(&Pkg(pkg.to_owned()));
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    run_waydroid(&args_ref, QUICK_MS)?;
    Ok(())
}

// --- Spoof catalog: compile-time-embedded canonical TOMLs -----------------
// ponytail: no new dep (toml/serde) on low-power box; tiny line parser only.

const S26_TOML: &str = include_str!("../../../profiles/spoof/s26-ultra.toml");
const ROG8_TOML: &str = include_str!("../../../profiles/spoof/gaming-full.toml");
const PIXEL8_TOML: &str = include_str!("../../../profiles/spoof/gaming-basic.toml");

/// Spoof ids shown in the picker (display id → source TOML).
pub const SPOOF_IDS: [&str; 3] = ["s26-ultra", "rog8", "pixel8"];

fn toml_source(id: &str) -> &'static str {
    match id {
        "rog8" => ROG8_TOML,
        "pixel8" => PIXEL8_TOML,
        _ => S26_TOML,
    }
}

/// Pull one `key = "value"` line from an embedded TOML source.
fn toml_value(src: &str, key: &str) -> String {
    src.lines()
        .filter_map(|line| line.split_once('='))
        .filter(|(k, _)| k.trim().trim_matches('"') == key)
        .filter_map(|(_, v)| {
            let v = v.trim().trim_matches('"').trim().to_owned();
            if v.is_empty() { None } else { Some(v) }
        })
        .next()
        .unwrap_or_default()
}

/// Canonical prop rows for a spoof id, read from the embedded profile TOML.
/// Keys mirror `profiles/spoof/*.toml`; empty values are skipped (never show
/// blank rows — WCAG AAA noise rule for this shell).
#[must_use]
pub fn spoof_props(id: &str) -> Vec<(String, String)> {
    tracing::debug!(id, "sync: spoof props");
    let src = toml_source(id);
    let mut rows = Vec::new();
    for key in [
        "ro.product.model",
        "ro.product.name",
        "ro.product.device",
        "fingerprint",
        "stack",
        "ro.product.cpu.abi",
        "ro.product.cpu.abilist",
        "ro.build.tags",
        "ro.build.type",
    ] {
        let value = if key == "stack" {
            toml_value(src, "stack")
        } else {
            toml_value(src, key)
        };
        if !value.is_empty() {
            rows.push((key.to_owned(), value));
        }
    }
    rows
}

/// Diff lines of `id` vs the `s26-ultra` baseline (`~ key: base → mine`,
/// `+ key: mine`). Empty when the profile matches baseline.
#[must_use]
pub fn spoof_diff(id: &str) -> Vec<String> {
    tracing::debug!(id, "sync: spoof diff");
    if id == "s26-ultra" {
        return Vec::new();
    }
    let base = spoof_props("s26-ultra");
    let mine = spoof_props(id);
    let mut out = Vec::new();
    for (key, value) in &mine {
        match base.iter().find(|(k, _)| k == key) {
            Some((_, b)) if b != value => {
                out.push(format!("~ {key}: {b} → {value}"));
            }
            None => out.push(format!("+ {key}: {value}")),
            _ => {}
        }
    }
    out
}

// --- Keymap summary: compile-time-embedded pubg.json -----------------------

const PUBG_JSON: &str = include_str!("../../../profiles/keymap/pubg.json");

/// Count keymap nodes by `"slot"` occurrences (one per node in pubg.json).
#[must_use]
pub fn keymap_node_count() -> usize {
    PUBG_JSON.match_indices("\"slot\"").count()
}

/// Fire key: value of `"key"` in the node whose `"id"` is `"fire"`.
#[must_use]
pub fn keymap_fire_key() -> String {
    let mut fire = false;
    for line in PUBG_JSON.lines() {
        let t = line.trim();
        if t.contains("\"id\"") && t.contains("\"fire\"") {
            fire = true;
        }
        if fire && t.contains("\"key\"") {
            if let Some(v) = t.split('"').nth(3) {
                return v.to_owned();
            }
        }
    }
    "MouseLeft".to_owned()
}

/// `(profile, fire_key, node_count)` for the Keys page header.
#[must_use]
pub fn keymap_summary() -> (String, String, usize) {
    ("pubg".to_owned(), keymap_fire_key(), keymap_node_count())
}

// --- Live log tail: ghostdroid socket --------------------------------------

/// Sockets tried in order (task names ghostdroid first, daemon second).
pub const LOG_SOCKETS: [&str; 2] = ["/tmp/ghostdroid.sock", "/tmp/wd-daemon.sock"];

/// Best-effort tail from the live daemon socket. Empty on any failure —
/// caller keeps the seeded tail (never blanks the Logs page).
#[must_use]
pub fn tail_socket_logs() -> Vec<String> {
    use std::io::Read as _;
    use std::os::unix::net::UnixStream;
    for path in LOG_SOCKETS {
        let mut stream = match UnixStream::connect(path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        if stream
            .set_read_timeout(Some(Duration::from_millis(300)))
            .is_err()
        {
            continue;
        }
        let mut buf = vec![0u8; 16_384];
        let text = match stream.read(&mut buf) {
            Ok(0) => continue,
            Ok(n) => String::from_utf8_lossy(&buf[..n]).into_owned(),
            Err(_) => continue,
        };
        let lines: Vec<String> = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_owned)
            .collect();
        if !lines.is_empty() {
            tracing::info!(path, count = lines.len(), "sync: socket logs");
            return lines;
        }
    }
    Vec::new()
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

    #[test]
    fn spoof_catalog_live_to_toml() {
        assert_eq!(SPOOF_IDS, ["s26-ultra", "rog8", "pixel8"]);
        let s26 = spoof_props("s26-ultra");
        assert!(
            s26.iter()
                .any(|(k, v)| k == "ro.product.model" && v == "SM-S948B")
        );
        assert!(s26.iter().any(|(k, _)| k == "fingerprint"));
        assert!(spoof_diff("s26-ultra").is_empty());
        assert!(!spoof_diff("rog8").is_empty());
    }

    #[test]
    fn pubg_has_21_nodes() {
        assert_eq!(keymap_node_count(), 21);
        assert_eq!(keymap_fire_key(), "MouseLeft");
        let (profile, fire, nodes) = keymap_summary();
        assert_eq!(
            (profile.as_str(), fire.as_str(), nodes),
            ("pubg", "MouseLeft", 21)
        );
    }

    #[test]
    fn device_action_labels() {
        assert_eq!(DeviceAction::Start.label(), "Start");
        assert_eq!(DeviceAction::Stop.label(), "Stop");
        assert_eq!(DeviceAction::Restart.label(), "Restart");
    }
}
