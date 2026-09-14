# 09 Quality + packaging — S-tier gate, order, global blast radius

Refs: 00-08 plans; `.devdocs/phantom/docs/{TESTING.md,EDGE_CASES.md,TROUBLESHOOT.md}`, `docs/{ARCHITECTURE,PROTOCOL,PROFILES,IPC,OPERATIONS}.md`, `packaging/`; `.devdocs/waydroid-mcp/tests/`; `.research/03-tech-stack.md:170-181`.

## Gates (0 errors 0 warnings, must pass per plan)
```
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings   # all+pedantic+nursery deny
cargo test --workspace
cargo doc --no-deps
slint-build warnings denied (ui/build.rs)
```
- Rules: no `unwrap/expect` libs, no `println!`, lib ≤800 lines, bin ≤150, .slint ≤400, unsafe only `wd-inject` uinput with audit comment.
- Known edge doc: `phantom/docs/EDGE_CASES.md + TROUBLESHOOT.md` (uevent timing, nested Wayland #2073, touchscreen-as-mouse #1049 X11 → test native Wayland KDE/Niri).

## Tests (goldens from devdocs)
- `wd-input`: 8 phantom profiles (`profiles/*.json` incl `pubg.json:4080B`) validate+audit green; QtScrcpy `gameforpeace.json:328 + ScrcpyKeyMapper pubg.json:587` import roundtrip; engine headless replay (tick 4ms constants); inject loopback frames `0x00/0x01/0x02/0x7F` (ANDROID_SOCKET_PROTOCOL.md:139).
- `wd-spoof`: `device_profile.conf:260` render snapshot; swap diff test (no secrets in repo).
- `wd-waydroid`: frozen parser (`waydroid status` FROZEN → kind frozen, `adb.py:39-51`), ready-path wait, rm-first dump retry 3×1.0 (`core.py:43-95`).
- `wd-mcp`: 24 tool JSON schemas (zod-equivalent), screenshot 3-surface (media.py pattern), logcat parse VDIWEF (logs.py).
- `ui`: gallery + per-page screenshots; `styles.test.ts:OVERLAY_TOKENS` guard port.
- `modules/sensor-noise`: OU variance 0.01-0.05 m/s² else fail.

## Packaging (reuse `phantom/packaging/` shape)
`cargo-deb | rpmbuild | PKGBUILD | flatpak freedesktop rust ext | static musl 300KB-2MB` (`.research/03-tech-stack.md:170`). `slint-build @material library_paths` (template build.rs). py-sidecar vendored pinned (`requirements tqdm/requests/InquirerPy`). Java server prebuilt `/usr/lib/wd/wd-server.jar` (PhantomServer.java:344 / wl 406).

## Order (dependency-safe, one plan per batch)
01 workspace → 02 daemon → 03 input/inject → 04 mgmt/sidecar → 05 root/spoof → 06 shell → 07 overlay → 08 cli/mcp/skills → 09 gate. No cross-plan edits. Small diffs, test each.

## Global blast radius
| Plan | Touches | Risk | Rollback |
|---|---|---|---|
| 01 workspace | Cargo, ui/build.rs | LOW | delete crates |
| 02 daemon | wd-waydroid, wd-daemon | MED | pkill daemon, LXC persists |
| 03 input | wd-input, wd-inject, java/, profiles/keymap | MED | kill engine, grab releases |
| 04 mgmt | py-sidecar, wd-spoof/templates, profiles/spoof | HIGH | restore prop snapshot + restart |
| 05 root/spoof | modules/*, wd-spoof, overlay magisk* | HIGH | toggle disable, uninstall, restore |
| 06 shell | ui/*, wd-shell | LOW-MED | revert ui, headless CLI works |
| 07 overlay | wd-overlay | LOW | pkill overlay |
| 08 cli/mcp | wd-mcp, skills/ | MED | disable http, stdio local |
| 09 gate | CI, packaging | LOW | block release only |

DONE when: workspace clippy+test green, daemon boot/frozen e2e, 8+2 keymap goldens, prop swap+restore e2e, PIF policy dry-run, shell gallery shots, overlay edit/map toggle, 24 tools schema green, packages build.
