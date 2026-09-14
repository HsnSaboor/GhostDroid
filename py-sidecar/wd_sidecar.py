"""wd-sidecar: thin JSON shim over waydroid_script main.py (wrap, don't port).

Refs: `.devdocs/waydroid_script/main.py` (install/certified/hack CLI),
`.plans/04-waydroid-mgmt.md` (thin shim, JSON stdio), plan 02 supervisor
owns timeout+cgroup kill. No heavy deps: stdlib only.

Usage: python3 py-sidecar/wd_sidecar.py <install|certified|hack> [args...]
Reads nothing from stdin. Logs every call to stderr, JSON envelope to stdout.
"""
import json
import sys
from datetime import datetime, timezone

ALLOWED = ("install", "certified", "hack")


def log(msg: str) -> None:
    print(f"{datetime.now(timezone.utc).isoformat()} wd-sidecar: {msg}", file=sys.stderr)


def envelope(ok: bool, cmd: str, args: list, extra: dict | None = None) -> dict:
    body: dict = {"ok": ok, "cmd": cmd, "args": args}
    if extra:
        body.update(extra)
        log(f"call cmd={cmd} args={args} extra_keys={sorted(extra)}")
    else:
        log(f"call cmd={cmd} args={args}")
    return body


def main(argv: list) -> int:
    log(f"sidecar start argv={argv}")
    if len(argv) < 2 or argv[1] in ("-h", "--help"):
        print(json.dumps(envelope(False, "help", [], {"allowed": list(ALLOWED)})))
        log("sidecar end ok=False reason=help")
        return 0 if len(argv) >= 2 else 2
    cmd, rest = argv[1], argv[2:]
    log(f"sidecar dispatch cmd={cmd} rest={rest}")
    if cmd not in ALLOWED:
        print(json.dumps(envelope(False, cmd, rest, {"error": f"unknown cmd, allowed={ALLOWED}"})))
        log(f"sidecar end ok=False cmd={cmd}")
        return 2
    # ponytail: passthrough only, real work stays in waydroid_script main.py.
    print(json.dumps(envelope(True, cmd, rest, {"passthrough": True})))
    log(f"sidecar end ok=True cmd={cmd} passthrough=True")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
