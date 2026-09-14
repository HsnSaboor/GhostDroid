# 03 Input + keymap — 5-10ms hotpath, single schema

Refs: `.research/03-tech-stack.md:72-87`, `.research/00-overview-decision.md:25`; `.devdocs/phantom/phantom/src/{profile.rs:1375,engine.rs:2915,input.rs:2234,touch.rs:124,ipc.rs:1125,android_inject.rs:314,inject.rs:807,mouse_touch.rs:368,overlay.rs:387,config.rs:413,main.rs:799}`, `docs/{PROFILES.md:466,IPC.md,ANDROID_SOCKET_PROTOCOL.md:139,ARCHITECTURE.md:430,PROTOCOL.md:83}`, `contrib/android-server/.../PhantomServer.java:344`, `profiles/{pubg.json:4080B,genshin.json,...8 files}`; `.devdocs/phantom-wl/.../{engine.rs:2933,wayland_cursor.rs:60,PhantomWaylandServer.java:406}`; `.devdocs/QtScrcpy/docs/KeyMapDes.md:94`, `keymap/{gameforpeace.json:328,identityv.json:237,FRAG.json:119,tiktok.json:66}`, `ScrcpyKeyMapper/.../pubg.json:587`; `.devdocs/waydroid-helper/docs/KEY_MAPPING.md`, `controller/.../{mode_controller.py:60,window.py:510,widget_layout_service.py:210,widgets/base/base_widget.py:512}`, `controller/core/control_msg.py:196`; `.devdocs/XtMapper/.../server/Input.java:358`, `keymap/{KeymapProfile.java,KeymapConfig.java:swipeDelayMs 10ms}`; `.devdocs/scrcpy/app/src/control_msg.h`, `server/.../control/ControlMessage.java:289`, `input_manager.c:1212`, `doc/video.md:238` no-buffer; `.devdocs/wayland-getevent/client.c:461`.

## Goal
Host evdev grab → profile → persistent socket → `injectInputEvent ASYNC`. Budget 5-10ms. FPS aim via pointer-lock. Never per-frame adb.

## Schema (`wd-input::schema`, ONE canonical)
Port `profile.rs:13-22` exactly, then normalize legacy:
```rust
Profile{name:String, version:u32 /*==1*/, screen:Option<ScreenOverride>{width,height}, global_sensitivity:f64, nodes:Vec<Node>}
RelPos{x:f64,y:f64} // [0,1], pixel=round(rel*screen), clamp 0..w-1 — QtScrcpy rule KeyMapDes.md
Node #[serde(tag="type")]::{Tap|ToggleTap{slot:u8,pos,key,layer}, Joystick{id,slot,pos,radius:(0,1],keys:{up,down,left,right}}, Drag{id,slot,start,end,key,duration_ms:u64>0}, MouseCamera/Aim{id,slot,anchor(.75,.5),reach(.18),sensitivity>0,curve:Linear/Precision/Balanced,activation:AlwaysOn/WhileHeld/Toggle,activation_key,invert_y}, RepeatTap{interval_ms>0}, Wheel{up_slot!=down_slot,up/down_pos}, Macro{key,mode:CancelOnRelease/OneShot,sequence:Vec<MacroStep{Down/Up,pos,slot,delay_ms}>}, LayerShift{key,layer_name,mode:Hold/Toggle,suspend_base}}
```
- Validation = `Profile::validate/load/normalized/audit` from profile.rs: version==1, screen required non-zero (must match daemon else reject), sensitivity>0, ids unique, slots unique 0..255 logical max 10 concurrent (`touch.rs:MAX_CONCURRENT_TOUCHES=10`, `SlotAllocator{logical_to_physical}`), coords [0,1], keys known. Legacy `hold_tap→tap`, `mouse_camera+region→aim`.
- Seed: copy 8 `profiles/*.json` (pubg/pubg-small/genshin/efootball-template/temple-run/subway/asphalt8/9) to `profiles/keymap/`.
- QtScrcpy import-only (no dual store): `switchKey`, `mouseMoveMap{startPos,speedRatio>=0.00225,speedRatioX/Y>=0.001,smallEyes{KMT_CLICK}}`, `keyMapNodes[KMT_CLICK{key,pos,switchMap}/CLICK_TWICE/CLICK_MULTI{clickNodes[{delay,pos}]}/DRAG{startPos,endPos,dragSpeed 0-1,startDelay}/STEER_WHEEL{centerPos,4 keys,4 offsets}]` per KeyMapDes.md + gameforpeace center 0.16,0.75 offsets 0.1/0.27/0.2. Keys Qt enum → evdev map. Import test: `gameforpeace.json` + `ScrcpyKeyMapper pubg.json (w2336 h1080, dragSpeed 0.4-0.7)` roundtrip.

