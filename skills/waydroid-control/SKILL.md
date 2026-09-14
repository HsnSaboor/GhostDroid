---
name: waydroid-control
description: Control Waydroid via wd-ctl/MCP. Use when tapping apps, screenshots, logcat, installing APKs, UI automation, or debugging a game session.
license: MIT
compatibility: Requires wd-ctl, adb, Wayland
metadata:
  author: waydroid-emulator
  version: "0.1.0"
---

# waydroid-control

MCP truth, CLI fallback. Every MCP tool maps 1:1 to a `wd-ctl` verb.

## Loop: observe → locate → act → re-observe

1. **Observe**: `vision.screenshot` (or `wd-ctl screenshot`) first. Check
   frozen: `device.status` — `FROZEN` means show the session UI, retry.
2. **Locate**: prefer `ui.find {text,rid,class,desc}` over guessed coords.
   Canvas games have no tree — use screenshot + `input.tap x y`.
3. **Act**: `input.tap/swipe/key/text`, `app.start --stop-first`.
4. **Re-observe**: screenshot after every act. Never assume it worked.

## Rules

- `--no-tree` (or `WM_NO_TREE=1`) for canvas apps: `uiautomator` dump
  rebinds a11y ~0.8s and returns empty anyway.
- `ui.dump` is rm-first + retry 3×1s internally; never cache dumps.
- `shell.exec` gated: needs `ANDROID_MCP_ALLOW_SHELL=1`. Allowlist only.
- Record macros (`record-start/stop`), never replay raw coords blindly.
- `--json` always on CLI. Exits: 3 = screen mismatch, 4 = device error.

## Compat

Copy this dir to `.claude/skills/waydroid-control/` or
`.opencode/skills/waydroid-control/` — same `SKILL.md` works everywhere.
