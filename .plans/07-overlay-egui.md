# 07 Overlay egui glow — editor + OSD (thin, separate proc)

Refs: `.research/00-overview-decision.md:9,19`, `.research/03-tech-stack.md:8-13`; `.devdocs/phantom/phantom-gui/src/{main.rs:5491,overlay.rs:1495,cursor_overlay.rs:545}`, `phantom/src/{overlay.rs:387,engine.rs:2915,input.rs:2234}`, `docs/OPERATIONS.md`; `.devdocs/phantom-wl/.../wayland_cursor.rs:60`; `.devdocs/waydroid-helper/docs/KEY_MAPPING.md`, `controller/.../{mode_controller.py:60 EDIT/MAPPING F1,window.py:510 switch_mode/F12,window_input_router.py:242,widget_layout_service.py:210 v1.3,widgets/base/base_widget.py:512 50x50}`, `widgets/{aim.py:888,directional_pad.py:791,fire.py:866,macro.py:975,repeated_click.py:593,right_click_to_walk.py:692,single_click.py:375,skill_casting.py:1191}`; `.devdocs/egui-material3/{src/{lib,theme,toolbar,fab,progress}.rs,examples/widget_gallery}`; `.devdocs/cage-xtmapper/README.md:90 F10`, `cage/{cage.c:756,seat.c:1240}`; `.devdocs/XtMapper/.../KeymapConfig.java:pause Ctrl+P/editor Ctrl+E/profile Ctrl+S`.

## Goal
Drag-keys editor + OSD. Immediate-mode wins. Separate process, killed/hidden in play. Zero profile logic — calls `wd-input` + `wd-shell` viewmodels.

## Why glow + layer-shell
- `eframe glow`: 5.6MB LTO vs wgpu 11.3MB, start 0.1-0.3s vs 1-7s (`.research/00-overview-decision.md:19`). Use `eframe` glow feature, NOT default wgpu.
- Passthrough: port `phantom-gui/.../overlay.rs:1495 + cursor_overlay.rs:545` + `phantom/src/overlay.rs:387` menu-touch (left=down/up, drag=move, layer-shell empty input region) + `wayland-client/smithay-client-toolkit`. wl cursor `wayland_cursor.rs:60` over hyprland/x11 files.

## Features (waydroid-helper parity + phantom hotkeys)
- Modes COPY `mode_controller.py:60`: `EDIT/MAPPING`, `is_mode_switch_key F1`, `apply/toggle`. waydroid-helper flow KEY_MAPPING.md: Home→Open → transparent overlay Edit default → right-click empty=menu → drag place → handles resize → double-click capture (letters/numbers/F-keys/Ctrl/Alt/Shift, Left/Right/Middle, combos e.g. Ctrl+A) → gear settings → F1 Mapping (minimal indicators) → F1 back. Window `window.py:510 switch_mode`, F12 transparency (`window.py:370`, `window_input_router.py:241`).
- 9 widgets port behavior (sizes from base 50x50 def, layout v1.3 `{version,screen_resolution,widgets[{type,x,y,w,h,text,keys,config}]}` scale-on-load per `widget_layout_service.py:210`): tap (`single_click.py:375`), repeat (`repeated_click.py:593`), aim rect (`aim.py:888`), fire LMB-locked (`fire.py:866`), dpad (`directional_pad.py:791`), skill-cast (`skill_casting.py:1191`), right-walk (`right_click_to_walk.py:692`), macro (`macro.py:975` click/press/release/switch + dragSpeed/startDelay + CancelOnRelease/OneShot + release_actions), + MenuTouch owned cursor.
- Hotkeys phantom: F1 mouse-route, F8 capture, F9 pause, F10 preview, F2 shutdown (`config.rs:413 RuntimeHotkeys + OPERATIONS.md + IPC.md`); cage F10 toggle Waydroid passthrough (`cage-xtmapper/README`); XtMapper Ctrl+P/E/S (`KeymapConfig.java`).
- OSD: pause/FPS/save-state toast. Headless Mapping mode (no GTK overhead). Baseline dark only (egui-material3 toolbar/fab/progress ref, no M3E port here).

## DRY
Calls `wd-input::{load,validate,audit}` (03) + `wd-shell` viewmodels (06). egui draws only. Same structs Slint reads. Scale helper: relative 0-1 → px (QtScrcpy rule), shared fn in wd-input, NOT duplicated.

## Steps
1. Glow window + layer-shell passthrough port. 2. Edit canvas drag/resize/capture. 3. 9 widgets bound to audit API. 4. OSD + hotkeys. 5. Edit-never-grabs / Map-grabs guard test.

## Blast radius
- Touches: `crates/wd-overlay/*` ONLY. No shell/daemon/input logic (calls them).
- Risk: LOW (separate proc). Guard: Edit mode no exclusive grab; Map grabs; F9 emergency release; socket-drop releases.
- Rollback: `pkill wd-overlay`; shell+daemon+game unaffected.
