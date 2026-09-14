# 04 Waydroid mgmt — installs, prop swap, apps, backup

Refs: `.research/03-tech-stack.md:89-117`, `.research/04-root-hide-spoof-2026.md:17,40,62-63`; `.devdocs/waydroid_script/{main.py,requirements.txt:tqdm/requests/InquirerPy,stuff/{magisk.py:18-163,general.py:resetprop.rc,ndk.py,houdini.py,gapps.py,widevine.py,microg.py,mitm.py},tools/{container.py:overlayfs/stop/upgrade,helper.py:dirs/run/shell/download/host/backup,images.py:mount/resize+500M,logger.py}}`; `.devdocs/waydroid/tools/{helpers/props.py,helpers/version.py,actions/prop.py,actions/*manager.py}`, `data/configs/config_base`, `data/scripts/waydroid-{net,post-stop}.sh`, `debian/changelog:1.6.3`; `.devdocs/waydroid-settings/.../utils.py:BASE_PROP_LOC/echo_command_to_wd_base`, `0.3.0`; `.devdocs/waydroid-scripts/spoof-device.sh:19 redfin`; `.devdocs/Waydroid-total-spoof/{waydroid.sh:20,V2.0.sh:25,README.md}`; `.devdocs/DeviceSpoofLab-Hooks/device_profile.conf:260 cheetah`; `.devdocs/waydroid-mcp/src/waydroid_mcp/{bootstrap.py,cli.py,core.py}`.

## Goal
Thin orchestrator over stock Waydroid 1.6.3. No installer rewrite. Per-game prop swap canonical.

## Reuse
- Installer logic: call `main.py install {gapps,microg,libndk,libhoudini,magisk,mitm,smartdock,widevine} remove hack nodataperm/hidestatusbar certified` (Android 11/13, overlayfs vs /tmp/waydroid mount+resize +500M). Wrap, don't port: `tools/container.py:use_overlayfs(WAYDROID_CONFIG mount_overlays)/stop/is_running/upgrade`, `helper.py:get_download_dir ~/.cache/get_data_dir ~/.local/share/run()/shell(sudo waydroid shell)/download_file/host(arch+SSE4.2)/backup/restore .gz`, `images.py:mount/umount/resize/get_image_dir`.
- Magisk APK fix (`.research/04-root-hide-spoof-2026.md:62`): `magisk.py:14 dl_link mistrmochov/magiskdeltaorig` STALE — swap to Magisk v30.7+/v31 APK. KEEP `magisk.py:18-163`: `oringinal_bootanim` stock service + `bootanim_component` (post-fs-data magiskpolicy×3 + mkdir /dev/magisk_* + setup-sbin + post-fs-data/nonencrypted/vold-restart/boot_completed/zygote-restart execs), `copy()` lib*.so→magisk64/32 + apk + chromeos/addon.d/stub, gzip backup, `extra1/delete_upper` overlay cleanup, `extra2` restore + wipe adb/magisk.db, `set_path_perm 0:2000`.
- Props core: `waydroid/tools/helpers/props.py:host_get/host_list/host_set/file_get(build.prop parser)/IPlatform setprop` + `actions/prop.py`. Persist pattern `waydroid-settings/utils.py:BASE_PROP_LOC=/var/lib/waydroid/waydroid_base.prop + echo_command_to_wd_base(tee -a)` + prop consts `persist.waydroid.multi_windows/suspend/width/height (+padding)`, `waydroid.blacklist/active_apps`, images `system.img`, scripts `~/.local/share/waydroid-settings/scripts`.
- Prop dict source = `DeviceSpoofLab-Hooks/device_profile.conf:260` (Pixel 7 Pro cheetah A16 SDK36, partitions product/system/system_ext/vendor*/odm/bootimage, emulator denylist qemu=0 verifiedboot green flash locked, hw gs201/mali/8core/12GB/1440x3120/512dpi, operator 310260 T-Mobile). Discard `spoof-device.sh` 2023 redfin print (dead DEVICE, `.research/04-root-hide-spoof-2026.md:40`) and `V2.0.sh` broken `sed '/^/,$d'` (keep its verified-boot list only). `waydroid.sh` android_id `openssl rand -hex 8` + gsf RANDOM + `session stop/start` flow = keep.

## Design
```
py-sidecar/wd_sidecar.py {install --json, certified, hack}  # thin shim, JSON stdio
wd-waydroid::props { snapshot/swap/restore waydroid_base.prop, runtime `waydroid prop set persist.waydroid.*` }
wd-spoof::templates { render profiles/spoof/<game>.toml {props[], fake_touch[], fake_wifi[], denylist[], pif_ref} }
profiles/spoof/<game>.toml  # props + globs + denylist ref (NOT secrets)
```
- Swap flow (02): stop → snapshot → render+append (tee -a pattern) → `upgrade --offline` → start+wait → clear gms+game → keymap+fake_* → launch. Runtime `prop set` needs session restart — enforce in daemon.
- Files: canonical `/var/lib/waydroid/waydroid_base.prop`, alt `waydroid.cfg [properties]`, images `system.img/vendor.img/rootfs/data/*` (`.research/03-tech-stack.md:117`).
- Apps: `install/uninstall/list/launch/stop/current` via `waydroid app` + adb (actions/app_manager.py). Backup: tar `data/*` + prop snapshot, `--confirm` required.

## Steps
1. Shim sidecar + JSON envelope, timeout+cgroup kill (02 supervisor). 2. Template renderer + snapshot/restore + 1 golden profile. 3. Swap flow e2e on test game. 4. Apps+backup with confirm.

## YAGNI
No image builder/OTA/store scraper. No print/keybox fetch here (05 owns policy).

## Blast radius
- Touches: `py-sidecar/*`, `crates/wd-spoof/templates/*`, `profiles/spoof/*` only. No engine/UI/MCP logic.
- Risk: HIGH (writes /var/lib/waydroid + overlay). Guard: snapshot before swap, `upgrade --offline` only, keep `original_bootanim`, dry-run renders diff first.
- Rollback: restore snapshot + `session restart`. Accept: swap e2e + restore green.
