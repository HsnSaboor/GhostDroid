# Loop 1 — GhostDroid 1.0.0 (jump from 0.1.0)

## Verified facts (2026-09-15, live laptop)

- Waydroid: Session RUNNING, Container RUNNING, MAINLINE, IP 192.168.240.112,
  `wayland-1`, user saboor(1000). No sudo needed for status/props/app.
- Apps: 19 (`waydroid app list`), incl Clash Royale
  (`com.supercell.clashroyale`), Magisk Delta (`io.github.huskydg.magisk`).
- Live print: `samsung/SM-G9860/waydroid_x86_64:13/...:userdebug/test-keys`
  (S20+ 5G name, waydroid device — must be replaced by spoof).
- ARM bridge live: `ro.dalvik.vm.native.bridge=libhoudini.so`,
  abilist `x86_64,x86,arm64-v8a,armeabi-v7a,armeabi`.
- `persist.waydroid.multi_windows=true`. `ro.debuggable=0`, `ro.secure=1`,
  `ro.build.tags=test-keys`, `ro.build.type=userdebug`.
- Release CI: `.github/workflows/release.yml` green (tag push → build →
  smoke → tarball → gh-release). RUSTFLAGS x86-64-v3. rust-cache on.

## S26 Ultra — real specs (NOT S25, web-verified Sep 2026)

- Retail: SM-S948B (global), SM-S948U (US), SM-S948Q (JP). Released 2026-03-06.
- Android device codename: **m3q**. Products: **m3qxeea** (EU/global),
  **m3qjpnw** (JP). Internal series codename Miracle M3 (not used in props).
- Real fingerprints (userdebug/test-keys NEVER ships; use user/release-keys):
  - EU: `samsung/m3qxeea/m3q:16/BP4A.251205.006/S948BXXS4AZG5_OXM4AZG5:user/release-keys`
  - JP: `samsung/m3qjpnw/m3q:16/BP4A.251205.006/S948QOPS1AZF2_SJP1AZF2:user/release-keys`
- SoC `sm8850` Snapdragon 8 Elite Gen 5 (2x4.74GHz Oryon V3 + 6x3.62GHz),
  GPU Adreno 840, Android 16, One UI 8.5, 1440x3120, 12GB RAM.

## Loop 1 scope (3 disjoint workstreams)

**A — S26 Ultra spoof (default root spoof profile).**
New `profiles/spoof/s26-ultra.toml` (stack=full, m3q fingerprint, m3qxeea
product, sm8850/Adreno840 hw, release-keys/user, verified-boot green list).
Keep `gaming-full.toml`/`gaming-basic.toml` untouched (tests pin them).
Seeded `AppState` spoof default becomes `s26-ultra` with S26 props.
Touch: `profiles/spoof/s26-ultra.toml` (new), `crates/wd-spoof/src/*`
(minimal: default-id const or doc), `crates/wd-gpui/src/state.rs` (seed only).

**B — Self-boot + auto-load apps (no MVP stubs).**
`ShellView` spawns a background thread on boot: `waydroid session start`
(if not RUNNING) → poll `waydroid status` → `waydroid app list` →
`set_games` with real Name/packageName rows. Devices page shows live
serial/state/IP. Pure addition: new `crates/wd-gpui/src/sync.rs`
(argv-spawn via `wd_waydroid::exec`, parse via `parse_status` /
`parse_app_list`), `app.rs` wiring only. Never block UI thread.
Rules: argv-only spawn, timeout-kill, missing-binary → seeded fallback.

**C — Dark-only UI correctness (all 5 pages).**
Every page renders real state, zero light-mode paths (ThemeMode::Dark forced
in `run()` already), no contrast issues (no gray-on-black < 4.5:1, no
unstyled div text on dark). Prebuilt kit only: Sidebar/Button/Tag/
DescriptionList/Progress/Input-or-Button-row/List — no custom paint.
Touch: `crates/wd-gpui/src/pages/*.rs`, `shared.rs` only.

## Subagent quality block (paste verbatim into every implementer prompt)

> it shoudl be able to strat waydroid it self etc . all the ui shodul eb
> correct and dark mode only no contrast issues . use hallamrk and
> impecccable skills write full S tier production app YAAGNI GPUI KIT 100%
> coorct usage prebuilt ui components no need to write from scratch no bugs
> S tier code quality ALL FEATURES WORKING TO TE FULLEST NOT A MVP OR DEMO
> okay

UI agent additionally: load `hallmark` + `impeccable` skills first via the
skill tool and follow them.

## Release (cloud only, laptop never builds)

1. `git tag v1.0.0 && git push origin v1.0.0` → GH Actions builds tarball.
2. Download `ghostdroid-1.0.0-linux-x86_64.tar.gz` + sha256, verify, install.
3. Verify: `wd-ctl status`, `wd-ctl spoof-load profiles/spoof/s26-ultra.toml`,
   skill loop (screenshot → ui-find → key → status), MCP discover.
4. `compress` this loop's context, open Loop 2.

## Done = release installed on this laptop, fully working, GhostDroid stack
## (root + full spoof as S26 Ultra) usable, 0 clippy warnings, fmt clean.