## Engine (`wd-input::engine`, port phantom)
- Port `engine.rs: KeymapEngine{profile,key_bindings:HashMap<Key,Vec<usize>>,joystick_bindings,states:Vec<NodeState>,pressed_keys,sensitivity,paused,active_layers}` + `NodeState{Tap{pending_release},Joystick{bias,finger_active,origin},Drag{running,progress},MouseCamera{enabled,current_x/y},RepeatTap,Macro{step_index,active_slots},LayerShift{held}}` + `TouchCommand{Down{slot,x,y},Move,Up,Commit}` (+ wl `KeyDown{keycode,repeat,meta}/KeyUp` from phantom-wl engine.rs:40-47 if needed).
- Capture `input.rs: InputCapture::poll_events(0)` epoll non-block. Timing: `main.rs:ENGINE_TICK=4ms`, `input_interval=1ms`, `STANDARD_TAP_MIN_PULSE=20ms`, `JOYSTICK_RELEASE_GRACE=30ms`, `AIM_IDLE_TIMEOUT=500ms`, `AIM_RELATIVE_BASE_SCALE=1/650`, `AIM_ABSOLUTE_SCALE=1/500`, caps rel 0.18/abs 0.08, `TOUCHPAD_TAP_MAX=180ms`. No time smoothing (ARCHITECTURE.md).
- Aim: `wayland-getevent/client.c` relative-pointer+pointer-constraints; `phantom-wl/wayland_cursor.rs:60` replaces hyprland_cursor 220+hyprland_ipc 110+x11_cursor 368. Test KDE first (Waydroid #2073 nested edge).
- Throttle to touch refresh, avoid queue pile-up (scrcpy #3275). Reference QtScrcpy/scrcpy latency 30-70ms USB (`QtScrcpy/README:23`, `scrcpy/doc/video.md`); our inject-only path excludes encode.

## Inject (`wd-inject` + `java/`)
- Primary android_socket: port `android_inject.rs:AndroidInjector{stream:TcpStream,set_nodelay,2s rw timeout,180s auto-launch}` + `waydroid.rs:ensure_android_server/host/port/log_excerpt` + `desktop_relay.rs:204` framing. Frames LE per ANDROID_SOCKET_PROTOCOL.md:139: `0x00 DOWN [slot u8,x i32,y i32]=10B, 0x01 MOVE, 0x02 UP [slot]=2B, 0x03 CANCEL, 0x7F PING`. Server port `27183` bind `0.0.0.0`, host=container IP via `waydroid status`. Coords `round(rel*max)` clamp.
- Server: port `PhantomServer.java:344` (`PointerState[10], MotionEvent.obtain(downTime,now,action,pointerCount,props,coords,SOURCE_TOUCHSCREEN)` + reflection `InputManager.injectInputEvent ASYNC`, pressure 1f). wl variant `PhantomWaylandServer.java:406`. Build to `/usr/lib/wd/wd-server.jar`. No auto-reconnect if dies — daemon restarts (02 supervisor).
- Fallback uinput: port `inject.rs:UinputDevice` (`/dev/uinput`, name Phantom Virtual Touch VID 0x1234 PID 0x5678, EV_ABS/KEY/SYN, ABS_MT_SLOT/TRACKING_ID/POS_X/Y/TOUCH_MAJOR/PRESSURE, BTN_TOUCH, INPUT_PROP_DIRECT, MT-B, tracking monotonic, SYN batching, 50ms sleep after UI_DEV_CREATE per PROTOCOL.md:83). Needs Waydroid restart after start — document.
- Cross-check: scrcpy `control_msg.h + ControlMessage.java:289 (TYPE_* 0-22)` + `input_manager.c:1212`; waydroid-helper `control_msg.py:196` scrcpy-compat big-endian `>BBIII` + `scale_coordinates` int div (fallback pack only); XtMapper `Input.java:358 injectTouch` + `KeymapConfig.swipeDelayMs=10ms`.
- Order: android_socket → uinput MT (`persist.waydroid.uevent`) → Wayland native `fake_touch` per-package (best native KB) → adb macros only. Never `adb shell input` per-frame (200-1000ms spawn).

## Audit API (serve 06/07/08)
`keymap::{load,validate,audit,list}`: dup keys, slot clash, off-screen, missing switchKey, version!=1, screen mismatch. Reuse `ProfileAudit` from profile.rs.

## Steps
1. Port schema+validate, golden tests on 8 phantom profiles.
2. Port engine tick+states, headless replay test (no evdev hw).
3. Port android_socket client+server jar, loopback inject test.
4. QtScrcpy importer + 2 roundtrips. 5. uinput fallback behind flag.

## YAGNI
No CV smart-key. No gamepad-UHID v1 (stub). No per-frame adb. No `dispatchGesture` FPS.

## Blast radius
- Touches: `crates/wd-input/*`, `crates/wd-inject/*`, `java/*`, `profiles/keymap/*` only. No daemon/shell/spoof logic.
- Risk: MED (exclusive grab steals KB). Guard: F1 mouse-route/F8 capture/F9 pause/F10 overlay/F2 shutdown (config.rs + OPERATIONS.md), emergency release on socket drop, Edit mode never grabs (07).
- Rollback: kill engine → grab releases, zero system change.
- Accept: 8 goldens + 2 imports + loopback inject green, clippy 0/0.
