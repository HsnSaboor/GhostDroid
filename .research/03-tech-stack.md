# Proposed stack — full system + flows

Date: 2026-09-14. Target: Linux Wayland, Waydroid LXC backend, gaming.
Refs: `.research/00-overview-decision.md:1`, `.research/01-slint-ui.md:1`, `.research/02-material-expressive.md:1`.

## Stack table

| Layer | Choice | Why | Reuse |
|---|---|---|---|
| Core/daemon/CLI/MCP | Rust workspace, tokio, zbus, evdev, wayland-client, gilrs | 42ms start, static bin, no GC pause aim path | phantom engine+profiles |
| Shell UI | Slint 1.17, `MaterialWindow` dark + custom Theme, `winit-skia` | only beautiful+Wayland+a11y today | official M3 lib MIT, 52 comps |
| Expressive gap | 5 in-repo wrappers | official M3 = baseline only | react-material-expressive tokens |
| Overlay/keymap editor | egui glow sidecar, layer-shell passthrough | 5.6MB LTO, 0.1-0.3s start, drag-keys win | — |
| Input inject | host evdev grab → JSON profile → socket → `app_process` `injectInputEvent ASYNC` | 5-10ms, persistent socket | phantom `android_socket`, scrcpy pattern |
| Waydroid installs | py-sidecar calls `waydroid_script` JSON | don't rewrite installer | waydroid_script, libndk/houdini |
| Spoof store | `waydroid_base.prop` per-profile swap + `waydroid prop set` | canonical persistent overrides | Waydroid-total-spoof prop list |
| Root/hide | Magisk/Zygisk → Shamiko → PIF → TrickyStore(user keybox) → LSPosed+HMA | BASIC+DEVICE gaming target | PIF fork, TrickyStore, HMA |
| AI face | CLI+MCP same core, stdio default, http opt-in, ~24 tools | no replay drift | waydroid-mcp lessons |
| Skills | `waydroid-control` + `waydroid-spoof` SKILL.md | observe→locate→act→re-observe | — |

Non-goals v1: no STRONG promise, no shipped keybox, no sensor/GPU fake, no CV smart-key.

## System overview

```mermaid
graph TB
  U[User + Agent] --> SH[Shell Slint<br/>MaterialWindow dark M3E<br/>winit-skia]
  U --> OV[Overlay egui glow<br/>keymap editor OSD<br/>layer-shell passthrough]
  SH <--> DA[Core daemon Rust<br/>tokio + zbus + evdev]
  OV <--> DA
  CLI[CLI waydroid-ctl] <--> DA
  MCP[MCP server stdio/http<br/>same core] <--> DA
  SK[Skills<br/>control + spoof] --> MCP
  SK --> CLI
  DA <--> WC[Waydroid LXC<br/>binder + hwcomposer<br/>InputFlinger]
  DA --> PY[py-sidecar<br/>waydroid_script JSON child<br/>cgroup kill + timeout]
  PY --> WC
  DA --> AP[app_process server<br/>in-container<br/>injectInputEvent ASYNC]
  AP --> WC
```

## Workspace crates

```mermaid
graph LR
  subgraph core-rs
    D[daemon]
    EV[evdev engine]
    WCTL[waydroid ctl<br/>lxc + binder + dbus]
    SP[spoof profiles<br/>prop templates]
    KM[keymap schema<br/>QtScrcpy compat]
    MC[mcp tools ~24]
  end
  subgraph shell-slint
    MW[MaterialWindow dark]
    M3E[M3E wrappers x5<br/>btngroup split fabmenu toolbar loading]
    TH[Theme global<br/>neon accent]
    GD[game grid<br/>ListView + VecModel]
  end
  subgraph overlay-egui
    ED[keymap canvas<br/>drag resize capture]
    OSD[OSD pause/fps/save]
  end
  PY2[py-sidecar] --> D
  D --> WCTL
  D --> EV
  EV --> KM
  MW --> D
  ED --> D
```

## Input hotpath (FPS-critical)

```mermaid
flowchart LR
  K[KB/mouse/gamepad<br/>evdev exclusive grab<br/>~1ms] --> P[Profile engine Rust<br/>tap/joystick/drag/aim<br/>macro/layer_shift<br/>~1ms]
  P --> S[Unix/TCP socket<br/>persistent<br/>~1-2ms]
  S --> J[app_process Java<br/>MotionEvent.obtain<br/>DOWN/MOVE/UP batch]
  J --> I[InputManager.injectInputEvent<br/>ASYNC<br/>~2-5ms]
  I --> G[Game touch pipeline]
  AIM[relative-pointer<br/>pointer-lock] --> P
  style K stroke-width:2px
```

Latency budget: evdev 1 + profile 1 + socket 1-2 + inject 2-5 = 5-10ms. Display round-trip dominates. Never `adb shell input` per-frame (200-1000ms spawn). Never `dispatchGesture` for FPS look.

Fallbacks: uinput MT device (`persist.waydroid.uevent`, fragile restart) → Wayland native forward (`fake_touch` per package, best native KB games) → macros only via adb.

## Waydroid mgmt + session flow

