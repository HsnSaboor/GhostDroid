#!/system/bin/sh
# ghost-stealth service: file-layer masks (global, prop hooks are per-proc).
# - cpuinfo bind kills EmulatorDetection.cpp:59-73 (intel/amd scan).
# - DMI tmpfs kills ThinkPad leaks (subagent live probe: 20L8S14H00/LENOVO).
# Idempotent: re-run safe (remount over existing bind).
MODDIR=${0%/*}
FAKE="$MODDIR/fake_cpuinfo"
cp "$MODDIR"/assets/fake_cpuinfo "$FAKE" 2>/dev/null || true
mount -o bind "$FAKE" /proc/cpuinfo 2>/dev/null || true
# DMI mask: hide host board/serial, keep dir present (ENOENT-safe).
if [ -d /sys/class/dmi/id ]; then
    mount -t tmpfs -o size=4k,mode=755 tmpfs /sys/class/dmi/id 2>/dev/null || true
fi
