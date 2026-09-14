# 00 Index — full implementation plan

Date: 2026-09-14. Target: Linux Wayland Waydroid gaming emulator. Rust-first.

## Stack verdict (citations)

- Core Rust workspace, daemon+CLI+MCP same core. `.research/00-overview-decision.md:7`, `.research/03-tech-stack.md:8-10`. Start 42ms vs py 690ms vs node 2300ms. `.research/00-overview-decision.md:17`.
- Shell Slint 1.17 `winit-skia`, `MaterialWindow` dark + Theme. `.research/01-slint-ui.md:7-10`, `.research/03-tech-stack.md:11`. Lib 1.17.0, MSRV 1.92, template pins 1.13 bump. `.research/01-slint-ui.md:30`.
- M3E gap 5 wrappers, upstream baseline only. `.research/02-material-expressive.md:45`, `.research/01-slint-ui.md:25`. Zero native Slint expressive port. `.research/02-material-expressive.md:48`.
- Overlay egui glow sidecar 5.6MB LTO 0.1-0.3s. `.research/00-overview-decision.md:19`, `.research/03-tech-stack.md:13`.
- Input evdev grab → JSON → socket → `injectInputEvent ASYNC` 5-10ms. `.research/03-tech-stack.md:72-85`. Never adb per-frame / dispatchGesture. `.research/03-tech-stack.md:85`.
- Root Magisk bootanim.rc only. KSU/APatch no LXC path. `.research/04-root-hide-spoof-2026.md:17`.
- Hide minimal Zygisk+Shamiko+PIFork v18, add Next/Re on kick, Vector+NeoZygisk, HMA iff applist kick. `.research/04-root-hide-spoof-2026.md:27`. ZygiskNext closed since v4-0.9.2 fetch zip. `.research/04-root-hide-spoof-2026.md:21`.
- Frida lab-only, ship Vector sensor-noise. `.research/04-root-hide-spoof-2026.md:43-48`.
- Non-goals: no STRONG, no shipped keybox, no sensor/GPU fake v1, no CV. `.research/00-overview-decision.md:36`, `.research/03-tech-stack.md:21`. Target BASIC+DEVICE. `.research/00-overview-decision.md:37`.

## Reuse inventory (extend, never rewrite)

