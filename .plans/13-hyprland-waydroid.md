# 13 — Hyprland + Waydroid gaming integration plan (RESEARCH ONLY, no impl)

Date: 2026-09-20. Box: Omarchy/CachyOS, Hyprland 0.56.2, eDP-1 1920x1080@60,
`multi_windows=true`, 7.6GB RAM + earlyoom(-m 15, waydroid avoided).
4 parallel research tracks; synthesis below. Sources inline.

## 1. Findings

### 1.1 Transparent window is NOT a Hyprland bug (track 1)
`waydroid app launch` creates the translucent splash task (`visible=false`,
no opaque buffer) as surface `waydroid.<pkg>`; the compositor faithfully
shows wallpaper through it. Proven live: `am start MAIN/LAUNCHER` on the
resolved component flips `visible=true` (GhostDroid v1.2.2 `resume_launcher`
already ships this). Local `hyprland.conf:26-29` matches only full-UI class
`^(Waydroid)$` — per-app `waydroid.*` classes have NO rules. Upstream:
waydroid#234 (open, multiwindow tiling), waydroid#2008 (closed, Hyprland
0.50 subsurface regression, fixed — box 0.56.2 post-fix), Hyprland#13998
(closed, xdg-activation). No `render:explicit_sync` option on 0.56.

### 1.2 Nested compositors (track 2)
weston 15.0.1 / cage 0.3.1 NOT installed (in pacman). Matrix:
direct-Hyprland (zero copy, native pointer-lock + relative-pointer, HUD via
outer wlr-layer-shell, per-app windows) beats weston-nested (extra copy,
hotkeys trapped in two shells) and cage-kiosk (good single-game compat via
cage-xtmapper + `--confine-pointer`, but HUD locked inside cage, F10-only
escape). Decision: default = direct Hyprland; cage-xtmapper = opt-in PUBG
kiosk profile; weston = legacy/X11 fallback only. Never nest by default.

### 1.3 waydroid-helper gaps worth porting (track 3)
Refs under `.devdocs/waydroid-helper/waydroid_helper/`. Priority order:
1. macro `release_sequence` + `Sleep/Key/Staring/Radius/ToggleGroup` steps
   (`widgets/components/macro.py:555-677`, KEY_MAPPING.md:162-292) over our
   `MacroStep{Down,Up}` (`wd-input/src/adv.rs`).
2. pointer-ownership arbiter + default-suppress + longest-combo/reentrant
   dispatch (`window_input_router.py`, `mapping/key_mapping_manager.py`).
3. smooth stick + live radius factor (`directional_pad.py:226-301`) over our
   instant `engine/stick.rs`.
4. Fire-requires-aim + drag-shot sens (`aim.py`, `fire.py:740-766`).
5. per-kind mapping px + split F12 transparency vs visibility
   (`canvas.rs:121-128` currently conflates; helper separates).
6. Skill/Walk/Repeat modes (`skill_casting.py`, `right_click_to_walk.py`,
   `repeated_click.py`).
Keep ours: rel 0-1 coords (no px scale), structured macro over text DSL,
existing WheelNode (they have none).

### 1.4 Notifications / focus / idle (track 4)
Quickshell notifier (no swaync/mako); DND indicator exists. Zero bare
F1/F2/F8/F9/F10/F12 or bare Alt/Ctrl/WASD Hyprland binds — overlay hotkeys
safe (only in-game `Ctrl+P/E/S` chat overlap to document).
`focus_on_activate=true` default — game raises on launch; do NOT add
`stay_focused`/`suppress_event`. Session runs as bare
`python3 ... session start` (no unit) — dies with shell/logout; container is
root lxc-start. hypridle locks at 152s; suspend path freezes session
(= match timeout). Session deaths observed = lock/suspend/shell death, not
earlyoom (waydroid avoided; browsers preferred).

## 2. Plan (staged, opt-in first)

### Stage A — GhostDroid-managed Hyprland snippet (no core changes)
Ship generator writing `~/.config/hypr/conf.d/ghostdroid.conf` (user enables
by keeping the dir sourced):
```conf
windowrule = match:class ^(Waydroid|waydroid\..*)$, tile on
windowrule = match:class ^(Waydroid|waydroid\..*)$, noblur on
windowrule = match:class ^(Waydroid|waydroid\..*)$, noshadow on
windowrule = match:class ^(Waydroid|waydroid\..*)$, idle_inhibit focus
windowrule = match:class ^(waydroid\..*)$, focus_on_activate on
windowrule = match:class ^(waydroid\.com\.tencent\.ig)$, immediate on
windowrule = match:class ^(Waydroid)$, workspace 7 silent
```
Plus: drop fixed `size 1430 777` + `fullscreen 0` for per-app classes (keep
workspace-only for full-UI shell). Tearing master toggle stays OFF until a
fullscreen-exclusive profile opts in.

### Stage B — session survivability
`~/.config/systemd/user/ghostdroid-waydroid-session.service`
(simple, `Restart=on-failure`, `OOMScoreAdjust=-500`,
`PartOf=graphical-session.target`), `systemctl --user enable --now`.
Replaces setsid/nohup. `ExecStartPre=/usr/bin/waydroid status`.

### Stage C — match session wrapper
`ghostdroid play <pkg>`: `systemd-inhibit --what=idle:sleep`, DND on via
omarchy-shell, `app launch` → resolve+launcher resume (shipped v1.2.2) →
`hyprctl dispatch focuswindow`, F12-clash assert
(`hyprctl binds | grep -i F12`), restore DND/inhibit on exit. Warn on
suspend-kills-match; never auto-suspend.

### Stage D — keymap engine ports (track 3 order)
Behind profile version bump (v2): macro release_sequence/steps, ownership
arbiter, smooth stick, Fire-needs-aim, per-kind px + F12 split, Skill/Walk/
Repeat modes. Each with golden tests in `wd-input/tests/golden.rs`.

### Stage E — opt-in cage kiosk (deferred)
`--cage` launcher flag for PUBG-only sessions
(`cage ... -- waydroid show-full-ui`), XtMapper path, documented HUD
limitation. Requires `omarchy pkg add cage`. Not default.

## 3. Explicit non-goals
- No weston default; no global `allow_tearing`; no `stay_focused`/`no_focus`
  on game classes; no resolution/DPI/Mesa changes (locked).
- No edits made in this research pass (plan only).
