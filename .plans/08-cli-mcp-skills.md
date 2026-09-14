# 08 CLI + MCP + skills — one tool core, 24 tools

Refs: `.research/03-tech-stack.md:154-168`; `.devdocs/scrcpy-mcp/{src/index.ts:StdioServerTransport,src/tools/{session:3,input:7,files:3,device:10,vision:3,clipboard:2,video:2,ui:2,apps:6,shell:1}=39,src/utils/*,AGENTS.md:registerTool+zod}`; `.devdocs/waydroid-mcp/src/waydroid_mcp/{server.py:6 tools,screen.py:strip_png/raw_to_jpeg,adb.py:39-51 frozen,core.py:43-95 --no-tree + rm-first,cli.py:dual-use + exits 3/4,session.py,bootstrap.py}`; `.devdocs/adb-mcp/adb_mcp/{core.py:quote_argv+shell_rc,server.py:FastMCP,tools/{apps:16,device:15,files:9,input:12,logs:8,media:5,network:11,system:11,ui:3}=90}`; `.devdocs/android-mcp-server-us/src/{index.ts:http opt-in,tool-registry.ts,config.ts:categories,adb.ts,tools/{ui.ts:sharp+takeScreenshot,takeAnnotated,logcat.ts:getLogcat+Crash,utils.ts:shellEscape+wrapToolHandler}}=76`; `.devdocs/android-mcp-server-mg/src/index.ts:25 snake`; `.devdocs/android-mcp-server/server.py:5 ppadb`; `.devdocs/Android-MCP/src/android_mcp/{__main__.py:device flags,tree/service.py:annotated_screenshot,mobile/service.py}:14 uiautomator2`; `.devdocs/scrcpy-mcp-ch/*` (cross-check).

## Goal
CLI + MCP same `wd-mcp` lib fns. No replay drift. Agent loop observe→locate→act→re-observe. 24 tools (NOT 90 — YAGNI cut adb-mcp bloat).

## Tool core (`wd-mcp` lib — every CLI cmd = one fn)
```
device.list/boot{waitMs}/status | app.install/uninstall/start{forceStop}/stop/list | activity.current
input.tap/swipe/key/text | vision.screenshot{scale}/stream_start{maxSize,fps}/stream_stop
ui.dump{compact}/find{text,rid,class,desc} | shell.exec{gated allowlist}/file.push/pull
logcat.dump/start/stop | prop.get/set | keymap.load | spoof.load
```
- Schemas zod-style per `scrcpy-mcp AGENTS.md` (registerTool + zod + annotations + content/structuredContent/isError) + `us/tools/ui.ts` (`takeScreenshot{format,quality,maxWidth,serial}`, `tap{x,y,serial}`, `getLogcat{lines,tag,priority,serial}`). Copy `us/tools/utils.ts:shellEscape/validators/wrapToolHandler` + `adb-mcp/core.py:quote_argv/shlex.quote + shell_rc(; echo __rc=$?)` for injection-safe shell. Serial resolve COPY `scrcpy-mcp/src/utils/adb.ts:resolveSerial/parseDeviceList` (CRLF, multi-device error, ANDROID_SERIAL).
- Device select COPY `Android-MCP/__main__.py:14-37` flags (`--device/--connection/--wifi/--usb` + env) + `us/config.ts` category filter (`ANDROID_TOOLS/ANDROID_DISABLE`) + 2-tier gate (`ANDROID_MCP_ALLOW_WRITE/SHELL`, `ADB_MCP_ALLOW_SHELL=1`).

## Screenshot pipeline (copy order)
1. `adb-mcp/tools/media.py:_screencap`: `exec-out screencap -p` binary + CRLF→LF repair (first 64B). 3 surfaces: inline Image, to_file, base64.
2. `scrcpy-mcp/src/tools/vision.ts`: scrcpy frameBuffer→jpeg/base64 first, fallback adb raw → png. `structuredContent{source:scrcpy|adb}`.
3. `us/tools/ui.ts:takeScreenshot`: sharp png fast-path skip transform else jpeg quality/maxWidth.
4. `waydroid-mcp/screen.py:strip_png` (cut warn text before PNG_MAGIC) + `raw_to_jpeg(data,w,h,scale=2,q70)` trailing-junk tolerant.
Return MCP `image` base64 + resource `android://device/frame/latest.jpg` @2FPS. v1 snapshot ~500ms; scrcpy 33ms later. Avoid `android-mcp-server/adbdevicemanager.py` fixed `compressed_screenshot.png` (multi-device race).

