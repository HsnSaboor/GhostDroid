# Decision — stack for Waydroid emulator

Date: 2026-09-14. Target: Linux Wayland, Waydroid LXC backend, gaming.

## Verdict

- Core: **Rust workspace**. Daemon + CLI + MCP same core. GUI same core.
- Shell UI: **Slint** (beautiful now). Alt if MIT must: **Iced**.
- Overlay / keymap editor: **egui glow** (or Dear ImGui) embedded, not main shell.
- No: Electron/Tauri/Dioxus-desktop (webview veto). No GTK/Qt6 (constraint + static-pack pain).
- Reuse, not rewrite: `phantom` engine+profiles, `waydroid_script` installs, `waydroid_base.prop` spoof pattern.
- Python only as supervised sidecar for scripts. No TS. No C++ except Skia underneath Slint.

## Why

- ponytail: need it? yes daemon. exists? `phantom/` already Rust daemon+IPC+profiles. stdlib? no. minimum: extend, not rewrite.
- Rust ~42ms start vs Python ~690ms vs Node ~2300ms. Rust bin 300KB-2MB static. No GC pause in aim path. evdev/uinput + `wayland-client` + `tokio` + `zbus` all native.
- Slint = only path with zero-gap beautiful dark UI + Wayland + AccessKit + multi-window today (v1.17.1 Jul 2026). License check: GPLv3 OR royalty-free desktop OR paid embedded.
- egui glow = 5.6MB LTO vs wgpu 11.3MB, 0.1-0.3s start vs 1-7s wgpu. Best drag-key editor.
- GPUI = fastest frames, zero widgets (746-line input example), no a11y, git-pin only. Skip as main.
- Makepad/Xilem/Freya = beauty high, Wayland/widgets unstable. Watch 12mo.

## Layout

```
core-rs/      # daemon, evdev engine, waydroid ctl, spoof profiles, keymap schema
shell-slint/  # main window, device controls, profile manager
overlay-egui/ # keymap editor OSD, layer-shell passthrough
py-sidecar/   # waydroid_script + total-spoof prop lists as JSON child
cli+mcp       # same core-rs, stdio default, http opt-in
skills/       # waydroid-control + waydroid-spoof SKILL.md
```

## Non-goals v1

- No STRONG integrity promise. No shipped keybox. No sensor/GPU fake. No CV smart-key.
- Target: BASIC+DEVICE for games, per-game spoof profiles, root toggle per-app hide.
