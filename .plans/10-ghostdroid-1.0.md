# 10 — GhostDroid 1.0.0: production app (Loop 1)

Live ground truth (2026-09-15, this box):
- `waydroid status`: Session RUNNING, Container RUNNING, MAINLINE, IP
  192.168.240.112, user saboor(1000), Wayland wayland-1.
- `waydroid app list`: 19 apps incl Clash Royale
  (`com.supercell.clashroyale`), Magisk Delta
  (`io.github.huskydg.magisk`), Play Store, Islam360, Al Quran.
- Fingerprint: `samsung/SM-G9860/waydroid_x86_64:13/...:userdebug/test-keys`
  (spoof needed → cheetah gaming-full default).
- `ro.dalvik.vm.native.bridge=libhoudini.so`, abilist incl arm64-v8a.
  `waydroid_base.prop` exists. `multi_windows=true`.
- adb: device `unauthorized` (on-device auth pending). `waydroid shell`
  needs root. Prefer `waydroid app list/prop` (no root, no auth).
- wd-gpui v0.1.0 renders sidebar + light theme + `total=0` empty pages.
- Hyprland dispatch (Lua build): `hyprctl dispatch 'hl.dsp.focus({ workspace = "5" })'`.

## Scope (NOT MVP, NOT demo — all working)

1. **Real spawn layer** (`wd-waydroid`): execute `waydroid` CLI as supervised
   child (timeout + output cap), parse `app list` → `GameRow{title,pkg}`,
   `status` → session/container/frozen, `prop get/set`, `session start/stop`,
   `app launch/stop`, `app install/remove`. Replace stub argv responses.
   Pure parsers unit-tested; spawn only in daemon/CLI, never in UI crate.
2. **Self-boot**: `device.boot` runs `waydroid session start --wait`,
   polls `status` until RUNNING (timeout 120s, frozen probe). UI Devices
   page has Start/Stop buttons wired to real actions.
3. **Auto-load apps**: Library boots from `waydroid app list` parse;
   refresh button re-runs; search + Ready/Running filter over real rows.
   Genre/state chips from package metadata (no invented metrics).
4. **Dark-only**: force gpui-kit dark theme at boot (`Theme` global apply,
   dark scheme). No light fallback, no white surfaces. Contrast: text on
   dark ≥ 7:1 body, ≥ 4.5:1 secondary. Verify via Xvfb screenshots.
5. **Prebuilt kit only**: Sidebar/Tabs/Input/Select/Switch/Slider/Button/
   Tag/DescriptionList/List/Progress/Dialog/Notification/Empty from
   gpui-kit 0.6. No hand-rolled div-buttons where a kit component exists.
   Docs: `.devdocs/gpui-kit/` (vendored). Agents MUST re-read vendored
   component source before use.
6. **Pages**: Library (search Input entity + filter Tabs + game grid +
   toolbar), Devices (status dot + props + Start/Stop/Scan),
   Spoof (profile Select + props + Hide-root Switch + sensor Slider),
   Keys (profile/fire rows + Map/Aim/DPad tabs + overlay handoff note),
   Logs (live tail + Clear + count).
7. **CLI+MCP parity**: every UI action has a `wd-ctl` verb + MCP tool.
   `wd-ctl boot` self-boots, `devices` lists real apps, `spoof-load`
   validates TOML, `keymap-load` validates JSON.

## Design discipline

- Hallmark: dark gamer aesthetic, structural variety per page, no AI-slop
  (no identical card grids, no gradient text, no side-stripe accents,
  no invented stats). Pre-emit self-critique per page.
- Impeccable product register: design serves the product. 8-state
  checklist for interactive controls. Contrast gates 40-41.
- Theme tokens locked: one dark palette, named tokens, no inline hex.

## Quality gates (every subagent)

- `cargo clippy -p <crate> --all-targets` 0 errors 0 warnings
  (workspace denies all+pedantic+nursery).
- `cargo fmt -p <crate> -- --check` clean. Files ≤150 lines.
- `cargo test -p <crate>` green. tracing at every op.
- Xvfb screenshots per page, dark verified, no white flash.
- Reuse `wd-shell` viewmodels + `wd-waydroid` parsers. No duplication.

## Loop protocol

Loop 1: implement → testing agent (all verbs, all pages, screenshots) →
gh release v1.0.0 → local install → verify via skill + CLI → debug log.
Loop 2+: same, with gaps list appended here. Stop only when zero gaps.
