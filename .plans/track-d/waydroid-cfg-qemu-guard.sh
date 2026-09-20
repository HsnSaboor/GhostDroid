#!/usr/bin/env bash
# waydroid.cfg [properties] vs base.prop qemu.* re-injection guard.
#
# PROPOSAL ONLY — read-only, never writes. Safe to run as-is.
# Refs: `.plans/04-waydroid-mgmt.md:22` (cfg [properties] alt path),
# Track D task 3. Live finding 2026-09-20: `waydroid.cfg` ships
# `qemu.hw.mainkeys = 1` while `waydroid_base.prop` is qemu-clean.
#
# Usage: ./waydroid-cfg-qemu-guard.sh [--cfg PATH] [--base PATH]
# Exit 0 = clean, 1 = qemu.* re-injection (or global ro.hardware) detected.
set -euo pipefail

CFG=/var/lib/waydroid/waydroid.cfg
BASE=/var/lib/waydroid/waydroid_base.prop

while [ $# -gt 0 ]; do
    case "$1" in
        --cfg) CFG="$2"; shift 2 ;;
        --base) BASE="$2"; shift 2 ;;
        *) echo "usage: $0 [--cfg PATH] [--base PATH]" >&2; exit 2 ;;
    esac
done

fail=0

# Section-scoped key dump: [properties] block only, `key = value` or `key=value`.
props_section() {
    awk '/^\[properties\]/{on=1; next} /^\[/{on=0} on && /=/ {print}' "$1" \
        | sed 's/[[:space:]]*=[[:space:]]*/=/' | sort -u
}

echo "== qemu.* in $CFG [properties] =="
if props_section "$CFG" | grep -E '^[^=]*qemu[^=]*=' ; then
    echo "FAIL: qemu.* key re-injected via waydroid.cfg [properties]" >&2
    fail=1
else
    echo "clean"
fi

echo "== qemu.* in $BASE =="
if [ -f "$BASE" ] && grep -E '^[[:space:]]*[^#[:space:]][^=]*qemu[^=]*=' "$BASE"; then
    echo "FAIL: qemu.* key present in base.prop" >&2
    fail=1
else
    echo "clean"
fi

echo "== global ro.hardware in $BASE (per-process only) =="
if [ -f "$BASE" ] && grep -E '^[[:space:]]*ro\.hardware=' "$BASE"; then
    echo "FAIL: global ro.hardware write (HAL bootloop vector)" >&2
    fail=1
else
    echo "clean"
fi

echo "== host bridge lines intact in $BASE (houdini/abilist) =="
for needle in libhoudini x86_64; do
    if [ -f "$BASE" ] && grep -qi "$needle" "$BASE"; then
        echo "ok: $needle present"
    else
        echo "WARN: $needle missing (merge_lines should preserve)" >&2
    fi
done

if [ "$fail" -ne 0 ]; then
    echo "RESULT: RE-INJECTION DETECTED" >&2
    exit 1
fi
echo "RESULT: clean"
