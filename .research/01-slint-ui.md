# Slint UI — libs, kits, verdict for emulator shell

Date: 2026-09-14. Verified against local `.devdocs/slint`, `material-rust-template`.

## Core (verified local)

- Repo: `.devdocs/slint` (slint-ui/slint). Lib version: `1.17.0` (`.devdocs/slint/ui-libraries/material/Cargo.toml:28`). Rust MSRV 1.92 (workspace).
- Renderers: Skia (default, GL/Vulkan/Metal/D3D, WGPU unstable) / FemtoVG (GL, wgpu) / Software CPU. Select via `SLINT_BACKEND=winit-skia|winit-femtovg|winit-software`. Docs: `docs.slint.dev/.../backends_and_renderers/`.
- Backends: winit (Wayland+X11 Linux, Win, macOS) / linuxkms (no compositor, arcade-cabinet mode) / Qt (QPainter). Pick `winit-skia` on Wayland desktop.
- Lang APIs: Rust primary (`slint` + `slint-build`), C++, Node (beta), Python (beta). `.slint` AOT compiles to native.

## Official M3 lib (verified: 52 component files, MIT)

Path: `.devdocs/slint/ui-libraries/material/src/`. Entry: `src/material.slint` (66 lines, all exports).
License: MIT (`src/LICENSE.md`) — NOT GPL like core. No extra trigger.

Components (from `material.slint` exports):
- Buttons: Filled/Elevated/Tonal/Outline/Text, SegmentedButton, FAB (`floating_action_button.slint`), Filled/Tonal/Outline/IconButton
- Nav: NavigationBar/Rail/Drawer(+Modal), BottomAppBar, Small/Medium/Large AppBar, TabBar(+Secondary), SearchBar
- Cards: Elevated/Filled/Outlined (`card.slint`); Dialog(+Fullscreen), Modal, ModalBottomSheet, Drawer(+Modal)
- Inputs: TextField, CheckBox(+Tile), RadioButton(+Tile), Switch, Slider, Chip (Action/Filter/Input), DropDownMenu, PopupMenu, Date/TimePickerPopup
- Feedback: SnackBar, Badge, Linear/Circular ProgressIndicator, ToolTip, Divider, ListView/ScrollView/Grid, Avatar/ListTile
- Root: MaterialWindow (+MaterialWindowAdapter `disable_hover`), MaterialPalette/Scheme(s)/Typography/Animations/StyleMetrics

Missing (grep `button.group|split|fab.menu|toolbar|loading` → NO hits): all M3 Expressive comps — ButtonGroup, SplitButton, FABMenu, Docked/FloatingToolbar, LoadingIndicator (wavy). Baseline M3 only.

Theming:
- `MaterialPalette` global (`ui/styling/material_palette.slint`, 173 lines): full 49-role scheme incl `surface_dim/bright`, `surface_container_lowest/low/_/high/highest`, `primary/secondary/tertiary_fixed(_dim)`, state-layer opacities, `background_modal`. Delegates to `MaterialSchemes {light, dark}` struct (59 lines, `material_schemes.slint`).
- Usage (verified `material-rust-template`): `build.rs` maps `"material"` → `material-1.0/material.slint` via `CompilerConfiguration::with_library_paths`; `ui/main.slint` does `import { FilledButton, ElevatedCard } from "@material"`.
- Template `Cargo.toml` pins `slint 1.13` — old. Bump to `1.17` for new work.

## Styling system (custom design system path)

```slint
export global Theme {
  in-out property <brush> bg: #0d1117;
  in-out property <brush> card: #161b22;
  in-out property <brush> accent: #00d4ff;
  in-out property <length> radius: 12px;
}
export component GameCard inherits Rectangle {
  in property <string> title; in property <image> art;
  background: Theme.card; border-radius: Theme.radius;
  border-color: touch.has-hover ? Theme.accent : transparent;
  animate background, border-color { duration: 150ms; easing: ease-out; }
  touch := TouchArea {}
  VerticalLayout { Image { source: art; } Text { text: title; } }
}
```

