# 02 Core daemon — supervisor + waydroid ctl + IPC

Refs: `.research/00-overview-decision.md:25-32`, `.research/03-tech-stack.md:8-10`, `.research/03-tech-stack.md:89-117`; `.devdocs/phantom/phantom/src/waydroid.rs`, `ipc.rs:1125`, `main.rs:799`, `desktop_relay.rs:204`, `docs/IPC.md`, `docs/OPERATIONS.md`; `.devdocs/phantom-wl/.../main.rs:976`; `.devdocs/waydroid/tools/actions/{session_manager,container_manager,prop,app_manager}.py`, `tools/helpers/props.py`; `.devdocs/waydroid-mcp/src/waydroid_mcp/{adb.py:39-51,core.py:43-95,session.py,cli.py}`.

## Goal
One daemon owns LXC session, child procs, sockets. CLI/MCP/shell thin clients call daemon IPC. No replay drift. Frozen always distinguished from timeout.

## Reuse (extend, never rewrite)
- Paths from `.devdocs/phantom/phantom/src/waydroid.rs:18-30`: `WAYDROID_DEFAULT_WORK_DIR=/var/lib/waydroid`, overlay/idc/cfg paths, `DEFAULT_ANDROID_SERVER_JAR=/data/local/tmp/phantom-server.jar`, `SYSTEM_JAR=/usr/lib/phantom/phantom-server.jar`, `PORT=27183`, `IDC_TEMPLATE=contrib/waydroid/Vendor_1234_Product_5678.idc`. Copy `WaydroidPaths/InstallReport/CommandReport/DiagnosisReport` structs + `ensure_android_server/host/port/log_excerpt`, `getevent/dumpsys` diagnose fns.
- IPC shape from `.devdocs/phantom/phantom/src/ipc.rs:1125` (`IpcRequest/IpcResponse/DaemonState`) + `docs/IPC.md` + `docs/OPERATIONS.md`. One method = one `wd-*` lib fn. Unix socket JSON-RPC newline-delimited, logs stderr only (MCP stdio safe).
- Engine tick constants from `.devdocs/phantom/phantom/src/main.rs:19`: `ENGINE_TICK_INTERVAL_MS=4`, `input_interval=1ms poll_events(0)` epoll non-blocking. Keep in daemon supervisor loop, not UI.
- Waydroid actions from `.devdocs/waydroid/tools/actions/session_manager.py,container_manager.py,prop.py,app_manager.py` + `helpers/props.py:host_get/host_list/host_set/file_get/IPlatform setprop`. Shell these, no custom binder. Config base `.devdocs/waydroid/data/configs/config_base`, scripts `waydroid-net.sh/waydroid-post-stop.sh`. Version 1.6.3.
- Frozen detect COPY from `.devdocs/waydroid-mcp/src/waydroid_mcp/adb.py:39-51`: `raw()` catches Timeout → probe `waydroid status` → `FROZEN` in output → `AdbError("frozen","container frozen — show session UI")` else timeout. No other MCP repo does this — mandatory.

## Modules (`crates/wd-waydroid` lib, `crates/wd-daemon` lib+thin bin)
```
wd-waydroid::session { boot{wait,frozen_check}, shutdown, freeze, unfreeze, status{session,container,frozen}, wait_ready(path=/run/waydroid/wayland-0,timeout) }
wd-waydroid::props { get/set/list, KEYS: fake_touch/fake_wifi/multi_windows/width/height/suspend/uevent }
wd-waydroid::apps { install/uninstall/list/launch/stop/current }
wd-daemon::supervisor { spawn supervised child: logcat tail, screencap, py-sidecar, inject socket; cgroupv2 kill-on-drop; restart backoff }
wd-daemon::ipc { serve Unix socket ~/.run/wd/daemon.sock; Req{method,params} Resp{ok,data} from wd-core::rpc }
```
- zbus typed proxy `id.waydro.Container` for status; fallback shell `lxc-info + waydroid status` parse.
- Session flow implement exactly `.research/03-tech-stack.md:92-101`: boot --wait --frozen-check → lxc-start+binder → hwcomposer `wayland-hwc` xdg_toplevel vsync 60Hz → input pipes MT_SLOT/TRACKING_ID → ready → launch PKG w/ fake_* globs → keymap load → play+OSD → backup/restore.
- Per-game swap `.research/03-tech-stack.md:103-115`: stop → swap `waydroid_base.prop` → `upgrade --offline` → start+wait → clear gms+game data → keymap+props → launch.

## ui.dump staleness guard (daemon-side screencap/uiautomator helper)
- COPY `.devdocs/waydroid-mcp/src/waydroid_mcp/core.py:43-95`: `rm -f /sdcard/ui.xml` BEFORE every `uiautomator dump` (stale `ERROR: null root node` bug), retries `dump_tries=3 gap=1.0` (transient rebind window). Unique tmp shape from `.devdocs/scrcpy-mcp/src/tools/ui.ts:52-60` (`/sdcard/.ui_dump_${Date.now()}_${rand}.xml`, strip `UI hier... dumped to:` line). Do NOT copy fixed `/sdcard/window_dump.xml` no-rm shape from android-mcp-server-us (stale-prone).

## Steps
1. `wd-waydroid::session+props` with dry-run flag. Test on real Waydroid: boot/status/frozen parse.
2. `wd-daemon::supervisor` owns 4 children. Kill-on-drop verified via test.
3. IPC socket + 5 methods wired to lib fns. `wd-ctl` thin client proves no drift.
4. Frozen integration test: mock `waydroid status` containing FROZEN → error kind frozen.

## YAGNI
- No new D-Bus daemon. No video encode v1 (screencap only, scrcpy 33ms later). No OTA/image builder.

## Blast radius
- Touches: `crates/wd-waydroid/*`, `crates/wd-daemon/*` only. No UI/input/spoof.
- Risk: MED (root LXC calls). Guard: `--dry-run`, timeouts everywhere, never auto-wipe without `--confirm`, snapshot props before swap (04).
- Rollback: `pkill wd-daemon`; LXC sessions persist independently.
- Accept: boot/status/frozen unit+integration green, clippy 0/0.
