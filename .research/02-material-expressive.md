# Material 3 Expressive — tokens + Slint port plan

Date: 2026-09-14. M3E = 2025 expansion of M3 (not M4). Sources: m3.material.io, Compose docs, `gersilva96/react-material-expressive` (local), Slint M3 lib (local, baseline only).

## M3E vs baseline M3

- Motion physics: springs replace tweens for spatial. 2 schemes `expressive` (hero moments) / `standard` (rest).
- Shape morphing: 10-step corner scale + 35-shape library (Circle/Square/Pill/Cookie/Clover/Flower/Burst/Slanted...). Buttons morph round↔square on press/select.
- Color: more saturated 2025 spec. New `Fixed/FixedDim/Dim` roles. `surface-container lowest/low/default/high/highest` + `surface-bright/dim`.
- 14 new/updated comps: ButtonGroup (replaces SegmentedButton), FABMenu (replaces speed dial), SplitButton, Docked + Floating Toolbar (replace BottomAppBar), LoadingIndicator wavy (replaces indeterminate circular), 5 button sizes XS-XL, icon buttons XS-XL + narrow/wide + square, FAB S/M/L/extended, updated AppBar/NavBar/Rail/Carousel/Progress/Slider.
- Typography: 15 baseline + 15 emphasized = 30 styles. Same sizes, emphasized = heavier wght + GRAD/ROND tweaks. Variable fonts (Roboto Flex / Google Sans Flex).
- Icons: Material Symbols 2500+, axes FILL/wght/GRAD/opsz/ROND. Desktop dense: 20dp opsz, min wght 200 at 24dp.

## Token sketch (port these)

Color roles (~49, `ColorScheme` source of truth):
```
primary, onPrimary, primaryContainer, onPrimaryContainer, inversePrimary
secondary, onSecondary, secondaryContainer, onSecondaryContainer
tertiary, onTertiary, tertiaryContainer, onTertiaryContainer
background, onBackground, surface, onSurface, surfaceVariant, onSurfaceVariant, surfaceTint
inverseSurface, inverseOnSurface, error, onError, errorContainer, onErrorContainer
outline, outlineVariant, scrim
surfaceBright, surfaceDim, surfaceContainerLowest/Low/_/High/Highest
primary/secondary/tertiaryFixed, _FixedDim, on_Fixed, on_FixedVariant
```
Rules: `on-*` only on paired parent. `container` never for text/icons. Primary=hero FAB, secondary=chips/nav, tertiary=badges.

Shape scale: `None 0 | XS 4 | S 8 | M 12 | L 16 | L-inc 20 | XL 28 | XL-inc 32 | XXL 48 | Full 50%`.
Defaults: buttons Full, FAB/drawer L, small FAB + cards M, chips S, menus/textfields/snackbar XS.

Elevation 0-5 (resting 0-3, 4-5 hover/drag): L3 dialogs/FAB/pickers, L2 menus/navbar/scrolled appbar/toolbar, L1 elevated button/card/chip/sheets/drawer, L0 filled/tonal/outlined buttons, filled/outlined cards, rail, tabs, slider.

Motion (12 tokens `md.sys.motion.spring.{fast,default,slow}.{spatial,effects}` × {expressive,standard}):
- Spatial (pos/size/radius): spring w/ overshoot. fast ~200ms / default ~350ms / slow ~600ms.
- Effects (color/alpha/elev): tween, damping 1.0, no overshoot. fast ~150ms / default ~250ms / slow ~400ms. Never swap.
- Known: expressive fast spatial damping 0.6 / stiffness 800 (Compose tokens). Legacy easings: emphasized `0.2,0,0,1`, decel `0.05,0.7,0.1,1`, accel `0.3,0,0.8,0.15`.

Type (Roboto sizes): display 57/45/36, headline 32/28/24, title 22M/16M/14M, body 16/14/12, label 14M/12M/11M. Emphasized: same size/line, wght 500-700. Brand face for Display/Headline, plain for Body/Label.

Token export tools: Figma Material Theme Builder (Android/Compose/DSP JSON), materialkolor.com, `material-foundation/material-color-utilities` (TS/Java 2025 spec), `deminearchiver/material-color-utilities-rust` (best for Slint port), `makmn1/material-design-token-gen` npm (baseline+expressive CSS).

## What exists in Rust/Slint (gap confirmed)