- Brushes: solid, `@linear-gradient(...)`, `@radial-gradient(...)`; `drop-shadow-*`, `border-*`, `clip`.
- SVG via `Image { source: @image-url("icon.svg"); }` + `Path`. Fonts: `import "./Font.ttf";` + `default-font-family`.
- Dark force: `Palette.color-scheme = dark` or `SLINT_STYLE=material-dark`. Branch custom: `Palette.color-scheme == dark ? ...`.
- Grids: `ListView` virtualized + `for` + Rust `VecModel` — handles 10k ROMs.
- Std-widget styles (`SLINT_STYLE` compile-time): fluent/material/cupertino/cosmic/qt/native, each `-light/-dark`. `material-*` here = thin skin, NOT the M3 lib above.

## Community kits (local + web)

- SurrealismUI (`.devdocs/SurrealismUI`, 377★ MIT): 40+ comps (SButton/SCard/SMenu/STab/STable/STree/SDialog/SDrawer/SPopup/SAlert/SBadge/STag/SProgress/SSlider/SSwitch/SInput/SSelect/SCalendar/SPagination/SCarousel/STimeline/SAvatar...), 7 theme colors + light/dark. Stale since 2024-12 — pin fork if use.
- sleek-ui (`.devdocs/sleek-ui`, 55★ MIT): Ant-Design-based, tiny (`UButton, UText, UAppTheme.spacing`). Settings-pages only.
- vivi (codeberg.org/vivi-ui/vivi, MIT): desktop lib, used by game engine `musi lili`. No local clone.
- slint-template (`heng30`, local): settings panel + config + sqlite pattern. Good scaffold ref.
- No gaming/dark-emulator kit exists. Start from official M3 lib dark, add custom `Theme` global for neon accent.

## Emulator UI effort

- Sidebar nav: easy. Material `NavigationRail/Drawer`. 1 day.
- Game grid: easy-med. `ListView` + Rust model, async covers via `image`. 2-3 days.
- Settings: easy. M3 Switch/Slider/TextField/DropDownMenu. 2 days.
- Dialogs/snackbar/tooltip/tray: built-in (`Dialog`, `SnackBar`, `ToolTip`, 1.17 `SystemTrayIcon`). Easy.
- OSD overlay: med. Borderless `Window` + `PopupWindow`; frame via `slint::Image::from_rgba8` CPU upload (OK retro res) or Skia/wgpu texture share (custom work).
- Keymap editor canvas: med. `FocusScope + TextInput` capture, SVG gamepad + `TouchArea`, Rust `evdev/gilrs` feed callbacks. 3-5 days. (Editor could be egui-glow sidecar instead — see 00-overview.)
- Hard bits live in Rust/winit layer, not Slint: global hotkeys unfocused, exclusive fullscreen/vsync.

## Prod proof

- Game Bub (FPGA retro handheld, Slint UI), Chiptrack (GB synth), GPCL (gamepad launcher), cargo-ui, WSL Dashboard, Zeedle (dark player). Commercial: OTIV rail, SK Signet EV HMI, WesAudio plugins.

## License triggers

- Core triple: GPLv3 OR Royalty-free-2.0 OR Commercial-3.0.
- Free/proprietary emulator on general-purpose Linux desktop = NOT embedded → Royalty-free free, add `AboutSlint` widget or MadeWithSlint badge. No source disclose.
- Open-source build → GPLv3, whole binary GPL.
- Preinstalled handheld/cabinet (task-specific device) = likely embedded → GPLv3 (open it) or pay commercial + per-device royalty. Ask info@slint.dev before HW ship.
- M3 lib alone = MIT, no trigger.

## Verdict

Shell = `MaterialWindow` dark + custom `Theme` global for accent. `winit-skia` backend. Bump template to slint 1.17. Build missing expressive comps (ButtonGroup/Split/FABmenu/Toolbar/Loading) as thin wrappers in-repo (see 02-material-expressive.md). ponytail: need it? yes. exists? official M3 lib covers ~80%. minimum: wrap, don't fork.