## Logcat + ui patterns
- Logcat COPY `adb-mcp/tools/logs.py` (best): `logcat_start` Popen `adb -s logcat -v threadtime` → temp file, `dump{tags,priority,message_contains,tail}` → `{time,pid,tid,level,tag,message}` VDIWEF order, `get_crash_log(-b crash)`, `get_anr_traces`, `capture_bugreport`. One-shot fallback `us/tools/logcat.ts` (`-d -t N`, `tag:V *:S`), `mg get_logs{package,level,lines=200,since}`. Redact tokens opt-in. SSE progress scoped per request (us http pattern).
- ui.dump COPY guards: waydroid-mcp `core.py:43-95` (`Core(tree)` + CLI `--no-tree`/env `WM_NO_TREE=1` — dump rebinds a11y ~0.8s, canvas empty anyway) + rm-first + retry 3×1.0s; tmp shape `scrcpy-mcp ui.ts:52-60` unique `/sdcard/.ui_dump_${Date.now()}_${rand}.xml` + strip dumped-to line. `adb-mcp ui.py:35-44 /dev/tty` fast-path optional. NEVER fixed no-rm path (us stale-prone). Find: `adb-mcp ui.py + us ui.ts + Android-MCP tree/service.py:annotated_screenshot` (uiautomator2 selector text/rid/class/desc).

## Transports
- stdio default ALL repos (`scrcpy-mcp index.ts StdioServerTransport`, `waydroid-mcp server.py:133 stdio`, `adb-mcp server.py`, `mg index.ts`). Newline JSON-RPC, logs stderr only.
- http opt-in ONLY `us/src/index.ts` (`startMcpServer`, `MCP_TRANSPORT=http` + Bearer + `/health`). Copy for upgrade: single POST `/mcp`, headers MCP-Protocol-Version, per-request SSE. Env `ADB_PATH/ANDROID_SERIAL/FFMPEG_PATH/SCRCPY_SERVER_PATH` (scrcpy-mcp) respected.

## CLI (`wd-ctl` thin bin, dual-use like `waydroid-mcp/cli.py`)
`devices|boot --wait --frozen-check|shutdown|freeze|unfreeze|status|install|uninstall|list-apps|launch --stop-first|stop|current|tap|swipe|long-press|drag|pinch|key|text|screenshot -o --scale|record start|stop|logcat --filter -f --redact|shell --|prop get|set|dumpsys|keymap load|audit|list|spoof load|list|backup|restore|ui dump --compact --no-tree|find|mcp --transport`. `--json` always. Exits `3=mismatch 4=device` (waydroid-mcp). Shares `Core`-equivalent `wd-mcp` lib with MCP — copy dual-use.

## Skills (`skills/`, Agent Skills spec, <5000 tokens)
- `waydroid-control/SKILL.md` frontmatter (name/desc/license/compat/requires wd-ctl+adb+Wayland): runbook observe→locate→act→re-observe, prefer `ui.find` over coords, screenshot after act, check frozen, `--no-tree` canvas, record macros not raw replay.
- `waydroid-spoof/SKILL.md`: profile refs, BASIC+DEVICE only, never STRONG, no banking PKGs.
- Paths: `.claude/skills/`, `.opencode/skills/` compat. MCP truth, CLI fallback.

## Blast radius
- Touches: `crates/wd-mcp/*`, `skills/*` ONLY. Calls daemon/input/spoof libs, never duplicates.
- Risk: MED (shell.exec). Guard: allowlist default + `ALLOW_SHELL=0`, redact, no raw default (adb-mcp `run_shell` gated).
- Rollback: kill http, stdio local-only remains.
