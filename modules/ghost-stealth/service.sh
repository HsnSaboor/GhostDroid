#!/system/bin/sh
# ghost-stealth service: file-layer masks (global, prop hooks are per-proc).
# - cpuinfo bind kills EmulatorDetection.cpp:59-73 (intel/amd scan).
# - DMI tmpfs kills ThinkPad leaks (subagent live probe: 20L8S14H00/LENOVO).
# - fake_mounts bind kills nvme0n1p2/btrfs/@/overlay + /var/lib/waydroid leaks.
# - battery nodes report 5Ah Li-ion discharging (no AC-wall/1000mAh leak).
# - cpufreq binds report Oryon V3 clocks (2x4.74GHz + 6x3.62GHz).
# Idempotent: re-run safe (remount over existing bind).
MODDIR=${0%/*}
# ACE cache purge: PUBG anti-cheat fingerprints persist in files/ano_tmp
# (comm.dat emu_tp, ace_cache_db.dat, fake_uiui.dat) + top-level
# ace_shell_di.dat across reboots. Purge at boot so each launch looks
# fresh. Guarded + idempotent; NEVER touches UE4Game (pak downloads).
PUBG_FILES=/data/data/com.tencent.ig/files
if [ -d "$PUBG_FILES" ]; then
  rm -rf "$PUBG_FILES"/ano_tmp/* 2>/dev/null || true
  rm -f "$PUBG_FILES"/ace_cache_db.dat "$PUBG_FILES"/ace_shell_di.dat "$PUBG_FILES"/SpeedUpCCH.dat "$PUBG_FILES"/fake_uiui.dat 2>/dev/null || true
fi
FAKE="$MODDIR/fake_cpuinfo"
cp "$MODDIR"/assets/fake_cpuinfo "$FAKE" 2>/dev/null || true
mount -o bind "$FAKE" /proc/cpuinfo 2>/dev/null || true
# Kernel mask: DeviceInfoHW System tab reads /proc/version directly
# (uname() hook does not cover file reads). GKI 5.15, no host strings.
cp "$MODDIR"/assets/fake_version "$MODDIR/fake_version" 2>/dev/null || true
mount -o bind "$MODDIR/fake_version" /proc/version 2>/dev/null || true
# DMI mask: hide host board/serial, keep dir present (ENOENT-safe).
# Populate Samsung S26 Ultra identity after tmpfs mount.
if [ -d /sys/class/dmi/id ]; then
    mount -t tmpfs -o size=4k,mode=755 tmpfs /sys/class/dmi/id 2>/dev/null || true
    echo "SM-S948B" > /sys/class/dmi/id/product_name 2>/dev/null || true
    echo "SAMSUNG" > /sys/class/dmi/id/sys_vendor 2>/dev/null || true
    echo "m3q" > /sys/class/dmi/id/board_name 2>/dev/null || true
    echo "S948BXXU1AXE4" > /sys/class/dmi/id/bios_version 2>/dev/null || true
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
# gs_fake_usb_devices, gs_fake_usb_drivers_list, gs_fake_pci_drivers_list,
# gs_fake_platform_drivers_list, gs_fake_sys_module_list,
# gs_fake_usb_{manufacturer,product,serial,idvendor,idproduct,version,
# busnum,devnum}, gs_fake_pci_{vendor,device}, gs_fake_pci_{devices,uevent},
# gs_fake_pci_dev_{uevent,modalias,revision}, gs_fake_kgsl_gpubusy,
# gs_fake_bat_{charge_full,charge_full_design,model,manufacturer},
# gs_fake_cpuinfo, gs_fake_build_prop (S26 identity/build subset;
# overlapping keys match spoof.conf exactly),
# gs_fake_cpufreq_{prime,perf,min}, gs_fake_cpu_online,
# gs_fake_{kallsyms,iomem,ioports,proc_misc},
# gs_fake_{meminfo,thermal_zone0,proc_stat,cgroup} (pid 2594 perf-loop
# RAM/thermal/stat tells: host cpuN lines + swap totals; libcubehawk
# cgroup reads: host "/.lxc" leak),
# gs_fake_version (System tab /proc/version file read), gs_fake_proc_net_tcp
# (ACE net-scan entry point: /proc/net/tcp+6), gs_fake_sys_kernel_{ostype,
# osrelease,version,hostname} (/proc/sys/kernel/* uname-class file reads).
for f in fake_modules fake_input_devices fake_asound_cards fake_usb_devices \
         fake_usb_drivers_list fake_pci_drivers_list fake_pci_devices \
         fake_pci_uevent \
         fake_platform_drivers_list fake_sys_module_list \
         fake_usb_manufacturer fake_usb_product fake_usb_serial \
         fake_usb_idvendor fake_usb_idproduct fake_usb_version \
         fake_usb_busnum fake_usb_devnum \
         fake_pci_vendor fake_pci_device \
         fake_pci_dev_uevent fake_pci_dev_modalias fake_pci_revision fake_kgsl_gpubusy \
         fake_bat_charge_full fake_bat_charge_full_design \
         fake_bat_model fake_bat_manufacturer \
         fake_cpuinfo fake_build_prop \
         fake_cpufreq_prime fake_cpufreq_perf fake_cpufreq_min \
         fake_cpu_online fake_kallsyms fake_iomem fake_ioports \
         fake_proc_misc fake_meminfo fake_thermal_zone0 fake_proc_stat fake_cgroup \
         fake_version fake_proc_net_tcp fake_proc_net_tcp6 fake_proc_net_dev \
         fake_sys_kernel_ostype fake_sys_kernel_osrelease \
         fake_sys_kernel_version fake_sys_kernel_hostname; do
    cp "$MODDIR"/assets/$f /data/local/tmp/gs_${f} 2>/dev/null || true
    chmod 644 /data/local/tmp/gs_${f} 2>/dev/null || true
done
cp "$MODDIR"/assets/fake_mounts "$MODDIR/fake_mounts" 2>/dev/null || true
# NOTE: NEVER bind /proc/mounts or /proc/self/mountinfo globally — Mesa
# Gallium + Bionic cgroups need real mount paths; global bind crashes
# SurfaceFlinger in primeCache() and deadlocks system_server (2026-09-18).
# Mounts masking is per-process only via file_hooks.cpp open/openat/fopen
# redirect serving /data/local/tmp/gs_fake_mounts.
# Block mask: hide host nvme0n1 from /sys/block (Storage tab). Guarded:
# only when the dir exists; mmcblk0 reports 256GB (536870912 x 512B
# sectors); sda stays absent (UFS-only device, no SCSI disk).
if [ -d /sys/block ]; then
    mount -t tmpfs -o size=4k,mode=755 tmpfs /sys/block 2>/dev/null || true
    mkdir -p /sys/block/mmcblk0 2>/dev/null || true
    echo 536870912 > /sys/block/mmcblk0/size 2>/dev/null || true
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
# USB mask: DeviceInfoHW enumerates via opendir/getdents64, bypassing
# libc open hooks. Tmpfs blinds listing globally (kills SunplusIT/xhci).
if [ -d /sys/bus/usb/devices ]; then
    mount -t tmpfs -o size=4k,mode=755 tmpfs /sys/bus/usb/devices 2>/dev/null || true
    mkdir -p /sys/bus/usb/devices/usb1 2>/dev/null || true
    echo "Qualcomm Technologies, Inc" > /sys/bus/usb/devices/usb1/manufacturer 2>/dev/null || true
    echo "Snapdragon USB Controller" > /sys/bus/usb/devices/usb1/product 2>/dev/null || true
fi
# Audio/input/modules binds: global masks (per-process open redirect in
# file_hooks.cpp covers other mount namespaces).
if [ -f /proc/asound/cards ]; then
    mount -o bind "$MODDIR/assets/fake_asound_cards" /proc/asound/cards 2>/dev/null || true
fi
if [ -f /proc/bus/input/devices ]; then
    mount -o bind "$MODDIR/assets/fake_input_devices" /proc/bus/input/devices 2>/dev/null || true
fi
if [ -f /proc/modules ]; then
    mount -o bind "$MODDIR/assets/fake_modules" /proc/modules 2>/dev/null || true
fi
# WiFi/sound strings: DeviceInfoHW GENERAL tab reads these via sysfs/ALSA
# paths (not open-hooked): iwlwifi came from /sys/bus/pci DRIVER=iwlwifi
# (PCI_ID=8086:24FD), PCH from /sys/class/sound/card0/id + ALSA version
# cachyos. Mask /sys/module scan dir, sound id, ALSA version.
# NOTE 2026-09-19: NEVER tmpfs-mask /sys/bus/pci/devices, /sys/bus/pci/drivers,
# or /sys/devices/pci0000:00 globally — minigbm/libdrm need the Intel GPU
# at 0000:00:02 (0x8086) to allocate GBM BOs; blank tmpfs caused
# "Unable to create BO" + SF abort "output buffer not gpu writeable".
# PCI masking is per-process only (file_hooks.cpp / Vector).
if [ -d /sys/module ]; then
    mount -t tmpfs -o size=4k,mode=755 tmpfs /sys/module 2>/dev/null || true
    mkdir -p /sys/module/virtio_gpu /sys/module/virtio_input /sys/module/virtio_snd /sys/module/virtio_blk /sys/module/virtio_net 2>/dev/null || true
fi
# PCI sysfs left UNMASKED globally (see NOTE above).
if [ -d /sys/class/sound ]; then
    for f in /sys/class/sound/card*/id; do
        [ -f "$f" ] || continue
        echo SM8850 > "$f" 2>/dev/null || mount -o bind "$MODDIR/assets/fake_sound_id" "$f" 2>/dev/null || true
    done
fi
if [ -f /proc/asound/version ]; then
    mount -o bind "$MODDIR/assets/fake_asound_version" /proc/asound/version 2>/dev/null || true
fi
# ALSA detail nodes: GENERAL Sound string came from /proc/asound/card0/id
# (PCH), not cards/version. Bind-report SM8850 across id/cards/modules,
# codec (Realtek ALC257/Intel HDMI -> Qualcomm SM8850), and pcm info
# (ALC257 Analog -> SM8850 Audio).
if [ -f /proc/asound/card0/id ]; then
    mount -o bind "$MODDIR/assets/fake_sound_id" /proc/asound/card0/id 2>/dev/null || true
fi
if [ -f /proc/asound/modules ]; then
    mount -o bind "$MODDIR/assets/fake_asound_modules" /proc/asound/modules 2>/dev/null || true
fi
for f in /proc/asound/card0/codec#*; do
    [ -f "$f" ] || continue
    mount -o bind "$MODDIR/assets/fake_codec" "$f" 2>/dev/null || true
done
for f in /proc/asound/card0/pcm*/info; do
    [ -f "$f" ] || continue
    mount -o bind "$MODDIR/assets/fake_pcm_info" "$f" 2>/dev/null || true
done
# NOTE: PCI enumeration leaves (Wi-Fi DRIVER=iwlwifi + Intel IDs) stay
# per-process ONLY via file_hooks.cpp (serves /data/local/tmp/gs_fake_pci_*,
# copied in the loop above). NEVER global tmpfs/bind here — minigbm/libdrm
# need real PCI sysfs for Intel GPU BO alloc (2026-09-19 root cause).
