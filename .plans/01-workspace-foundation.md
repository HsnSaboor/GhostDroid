# 01 Workspace foundation — shared core, lints, zero-warn gate

Refs: `.research/00-overview-decision.md:5-12`, `.research/00-overview-decision.md:25-32`, `.research/03-tech-stack.md:8-20`, `.research/01-slint-ui.md:7-10`, `.research/01-slint-ui.md:30`; `.devdocs/phantom/Cargo.toml`, `.devdocs/material-rust-template/Cargo.toml:16-23`, `.devdocs/material-rust-template/build.rs:5-12`.

## Goal
One Rust workspace. Thin bins, fat libs. Shared types once in `wd-core`. 0 errors 0 warnings from day one. No monolith, no repeats.

## Reuse (extend, never rewrite)
- Workspace shape from `.devdocs/phantom/Cargo.toml`: `[workspace] members=["phantom","phantom-gui"] resolver="2"`, `[workspace.dependencies] serde/serde_json/anyhow/thiserror/tracing/tracing-subscriber/toml/dirs`. Copy shape. Bump edition 2021→2024, add `tokio, zbus, evdev, wayland-client, smithay-client-toolkit, gilrs, clap, rmcp, slint, slint-build, eframe`.
- Slint build pattern from `.devdocs/material-rust-template/build.rs:5-12`: `CompilerConfiguration::new().with_library_paths(HashMap::from([("material", CARGO_MANIFEST_DIR/material-1.0/material.slint)]))` + `compile_with_config("ui/main.slint")`. Same for `ui/build.rs`. Consumer import `import {FilledButton,ElevatedCard} from "@material"` per `.devdocs/material-rust-template/ui/main.slint:1-2`.
- Version rule: template pins `slint 1.13` (`.devdocs/material-rust-template/Cargo.toml:16`) — stale. Use `slint 1.17 / slint-build 1.17` per `.research/01-slint-ui.md:30`. MSRV 1.92 per `.research/01-slint-ui.md:7`, `.devdocs/slint/Cargo.toml:84`.

## Layout (final)
```
Cargo.toml                    # workspace + lints + shared deps
crates/wd-core/               # types+config+error ONLY. ≤300 lines.
crates/wd-waydroid/           # lib: session/props/apps. No UI deps.
crates/wd-daemon/             # lib: supervisor+IPC; bin wd-daemon ≤150 lines.
crates/wd-input/              # lib: schema+engine+audit. Port phantom.
crates/wd-inject/             # lib: socket client; java/ server src.
crates/wd-spoof/              # lib: templates+policy. No keybox ship.
crates/wd-shell/              # lib: viewmodels; bin wd-shell thin.
crates/wd-overlay/            # bin only thin (egui glow).
crates/wd-mcp/                # lib: 24 tools; bins wd-ctl, wd-mcp thin.
py-sidecar/wd_sidecar.py      # JSON shim over waydroid_script.
ui/main.slint                 # MaterialWindow root.
ui/ui-shared.slint            # GameCard/StatusChip/PropRow/LogView/DeviceDot.
ui/m3e/m3e_{color,shape,motion,type}.slint
ui/m3e/{button_group,split_button,fab_menu,docked_toolbar,loading_indicator}.slint
profiles/keymap/*.json        # seed from phantom profiles.
profiles/spoof/*.toml         # per-game prop+hide refs.
modules/{sensor-noise,telephony-shim,network-shim}/
skills/{waydroid-control,waydroid-spoof}/SKILL.md
java/PhantomServer.java        # port from phantom contrib.
```

## wd-core API (only shared crate may be depended on by all)
```rust
// crates/wd-core/src/lib.rs
pub mod ids { pub struct Serial(pub String); pub struct Pkg(pub String); }
pub mod geom { pub struct RelPos { pub x: f32, pub y: f32 } } // 0.0-1.0, QtScrcpy rule .devdocs/QtScrcpy/docs/KeyMapDes.md
pub mod err { #[derive(thiserror::Error)] pub enum WdError { Io, Serde, Timeout, Frozen, Validation(String), ... } }
pub mod cfg { pub struct WdConfig { pub screen: (u32,u32), pub work_dir: PathBuf, pub socket: PathBuf } }
pub mod rpc { pub struct Req { pub method: String, pub params: serde_json::Value } pub struct Resp { pub ok: bool, pub data: serde_json::Value } }
```
- Config file `~/.config/wd/config.toml`. Trim `.devdocs/phantom/phantom/src/config.rs:413` (ScreenConfig, TouchBackendKind, RuntimeHotkeys, AndroidConfig, WaydroidConfig) to needed fields only — YAGNI.
- No logic, no Waydroid calls, no evdev here. Types + serde + validation helpers only.

## Workspace lints (deny warn = 0/0 gate)
```toml
[workspace.lints.rust]
unsafe_code = "forbid"  # EXCEPTION: wd-inject uinput only, `#[allow]` per-file + audit comment
missing_docs = "warn"
[workspace.lints.clippy]
all = "deny"; pedantic = "deny"; nursery = "deny"
```
- `RUSTFLAGS="-D warnings"`. CI: `cargo fmt --check`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`, `cargo doc --no-deps`.
- Rules: no `unwrap/expect` in libs, no `println!` (tracing only), bins ≤150 lines (parse args → call lib), lib crates ≤800 lines, `.slint` ≤400 lines.

## Steps
1. `cargo new --lib` all crates + bins. Wire workspace deps.
2. Write `wd-core` first, test serde roundtrip.
3. Add `ui/build.rs` library_paths pattern. `cargo check -p wd-shell` green.
4. Add clippy/rustfmt CI stub. Verify `cargo clippy` clean before 02.

## YAGNI cuts
- No TS, no C++ except Skia under Slint (`.research/00-overview-decision.md:12`). No new D-Bus daemon. No image builder. No SurrealismUI/sleek-ui pull (stale/tiny).

## Blast radius
- Touches: `Cargo.toml`, `crates/*/`, `ui/build.rs` only. No `.devdocs` edits, no Waydroid writes.
- Risk: LOW. Rollback: delete `crates/`, workspace still parses.
- Accept: `cargo check --workspace` 0/0, clippy clean, no logic yet.
