# Track D verification harness — per-vector checklist (proposal)

Refs: Track D task 5. Triple-compare = Waydroid prop value vs `adb getprop`
vs in-app read; any mismatch on a spoofed key = leak or stale layer.

## 0. Pre-flight (host, read-only)
- [ ] `waydroid prop get persist.waydroid.fake_touch` / `fake_wifi` match profile globs
- [ ] `.plans/track-d/waydroid-cfg-qemu-guard.sh` exits 0 (no qemu.* re-injection, no global `ro.hardware`)
- [ ] `cargo test -p wd-spoof` green (stable MAC unit tests)

## 1. Build / fingerprint vector
- [ ] Detector: **YASNAC** — expect BASIC + DEVICE, never promise STRONG
- [ ] Detector: **Integrity Checker** — verdict after each swap
- [ ] `dumpsys package <game> | grep -i fingerprint` sanity
- [ ] Triple-compare `ro.build.fingerprint` + 5 partition fingerprints
  (`ro.system[.ext].build.fingerprint`, `ro.vendor[.dlkm].build.fingerprint`, `ro.odm.build.fingerprint`)

## 2. Network identity vector (Track D: stable MACs)
- [ ] Triple-compare `wifi.mac`: `waydroid prop` vs `adb shell getprop` vs in-app `WifiInfo.getMacAddress`
- [ ] `WifiInfo.getBSSID` unicast (`& 0xfe`), stable across two session restarts
- [ ] `BluetoothAdapter.getAddress` distinct from wifi/BSSID, LA bit set
- [ ] No `02:00:00:00:00:0x` placeholder visible in any vector
- [ ] Lease file `/var/lib/misc/dnsmasq.waydroid0.leases` hostname matches profile

## 3. Emulator-denylist vector
- [ ] `getprop | grep -i qemu` empty inside container (guard covers cfg + base.prop paths)
- [ ] `ro.hardware` absent globally; per-process view only (Zygisk/Vector)
- [ ] Houdini intact: `ro.dalvik.vm.native.bridge=libhoudini.so`, x86 in abilist (host side)

## 4. Applist / attestation vector
- [ ] **Applist Detector** clean before HMA
- [ ] **KeyAttestation 1.8.4 offline**: `status.json`, AOSP box = DEVICE max

## 5. Command cheat-sheet
```sh
cargo test -p wd-spoof            # pure, allowed
./.plans/track-d/waydroid-cfg-qemu-guard.sh
adb shell getprop | grep -E 'fingerprint|qemu|hardware|wifi|bluetooth'
adb shell dumpsys wifi | grep -iE 'mac|bssid|ssid'
adb shell dumpsys bluetooth_manager | grep -iE 'address|name'
adb shell settings get secure android_id
cat /var/lib/misc/dnsmasq.waydroid0.leases
ip route show table all | grep '^default'   # VPN-flap watch
```
