# Loop 2 — GhostDroid prod gaps (after v1.0.0)

## Verified facts (2026-09-15, live laptop)

- Repo at 985bbda, tags v0.1.0 + v1.0.0 exist. `/opt/ghostdroid` installed (v1.0.0).
- Waydroid live: Session+Container RUNNING, MAINLINE, IP 192.168.240.112,
  wayland-1, 19 apps (`com.supercell.clashroyale`, `io.github.huskydg.magisk`…).
- Stock `waydroid_base.prop` has NO fingerprint row; live ARM bridge
  `libhoudini.so` + abilist `x86_64,x86,arm64-v8a,…` MUST be preserved on swap.
- `waydroid shell` needs root (plan: pkexec path, never silent sudo).
- `s26-ultra.toml` already default in `state.rs` (stack full, m3q/m3qxeea,
  arm64-only abilist, release-keys/user). `render.rs` GREEN tail ok.
- `wd-input` schema covers all 9 phantom node types; pubg.json = 21 nodes.
  Keys page still shows "0 nodes · Map tab" placeholder.
- `wd-inject` frame builders + tags (0x00/0x01/0x02/0x03/0x7f) + port 27183 done.
- Vendor server: `.devdocs/phantom/contrib/android-server/.../PhantomServer.java`
  (344 lines, TCP LE frames → MotionEvent → `injectInputEvent` ASYNC).
  `java/PhantomServer.java` is a 1-line stub — THE gap the user flagged.
- Stubs remaining: `wd-daemon` bin (no listen), `dispatch.rs`
  (`device.list`/`app.list` stub shapes), `wd-overlay::run()` (logs only),
  `call.rs` app/input/vision/ui/shell/logcat arms (stub JSON), Keys canvas,
  Devices live IP not shown, spoof-load validates only (no apply).
- Release CI green with rust-cache; gpui build dominates time.
- Rules: CLOUD BUILDS ONLY (`gh` + Actions). NEVER `cargo build/check/clippy/test`
  on `wd-gpui` or `--workspace` locally (slow laptop). Allowed locally:
  `cargo fmt`, `cargo test -p <pure-crate>` for non-gpui crates only
  (wd-core/waydroid/input/inject/spoof/shell/overlay/mcp/daemon).

## Loop 2 scope (4 disjoint workstreams, no file overlap)

**A — PhantomServer.java full port + push/run wiring.**
Port vendor 344-line server into `java/PhantomServer.java` (GhostDroid header,
same LE protocol 0x00/0x01/0x02/0x03/0x7f, port 27183, TouchInjector 10 slots,
ASYNC inject). Add `wd-waydroid` push+run helpers (argv builders:
`adb push <jar>`, `app_process`/`dalvikvm` launch line — builders + parsers
only, no spawn in wd-waydroid; spawn lives in sync/exec callers). Ship the
`.idc` (`Vendor_1234_Product_5678.idc`) reference into `java/` README or keep
as-is with doc. Touch ONLY: `java/*`, `crates/wd-waydroid/src/android_server.rs`
(new) + `lib.rs` export line.

**B — Spoof apply (swap+restart+verify, houdini-safe) + root enable.**
New `wd-spoof::apply` (snapshot → render → merge preserving
`libhoudini.so`/`native.bridge`/x86 abilist host lines → write base.prop →
`waydroid session stop/start --wait` via caller → verify `getprop`
fingerprint). Houdini rule: rendered arm64 props go in, host bridge lines
never removed (merge, not overwrite). Root enable: Magisk Delta already
installed (live); expose `stack=full` layer checklist via `policy.rs`
(minimal addition, no new spawn in wd-spoof — return plan steps, caller runs).
Wire `spoof-load` in `wd-mcp::call.rs` to return rendered line count +
houdini-kept flag (validation today → validate+render preview, apply behind
explicit `spoof-apply` local verb in `wd-ctl`). Touch ONLY:
`crates/wd-spoof/src/apply.rs` (new) + `lib.rs`, `crates/wd-mcp/src/call.rs`
(spoof arm only), `crates/wd-mcp/src/bin/wd-ctl.rs` (new `spoof-apply` verb).

**C — Live actions end-to-end (daemon listen + app/input/ui/logcat live).**
`wd-daemon`: bind `socket_path()` NDJSON loop calling existing `dispatch`
(no new protocol). `call.rs`: make `app.start/stop/list`, `input.tap/swipe/key`,
`ui.dump/find`, `logcat.dump`, `vision.screenshot` live via
`wd_waydroid::exec::run_waydroid` / adb argv (same boot_live/status_live
pattern, stub fallback when binary missing). NO new tools (24 stays).
Touch ONLY: `crates/wd-daemon/src/bin/wd-daemon.rs` + `serve.rs` (new),
`crates/wd-mcp/src/call.rs` (non-spoof arms only — coordinate with B by
function boundary: B owns `spoof_load` fn, C owns the rest).

**D — Keys canvas real data + Devices live IP + release speed.**
Keys page: show real node count from `profiles/keymap/pubg.json` via
`wd-shell::KeymapState` extension (add `node_count` field, seed from pubg 21;
canvas still overlay-owned, page shows counts per tab + fire key).
Devices page: display live IP (`fetch_device` already returns it — thread
through `AppState`: add `ip: String`, set in `app.rs` sync update instead of
dropping `_ip`). Release: split `build` job (bins matrix or
`cargo build --release -p wd-mcp -p wd-daemon` fast job first is NOT needed —
instead: keep single build but add `CARGO_INCREMENTAL=0`,
`CARGO_PROFILE_RELEASE_DEBUG=0`, keep rust-cache, smoke only `wd-ctl` exit
code to save minutes). Touch ONLY: `crates/wd-shell/src/lib.rs`
(`KeymapState::node_count`, doc), `crates/wd-gpui/src/state.rs` (seed ip +
node count), `crates/wd-gpui/src/app.rs` (use ip), `crates/wd-gpui/src/pages/keys.rs`
+ `devices.rs` (display only), `.github/workflows/release.yml` (env/flags only).

## Subagent quality block (paste verbatim into every implementer prompt)

> it shoudl be able to strat waydroid it self etc . all the ui shodul eb
> correct and dark mode only no contrast issues . use hallamrk and
> impecccable skills write full S tier production app YAAGNI GPUI KIT 100%
> coorct usage prebuilt ui components no need to write from scratch no bugs
> S tier code quality ALL FEATURES WORKING TO TE FULLEST NOT A MVP OR DEMO
> okay . CLOUD BUILDS ONLY — never run cargo build/check/clippy/test on
> wd-gpui or --workspace locally; allowed: cargo fmt + cargo test -p on
> pure non-gpui crates only. Read .plans/ .devdocs/ .research/ before coding.