- Official Slint M3 lib (local, v1.0 Sep 2025): ~50 comps baseline. NO expressive (verified grep: no ButtonGroup/Split/FABMenu/Toolbar/Loading). `MaterialPalette` simplified (bg/fg/alt/control/accent/select/border + surface-container set).
- `moeleak/material-ui-rs` (iced 0.14): baseline only. `nikescar/egui-material3`: baseline only. `FerrisMind/twill`: token infra, no M3E semantics. `nikandlv/material-rs` (Yew WASM): claims expressive, web-only.
- `react-material-expressive` (local `.devdocs/`): only complete M3E web impl. Full 49-role `--md-sys-color-*`, shape/elev/motion scales, ButtonGroupConnected, FABMenu, SplitButton, Toolbars, LoadingIndicator, expressive List tiles. Use as token + behavior reference. Note: it drops NavigationDrawer→expanded Rail, BottomAppBar→DockedToolbar, SegmentedButtons→ButtonGroupConnected — follow same.
- Zero native Slint expressive port. Gap = our build.

## Emulator mapping (dark fixed seed, no dynamic color)

Seed violet `#6750A4` or neon `#00E5CC`, tertiary magenta badges. Surfaces near-black: `surfaceDim` bg, `surfaceContainerLowest/Low` panels, `surfaceContainerHigh/Highest` cards/dialogs.

| Need | M3E comp | Desktop adapt |
|---|---|---|
| Library nav | NavigationRail expanded + FAB | 72-80px collapsed, 200px expanded, 20dp icons |
| Filters | ButtonGroup connected toggleable | XS-S, shape morph on select |
| Game grid | Filled Card M 12dp, surface-container-low | 160-220px tiles, XL 28dp hero, hover +1 elev |
| Play/scan | FAB medium + FAB menu | FAB menu large items, contrasting close |
| Emu actions | DockedToolbar + SplitButton Save | pause/save/load in toolbar, split trailing spins |
| Settings | Switch/Slider, fastEffects color | 32-40px targets (not 48dp touch) |
| ROM details | Modal Dialog XL / BottomSheet | scrim 32%, enter 400ms decel, exit 200ms accel |
| Feedback | Snackbar, Linear Progress, Chips | genre/region chips, snackbar savestate |
| BIOS load | LoadingIndicator wavy | animated arc |

Density: touch 48px → desktop 32-40px. Type: emphasized Title/Headline heroes, baseline Body/Label elsewhere. Motion: expressive spatial only for card expand/sheet/rail morph; standard/effects elsewhere; no bounce during gameplay.

## Port plan to Slint DSL

Slint lacks true springs + path morph. Approximate: spatial → `easing: ease-out-back` / cubic-bezier, effects → `ease-in-out`, shape morph → animate `border-radius` (not RoundedPolygon). Good enough desktop.

1. Generate tokens: MCU-Rust script or materialkolor → `dark.json`. Parse to `.slint` globals.
2. Extend palette (keep `MaterialPalette` compat):
```slint
export global M3EColor {
  out property <color> primary: #D0BCFF;
  out property <color> on-primary: #371E73;
  out property <color> primary-container: #4F378B;
  // ... all 49 + surface-container-lowest/low/high/highest/bright/dim
  out property <color> surface-container-low: #1A1C1E;
}
export global M3EShape {
  out property <length> xs: 4px; out property <length> s: 8px;
  out property <length> m: 12px; out property <length> l: 16px;
  out property <length> xl: 28px; out property <length> xxl: 48px;
}
export global M3EMotion {
  out property <duration> fast-spatial: 200ms;
  out property <duration> default-spatial: 350ms;
  out property <duration> slow-spatial: 600ms;
  out property <duration> fast-effects: 150ms;
  out property <duration> default-effects: 250ms;
}
```
3. New wrappers in-repo: `button_group.slint, split_button.slint, fab_menu.slint, docked_toolbar.slint, loading_indicator.slint` (wavy = animated arc). Wrap existing M3 comps, override colors/shapes. Keep `@material` import path via `build.rs` library_paths (verified pattern).
4. Motion: `animate background { duration: M3EMotion.fast-effects; easing: ease-in-out; }` for effects; `easing: ease-out-back` spatial fake; `animate border-radius { duration: 350ms; }` shape morph.
5. Verify contrast: `on-*` on paired parent only, check dark tonal.

Links: m3.material.io (blog/building-with-m3-expressive, styles/motion, color/roles, shape, elevation, typography, components/fab-menu+buttons), Compose MaterialExpressiveTheme/MotionScheme/ColorScheme/Shapes, `slint.dev/blog/material-comp-1.0`, `material.slint.dev`, gallery WASM, Figma M3 kit + Shapes library, `pub.dev/packages/material_3_expressive`.
