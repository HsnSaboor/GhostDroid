---
name: waydroid-spoof
description: Per-game spoof profiles for Waydroid. Use when a game needs a device fingerprint, fake touch/wifi props, or Play Integrity BASIC+DEVICE.
license: MIT
compatibility: Requires wd-ctl, Magisk, PIFork
metadata:
  author: waydroid-emulator
  version: "0.1.0"
---

# waydroid-spoof

## Profiles

- Source: `profiles/spoof/*.toml` (`fingerprint`, `fake_touch`,
  `fake_wifi`, `denylist`). Canonical fingerprint: cheetah A16.
- Load: `wd-ctl spoof-load <profile>` (validates) or `spoof.load`.
- Applies per-game `waydroid_base.prop` swap + `prop set`. Never global.

## Rules

- Target **BASIC+DEVICE only** — gaming unlock, nothing more.
- **Never STRONG**, never ship keybox/prints, never bundle secrets.
- No banking / payment PKGs. Refuse applist-hide for those.
- Verify with YASNAC / Play Integrity Checker after apply.
- Denylist + Shamiko blacklist per-PKG over global root-off.

## Compat

Copy this dir to `.claude/skills/waydroid-spoof/` or
`.opencode/skills/waydroid-spoof/` — same `SKILL.md` works everywhere.
