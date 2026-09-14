# 06 Shell Slint — MaterialWindow dark + M3E wrappers + shared UI

Refs: `.research/01-slint-ui.md:12-89`, `.research/02-material-expressive.md:41-97`, `.research/03-tech-stack.md:136-151`; `.devdocs/slint/ui-libraries/material/src/material.slint:5-66` (52 exports), `ui/components/*` (50+ files), `ui/styling/{material_palette.slint:7-169,material_schemes.slint:4-58,material_typography.slint:14-92,material_style_metrics.slint:6-74,material_animations.slint:5-16}`, `examples/gallery/ui/{main.slint:20-30,views/*}`; `.devdocs/material-rust-template/{build.rs:5-12,Cargo.toml:16 slint1.13 bump,ui/main.slint:1-17,src/{main,lib}.rs}`; `.devdocs/react-material-expressive/src/{styles.css:29-202,theme.css:30-192,components/index.ts:25-206,styles.test.ts:10-19,FOUNDATIONS.md:33-80}`; `.devdocs/gpcl/{res/main.slint:18-91,res/style.slint:1-13,src/{main,config,launcher},Cargo.toml:slint1.15.1 femtovg}`; `.devdocs/slint-template/.../{desktop-window.slint:21-33,android-window,web-window,base,panel,store.slint},Cargo.toml:53 slint1.17`; `.devdocs/sleek-ui/{ui/app-window.slint:13-30,ui/sleek-ui/app-theme.slint:6-60,Cargo.toml:slint1.17.1 skia}`; `.devdocs/SurrealismUI/{index.slint:45-99,surrealism-ui.toml:slint1.9.1 STALE}`; `.devdocs/egui-material3/src/{lib,theme,toolbar,fab,progress}.rs` (ref only); `.devdocs/slint/Cargo.toml:85 v1.18.0 rust1.92`.

## Goal
Beautiful dark gaming shell. Shared components once in `ui-shared.slint`. No M3 fork. Slint 1.17 `winit-skia`.

## Setup
- `slint 1.17 + slint-build 1.17` (NOT template 1.13, NOT core 1.18.0 unreleased). `SLINT_BACKEND=winit-skia`, `SLINT_STYLE=material-dark`. Backend order winit > linuxkms cabinet later. MSRV 1.92.
- `ui/build.rs` COPY `material-rust-template/build.rs:5-12` library_paths (`material→material-1.0/material.slint`) + `compile_with_config("ui/main.slint")`. Root import `import {FilledButton,ElevatedCard,MaterialWindowAdapter} from "@material"` per template `ui/main.slint:1-2` / minimal `ui/main.slint:4-17` counter.
- License: desktop proprietary = royalty-free + AboutSlint badge; open = GPLv3 whole binary; handheld/cabinet = likely embedded commercial ask info@slint.dev (`.research/01-slint-ui.md:80-85`). M3 lib MIT no trigger.

## Shared UI — DRY single source (`ui/ui-shared.slint`)
`GameCard/FilledCard M 12dp` (art image async + title + StatusChip), `StatusChip` (genre/region/state), `PropRow` (key/value + copy), `LogView` (virtualized, follows daemon stream), `DeviceDot` (ready/frozen/offline). ALL pages import from here. No duplicate card/chip code. egui overlay reads same Rust viewmodels (07), never duplicates logic.
- Grid: `ListView` virtualized + `for` + Rust `VecModel` — 10k ROMs, async covers via `slint::Image` (`.research/01-slint-ui.md:54`). OSD frame `Image::from_rgba8` CPU upload retro-OK; GPU texture share later.

## M3E tokens + 5 wrappers (gap confirmed: grep ButtonGroup/Split/FABMenu/Toolbar/Loading = 0 hits)
Port from `react-material-expressive`:
- `m3e_color.slint`: 49 roles from `styles.css:29-78 light + 211-263 dark` → `@theme` map `theme.css:30-88`. Keep `MaterialPalette` compat (palette 173 lines light 67-118 dark 119-169 selector :172 dark?dark:light).
- `m3e_shape.slint`: `styles.css:81-90` none..full → metrics `radius 2,4,8,12,16,28` + XS4 S8 M12 L16 XL28 XXL48 Full50%.
- `m3e_motion.slint`: durations `styles.css:167-182` short1..extra-long4 + easings `111-117` emphasized/standard + expressive `linear()` springs `119-164` → Slint approx `material_animations.slint:5-16` (emphasized cubic 0.05,0.7,0.1,1 500ms / standard 0,0,0,1 150-250ms): spatial `ease-out-back` 350ms, effects `ease-in-out` 150-250ms (`.research/02-material-expressive.md:70`). Shape morph animate `border-radius` NOT RoundedPolygon.
- `m3e_type.slint`: `theme.css:131-192` display/headline/title/body/label + `typography.slint:14-92` sizes + weights regular300/medium600/semibold900 (30 styles: 15 baseline + 15 emphasized wght 500-700).
- z/state guard `styles.test.ts:OVERLAY_TOKENS` + `FOUNDATIONS.md:46-80` (state 0.08/0.12/0.16, z base0..snackbar70).
- 5 wrappers wrap existing (NOT fork): `button_group.slint` (wrap SegmentedButton `segmented_button.slint:10 SegmentedItem` → Connected per index.ts:25-28), `split_button.slint` (index.ts:48-53), `fab_menu.slint` (extend `floating_action_button.slint:10-16` FABStyle small/standard/large + expand), `docked_toolbar.slint` (AppBar/BottomAppBar → Docked/Floating per index.ts:200-206), `loading_indicator.slint` (extend `progress_indicator.slint:7,35` linear/circular → wavy arc per index.ts:135-139). Density 32-40px NOT 48dp touch. Emphasized Title/Headline heroes only. Expressive spatial only card/sheet/rail, rest standard, no bounce gameplay.

## Pages (MaterialWindow dark `MaterialPalette.background`)
Rail 72-200px NavigationRail+FAB (material.slint:36,22) | ButtonGroup filters XS-S | Grid Filled/Elevated/OutlinedCard (sint:16) | FAB medium+menu Play/scan | DockedToolbar+SplitButton pause/save/load | Dialog/FullscreenDialog + Modal/BottomSheet scrim 32% (sint:11,15,54) | SnackBar+Chips(Action/Filter/Input sint:9)+LinearProgress | SearchBar + TabBar + Switch/Slider/TextField/DropDownMenu + Date/TimePicker | Tooltip + Tray 1.17 SystemTrayIcon | NavigationBar/Drawer/Rail + AppBar S/M/L.
- Launcher shape from `gpcl/res/main.slint:18-91 + style.slint:1-13 + src/launcher` (gamepad launcher). Settings scaffold `slint-template/.../desktop-window + panel + store.slint`. Page-manager `sleek-ui/app-window.slint:13-30`. Gallery ref `gallery/ui/views/*`. Do NOT pull SurrealismUI (slint 1.9.1 stale) / sleek-ui tiny / gamebub (FPGA, no .slint) / material-ui-rs (iced, not Slint).

## Rust bridge (`wd-shell` lib + thin bin)
Viewmodels ONLY: `GameList/DeviceState/SpoofProfile/KeymapState/LogStream`. Slint callbacks → daemon IPC (02). Zero Waydroid logic in UI. Screenshot test per page.

## Blast radius
- Touches: `ui/*`, `crates/wd-shell/*` only. No daemon/input/spoof/MCP.
- Risk: LOW-MED (1.13→1.17 bump). Guard: pin 1.17, gallery builds first, per-page shots.
- Rollback: revert `ui/`; daemon headless via CLI still works.