| Domain | Source | What reuse |
|---|---|---|
| engine/schema | `.devdocs/phantom/phantom/src/profile.rs:13-22`, `engine.rs:TouchCommand/KeymapEngine/NodeState`, `touch.rs:SlotAllocator/TouchDevice`, `input.rs:InputCapture`, `ipc.rs:IpcRequest/Response`, `waydroid.rs:18-30` paths, `android_inject.rs:AndroidInjector`, `inject.rs:UinputDevice`, `contrib/.../PhantomServer.java`, `docs/PROFILES.md,IPC.md,ANDROID_SOCKET_PROTOCOL.md`, `profiles/pubg.json` | profile schema, engine loop, socket frames, waydroid paths |
| wl variant | `.devdocs/phantom-wl/.../wayland_cursor.rs:60`, `engine.rs:2933` KeyDown/Up | wayland cursor replace hyprland+x11 |
| keymap compat | `.devdocs/QtScrcpy/docs/KeyMapDes.md`, `keymap/gameforpeace.json:328`, `ScrcpyKeyMapper/.../pubg.json:587` | switchKey/mouseMoveMap/KMT_* import-only |
| editor UX | `.devdocs/waydroid-helper/docs/KEY_MAPPING.md`, `controller/.../mode_controller.py:60`, `window.py:510`, `widget_layout_service.py:210`, `widgets/base/base_widget.py:512` | F1 Edit/Map, drag/resize/capture, 9 widgets |
| control compat | `.devdocs/waydroid-helper/.../control_msg.py:196` ControlMsgType big-endian | scrcpy-compatible pack fallback |
| in-android ref | `.devdocs/XtMapper/.../server/Input.java:358`, `KeymapProfile.java`, `KeymapConfig.java:swipeDelayMs 10ms` | injectTouch pattern, dpad/mouseaim structs |
| scrcpy proto | `.devdocs/scrcpy/app/src/control_msg.h`, `server/.../control/ControlMessage.java:289`, `input_manager.c:1212`, `doc/video.md:238` no-buffer | frame types, latency baseline 30-70ms |
| slint build | `.devdocs/material-rust-template/build.rs:5-12` library_paths, `ui/main.slint:1-2` @material import, `Cargo.toml:16` slint 1.13 bump 1.17 | build.rs pattern |
| M3 lib | `.devdocs/slint/ui-libraries/material/src/material.slint:5-66` 52 exports, `ui/styling/material_palette.slint:7-169` 49 roles, `material_schemes.slint:4-58`, `material_typography.slint:14-92`, `material_style_metrics.slint:6-74`, `material_animations.slint:5-16` | palette/schemes/type/metrics/motion base |
| M3E tokens | `.devdocs/react-material-expressive/src/styles.css:29-202`, `theme.css:30-192`, `components/index.ts:ButtonGroupConnected/SplitButton/FABMenu/DockedToolbar/LoadingIndicator`, `FOUNDATIONS.md:33-80` | 49 roles/shape/elev/duration/easing/z to port |
| proof apps | `.devdocs/gpcl/res/main.slint:18-91`, `src/{main,config,launcher}`, `.devdocs/slint-template/.../desktop-window.slint`, `.devdocs/sleek-ui/ui/app-window.slint:13-30` | launcher/config/page-manager shape; skip SurrealismUI 1.9.1 stale, gamebub no .slint |
| installer | `.devdocs/waydroid_script/stuff/magisk.py:18-163` bootanim block, `tools/{container,helper,images,logger}.py`, `stuff/general.py:resetprop.rc`, `main.py` CLI | bootanim.rc pattern, overlayfs logic, resetprop inject |
| waydroid core | `.devdocs/waydroid/tools/helpers/props.py`, `actions/{prop,session_manager,container_manager,app_manager}.py`, `data/configs/config_base`, `debian/changelog:1.6.3` | prop get/set, session/container |
| settings persist | `.devdocs/waydroid-settings/.../utils.py:BASE_PROP_LOC,echo_command_to_wd_base`, version 0.3.0 | waydroid_base.prop append |
| spoof props | `.devdocs/waydroid-scripts/spoof-device.sh:19` redfin, `.devdocs/Waydroid-total-spoof/waydroid.sh:20` profiles + android_id/gsf_id, `V2.0.sh:25` verified-boot hide (discard broken sed), `.devdocs/DeviceSpoofLab-Hooks/device_profile.conf:260` cheetah canonical dict | prop dict source = device_profile.conf |
| PIF | `.devdocs/PlayIntegrityFork/module/{autopif4.sh:279,post-fs-data.sh,service.sh,common_func.sh,customize.sh,action.sh,killpi.sh,example.pif.prop}`, `module.prop:v18/180001`, `app/.../cpp/main.cpp:ShadowHook`, `EntryPoint.java` | resetprop lists, autopif generator, native hook (don't fork) |
| tricky | `.devdocs/TrickyStore/update.json:v1.4.1/245`, `.devdocs/Tricky-Addon/module/{post-fs-data.sh,service.sh,prop.sh,push extra}`, `module.prop:v5.0-beta.4/729` vs update v4.4 | late-prop list merge w/ PIF, keybox slot empty |
| vector/zygisk | `.devdocs/Vector/zygisk/.../ParasiticManagerHooker.kt`, `module.prop:v2.2/3080`, `.devdocs/NeoZygisk/loader/src/ptracer/*`, `module:v2.4/289` | parasitic only if needed; NeoZygisk iff Magisk Zygisk missing |
| hooks base | `.devdocs/DeviceSpoofLab-Hooks/app/.../MainHook.java`, `hooks/{BuildHooks,TelephonyHooks,NetworkHooks:30-148,...19 files}`, `cpp/{native_hooks,property_hooks}.cpp`, `module.prop:1.2/4 targetApi101` | whole module base, feed conf |
| sim table | `.devdocs/TikTokSimSpoof/.../MainHook.kt` scope+hooks, `SpoofConfig.kt:60+` MCC/MNC, `1.2.0/2` | carrier table + safe-hook helper |
| prefs pattern | `.devdocs/TelephonySpoofer/.../SpoofInjection.java` hookBool/String/Int via XSharedPrefs, `1.0/1` | runtime UI editing |
| framework scope | `.devdocs/Lsposed-SimSpoof/.../SimSpoofModule.java:750` scope android, hardcoded (discard values) | framework-only scope idea |
| net table | `.devdocs/waydroid_network_spoof/README.md` hasTransport/type/WifiManager/NetworkType table | port into NetworkHooks (hasTransport WIFI+CELL swap missing) |
| verifier | `.devdocs/KeyAttestation/...` `1.8.4`, `status.json` | verify PIF/Tricky success, not spoofer |
| HMA | `.devdocs/HideMyApplist-mirror/README.md:SOURCE_URL Dr-TSNG/Hide-My-Applist` mirror only | upstream direct, SimSpoof already ports PM filter |
| MCP tools | `.devdocs/scrcpy-mcp/src/tools/{session,input,files,device,vision,clipboard,video,ui,apps,shell}.ts:39`, `src/index.ts:StdioServerTransport`, `AGENTS.md` registerTool+zod | tool shapes, stdio pattern |
| MCP waydroid | `.devdocs/waydroid-mcp/src/waydroid_mcp/{server.py:6 tools,screen.py:strip_png/raw_to_jpeg,adb.py:39-51 frozen,core.py:43-95 --no-tree,cli.py}` | frozen detect, rm-first dump, Core dual-use |
| MCP adb | `.devdocs/adb-mcp/adb_mcp/{core.py:quote_argv,tools/media.py:screencap CRLF repair,tools/logs.py:Popen temp,tools/ui.py}` 90 tools | adb wrapper, screenshot 3 surfaces, logcat bg |
| MCP misc | `.devdocs/android-mcp-server-us/src/{index.ts:http opt-in,tools/ui.ts:sharp,tools/logcat.ts,tools/utils.ts:shellEscape}` 76 tools; `...-mg/src/index.ts:25` snake; `Android-MCP/.../tree/service.py:annotated_screenshot` uiautomator2 | http upgrade, sharp pipeline, annotated shot |

## Files

- `01-workspace-foundation.md` — workspace, wd-core, lints, gate.
- `02-core-daemon.md` — supervisor, session, props, IPC.
- `03-input-keymap.md` — schema, engine, inject, audit.
- `04-waydroid-mgmt.md` — sidecar, prop swap, apps, backup.
- `05-root-hide-spoof.md` — magisk, shamiko, PIF, tricky, vector modules.
- `06-shell-slint.md` — shell, ui-shared, M3E wrappers, viewmodels.
- `07-overlay-egui.md` — glow editor + OSD.
- `08-cli-mcp-skills.md` — 24 tools, transports, skills.
- `09-quality-packaging.md` — tests, clippy, packaging, order.

## S-tier rules (all plans)

- Extend never rewrite. One schema / one prop engine / one tool core / one UI viewmodel.
- No monolith: lib crates ≤800 lines, bins ≤150 lines, .slint files ≤400 lines.
- DRY: wd-core types only shared; ui-shared.slint single component source; egui reads same viewmodels.
- 0/0: `deny warnings`, clippy all+pedantic+nursery, rustfmt, `cargo test --workspace`.
- Blast radius per plan: touches + risk + rollback. One plan per batch, no cross-plan edits.