```mermaid
flowchart TD
  B[boot --wait --frozen-check] --> L[lxc-start + binder mounts]
  L --> H[hwcomposer wayland-hwc<br/>xdg_toplevel + vsync 60Hz]
  H --> IN[input pipes<br/>INPUT_KEYBOARD/TOUCH<br/>MT_SLOT/TRACKING_ID/POS]
  IN --> RDY[session ready<br/>/run/waydroid/wayland-0]
  RDY --> APP[launch PKG<br/>fake_touch/fake_wifi per glob]
  APP --> KEY[keymap load profile.json]
  KEY --> PLAY[play + OSD]
  PLAY --> SV[save-state backup/restore]
```

```mermaid
sequenceDiagram
  participant D as daemon
  participant W as waydroid CLI/LXC
  participant A as Android container
  D->>W: session stop
  D->>W: swap waydroid_base.prop (per-game)
  D->>W: waydroid upgrade --offline
  D->>W: session start + wait
  D->>A: clear gms + game data
  D->>A: keymap load + fake_touch/wifi props
  D->>A: launch PKG
```

Prop files: `/var/lib/waydroid/waydroid_base.prop` canonical, `/var/lib/waydroid/waydroid.cfg` alt, runtime `waydroid prop set persist.waydroid.*`, images `system.img/vendor.img/rootfs/data/*`.

## Spoof + root/hide layers (order matters)

```mermaid
graph TB
  L1[1 base props<br/>Pixel redfin fingerprint<br/>release-keys user<br/>debuggable=0 secure=1] --> L2[2 ARM bridge<br/>libndk/houdini<br/>native.bridge + abilist + binfmt]
  L2 --> L3[3 Magisk/Zygisk<br/>bootanim.rc inject<br/>DenyList]
  L3 --> L4[4 Shamiko<br/>Enforce OFF<br/>hide Zygisk]
  L4 --> L5[5 PIF pif.json<br/>certified fp<br/>auto-update 4-6wk]
  L5 --> L6[6 TrickyStore opt<br/>user keybox slot<br/>never ship]
  L6 --> L7[7 LSPosed + HMA<br/>hide applist per PKG]
  L7 --> T[Target BASIC+DEVICE<br/>gaming only]
```

Root toggle: enable = ensure `bootanim.rc` Magisk block + restart; disable = restore `original_bootanim`, purge `overlay*/magisk*`, clear `/data/adb/magisk.db`. Prefer per-app DenyList+Shamiko blacklist+HMA template over global off.

Verify: YASNAC, Play Integrity Checker, Applist Detector. Known gap v1: SwiftShader/ANGLE GPU string, zero-noise sensors, fixed battery.

## Shell UI tree (Slint)

```mermaid
graph TB
  MW2[MaterialWindow dark<br/>background=MaterialPalette.background] --> R[Rail 72-200px<br/>NavigationRail + FAB]
  MW2 --> BG[ButtonGroup filters<br/>XS-S morph select]
  MW2 --> GRID[Game grid<br/>Filled Card M 12dp<br/>ListView virtualized]
  MW2 --> FABM[FAB medium + FABMenu<br/>Play/scan]
  MW2 --> DT[DockedToolbar + SplitButton<br/>pause/save/load]
  MW2 --> DL[Dialog XL / BottomSheet<br/>ROM details<br/>scrim 32%]
  MW2 --> SB[SnackBar + Chips + LinearProgress<br/>savestate + genre]
  MW2 --> TH2[Theme global<br/>bg/card/accent/radius]
  M3E2[M3EColor/M3EShape/M3EMotion<br/>49 roles + shape + motion] --> TH2
```

Motion approx (no Slint springs): spatial `ease-out-back` 350ms, effects `ease-in-out` 150-250ms, shape morph animate `border-radius`. Density 32-40px desktop (not 48dp touch). Backend `SLINT_BACKEND=winit-skia`. Tray 1.17 `SystemTrayIcon`.

## CLI + MCP same core

```mermaid
graph TB
  CORE[core-rs lib] --> C1[devices boot shutdown freeze status]
  CORE --> C2[install uninstall list-apps launch stop current]
  CORE --> C3[tap swipe long-press drag pinch key text screenshot record]
  CORE --> C4[logcat shell prop dumpsys keymap spoof backup restore ui.dump/find]
  CORE --> CL[CLI binary]
  CORE --> MS[MCP stdio default]
  MS --> HT[http opt-in POST /mcp<br/>SSE progress]
  AG[Agent loop<br/>observe-locate-act-reobserve] --> MS
  AG --> CL
```

Tools ~24: `device.list/boot/status, app.install/uninstall/start/stop/list, activity.current, input.tap/swipe/key/text, vision.screenshot/stream, ui.dump/find, shell.exec, file.push/pull, logcat.dump/start/stop, prop.get/set, keymap.load, spoof.load`. Screenshot returns MCP `image` base64 + `android://device/frame/latest.jpg` @2FPS. Logcat handle + poll + redact tokens.

## Build/packaging

```mermaid
graph LR
  CR[cargo build] --> DEB[cargo-deb]
  CR --> RPM[rpmbuild]
  CR --> PKG[PKGBUILD]
  CR --> FL[flatpak freedesktop rust ext]
  CR --> MUSL[static musl<br/>300KB-2MB bin]
  SL[slint-build<br/>@material library_paths] --> CR
  PY3[py-sidecar<br/>supervised child] --> CR
```
