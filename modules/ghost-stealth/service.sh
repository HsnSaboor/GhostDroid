#!/system/bin/sh
# ghost-stealth service: file-layer masks (global, prop hooks are per-proc).
# - cpuinfo bind kills EmulatorDetection.cpp:59-73 (intel/amd scan).
# - DMI tmpfs kills ThinkPad leaks (subagent live probe: 20L8S14H00/LENOVO).
# - fake_mounts bind kills nvme0n1p2/btrfs/@/overlay + /var/lib/waydroid leaks.
# - battery nodes report 5Ah Li-ion discharging (no AC-wall/1000mAh leak).
# - cpufreq binds report Oryon V3 clocks (2x4.74GHz + 6x3.62GHz).
# Idempotent: re-run safe (remount over existing bind).
MODDIR=${0%/*}
FAKE="$MODDIR/fake_cpuinfo"
cp "$MODDIR"/assets/fake_cpuinfo "$FAKE" 2>/dev/null || true
mount -o bind "$FAKE" /proc/cpuinfo 2>/dev/null || true
# Kernel mask: DeviceInfoHW System tab reads /proc/version directly
# (uname() hook does not cover file reads). GKI 5.15, no host strings.
cp "$MODDIR"/assets/fake_version "$MODDIR/fake_version" 2>/dev/null || true
mount -o bind "$MODDIR/fake_version" /proc/version 2>/dev/null || true
# DMI mask: hide host board/serial, keep dir present (ENOENT-safe).
if [ -d /sys/class/dmi/id ]; then
    mount -t tmpfs -o size=4k,mode=755 tmpfs /sys/class/dmi/id 2>/dev/null || true
fi
# Mounts mask: DeviceInfoHW Mounts tab leaks /dev/nvme0n1p2, btrfs subvol
# /@/@home, and overlay lowerdirs under /var/lib/waydroid. The canonical
# mask is the native open/openat/fopen redirect (file_hooks.cpp) because
# /proc/mounts is a per-process symlink (binds don't propagate across
# mount namespaces). service.sh keeps a world-readable snapshot at
# /data/local/tmp/gs_fake_mounts for the redirect to serve.
cp "$MODDIR"/assets/fake_mounts /data/local/tmp/gs_fake_mounts 2>/dev/null || true
chmod 644 /data/local/tmp/gs_fake_mounts 2>/dev/null || true
# Extended redirect table (file_hooks.cpp): drivers/input/audio/usb fakes.
# World-readable snapshots; per-process open/openat/fopen redirect serves
# them because binds don't propagate across mount namespaces.
# Snapshot names MUST match file_hooks.cpp #defines:
# gs_fake_modules, gs_fake_input_devices, gs_fake_asound_cards,
# gs_fake_usb_devices.
for f in fake_modules fake_input_devices fake_asound_cards fake_usb_devices; do
    cp "$MODDIR"/assets/$f /data/local/tmp/gs_${f} 2>/dev/null || true
    chmod 644 /data/local/tmp/gs_${f} 2>/dev/null || true
done
cp "$MODDIR"/assets/fake_mounts "$MODDIR/fake_mounts" 2>/dev/null || true
mount -o bind "$MODDIR/fake_mounts" /proc/mounts 2>/dev/null || true
mount -o bind "$MODDIR/fake_mounts" /proc/self/mountinfo 2>/dev/null || true
# Block mask: hide host nvme0n1 from /sys/block (Storage tab). Guarded:
# only when the dir exists; dummy sda/mmcblk0 keep enumeration sane.
if [ -d /sys/block ]; then
    mount -t tmpfs -o size=4k,mode=755 tmpfs /sys/block 2>/dev/null || true
    mkdir -p /sys/block/sda /sys/block/mmcblk0 2>/dev/null || true
    echo 274877906944 > /sys/block/sda/size 2>/dev/null || true
    echo 0 > /sys/block/mmcblk0/size 2>/dev/null || true
fi
# Battery mask: DeviceInfoHW Battery tab leaks AC-wall + 1000mAh host UPS.
# Bind fake values over real sysfs nodes when present (guarded per-node).
BAT=/sys/class/power_supply/battery
if [ -d "$BAT" ]; then
    echo 85 > "$MODDIR/fake_capacity" 2>/dev/null || true
    echo "Discharging" > "$MODDIR/fake_status" 2>/dev/null || true
    echo 5000000 > "$MODDIR/fake_charge_full" 2>/dev/null || true
    echo 4250000 > "$MODDIR/fake_charge_counter" 2>/dev/null || true
    echo "Li-ion" > "$MODDIR/fake_tech" 2>/dev/null || true
    echo 85 > "$BAT/capacity" 2>/dev/null || mount -o bind "$MODDIR/fake_capacity" "$BAT/capacity" 2>/dev/null || true
    echo "Discharging" > "$BAT/status" 2>/dev/null || mount -o bind "$MODDIR/fake_status" "$BAT/status" 2>/dev/null || true
    echo "Li-ion" > "$BAT/technology" 2>/dev/null || mount -o bind "$MODDIR/fake_tech" "$BAT/technology" 2>/dev/null || true
    echo 5000000 > "$BAT/charge_full_design" 2>/dev/null || mount -o bind "$MODDIR/fake_charge_full" "$BAT/charge_full_design" 2>/dev/null || true
    echo 4250000 > "$BAT/charge_counter" 2>/dev/null || mount -o bind "$MODDIR/fake_charge_counter" "$BAT/charge_counter" 2>/dev/null || true
fi
# CPUfreq mask: DeviceInfoHW SoC tab reads cpu*/cpufreq/*_freq (kHz).
# Oryon V3 (SM8850): cores 0-1 prime @4.74GHz, cores 2-7 perf @3.62GHz.
for cpu in 0 1 2 3 4 5 6 7; do
    D=/sys/devices/system/cpu/cpu$cpu/cpufreq
    if [ "$cpu" -lt 2 ]; then F=4740000; else F=3620000; fi
    [ -d "$D" ] || continue
    echo $F > "$MODDIR/fake_freq_$cpu" 2>/dev/null || true
    for node in cpuinfo_max_freq cpuinfo_min_freq scaling_cur_freq scaling_max_freq scaling_min_freq; do
        [ -f "$D/$node" ] || continue
        echo $F > "$D/$node" 2>/dev/null || mount -o bind "$MODDIR/fake_freq_$cpu" "$D/$node" 2>/dev/null || true
    done
done
