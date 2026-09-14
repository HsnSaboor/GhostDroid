# Root + hide + spoof — Sep 2026 state (verified)

Date: 2026-09-14. Prior tablet assumed old stack; this replaces it.

## Root solutions

| Root | Verdict | Version | Repo | Notes |
|---|---|---|---|---|
| Magisk topjohnwu | ALIVE, canonical | v31.0 pre (2026-09-02), v30.7 stable (2026-02) | topjohnwu/Magisk | Compose M3 UI, built-in terminal, Zygisk A17/API37 + Quest HorizonOS. Rust migration >40%. Only root that works on Waydroid (bootanim.rc inject, no boot patch). |
| Kitsune/Delta | DEAD/STALLED | — | HuskyDG dead; Jordan231111/KitsuneMagisk experimental | waydroid_script pins stale `mistrmochov/magiskdeltaorig` APK — swap source. |
| KernelSU tiann | ALIVE | v3.3.0 (2026-08-28) | tiann/KernelSU | GKI 5.10+, LKM mode, x86_64 panic caution. NO Waydroid path (needs kernel driver, host kernel shared). |
| KernelSU-Next rifsxd | ALIVE | v3.3.0 | KernelSU-Next/KernelSU-Next (local `.devdocs/KernelSU-Next` remote `rifsxd/...` — same project moved) | Kernels 4.4-6.6, Magic Mount+OverlayFS, susfs controls. NO Waydroid path. |
| APatch bmax121 | ALIVE slow | build 11224 (2026-08-07) | bmax121/APatch + KernelPatch | ARM64 only, 3.18-6.12. NO Waydroid path. |
| SukiSU Ultra | ACTIVE fork | v4.1.3 (2026-06-02) | SukiSU-Ultra/SukiSU-Ultra | Fork of tiann KSU + non-GKI + KPM + susfs mgmt. NO Waydroid path. |
| ReSukiSU/BPatch/ApexSU | niche | ReSukiSU v4.2-rc1 (dropped KPM) | misc | skip. |

Waydroid verdict: stay Magisk-injection. `waydroid_script/stuff/magisk.py` appends post-fs-data/nonencrypted/boot_completed execs, extracts lib/*.so from APK. KSU-family needs own kernel — impossible in LXC container.

## Zygisk + hide (all verified Sep 2026)

- ZygiskNext: `Dr-TSNG/ZygiskNext` EXISTS (websearch confirms v1.5.0 2026-08-26, A17 QPR2 Beta3, HyperOS Runtime). Earlier `git clone https://github.com/Dr-TSNG/ZygiskNext` → "Repository not found" + `gh search` missed it (case/casing + default-branch `copyright` weirdness) — use web URL / zip from releases or kernelsu modules mirror. Closed-source since v4-0.9.2. Magisk>=26402, built-in OFF.
- ReZygisk: `PerformanC/ReZygisk` v1.0.0 (2026-05-11, local clone OK). Open-source C rewrite, API v4 parity. Pick if distrust closed blob.
- Shamiko: LSPosed.io releases only, v1.2.5 (2025-06-18, closed). Still go-to hider. DenyList configured, Enforce OFF.
- LSPosed → Vector: `JingMatrix/LSPosed` renamed `JingMatrix/Vector` (local `.devdocs/Vector` cloned, pushed 2026-09-14 today). Vector v2.2-canary, v2.0/2.1 stable; LSPosed v1.11.0 (2026-01-31, A16) fallback. Needs NeoZygisk (local `.devdocs/NeoZygisk` cloned, ptrace zygote inject, clean-mount-namespace hide). Old `.devdocs/LSPosed` remote = dead `LSPosed/LSPosed`.
- HideMyApplist: source `Dr-TSNG/Hide-My-Applist` (capital M-A, hyphen) — NOT `HideMyApplist`. Clone failed on wrong case; correct source exists per modules.lsposed.org (v3.8.3 2026-08-01, API 101, A11+ min). Local mirror: `.devdocs/HideMyApplist-mirror` (Xposed-Modules-Repo/com.tsng.hidemyapplist, v3.8.3 confirmed). Template+blacklist still valid.

Gaming combo (BASIC+DEVICE): Magisk + built-in Zygisk (or ZygiskNext 1.5.0) + Shamiko 1.2.5 + PlayIntegrityFork v18 + Vector 2.x + HMA 3.8.3 only if applist kick. Start minimal Zygisk+Shamiko+PIFork.

## PIF / TrickyStore

- `chiteroman/PlayIntegrityFix` dead/removed. `osm0sis/PlayIntegrityFork` v18 (2026-08-29) canonical (local clone OK). `KOWX712/PlayIntegrityFix` inject-s mirror. No bundled print; `autopif4.sh --strong`. Beta/Canary prints = STRONG-only now, DEVICE needs private print. Public prints die days, canary expiry ~6wk.
- TrickyStore `5ec1cff/TrickyStore` v1.4.1 (2025-11, closed since 1.1.0, local OK). Addon `KOWX712/Tricky-Addon-Update-Target-List` v4.4 (2026-06, local OK). No durable public keybox 2026; RKP RSA-4096 mandatory Feb-Apr 2026 kills old 2048 boxes. For BASIC+DEVICE skip keybox hunt.

## Sensor / SIM / network spoof

- Sensor noise: NO maintained LSPosed module for realistic accel/gyro/mag noise. MotionEmulator/sensor-disabler/VirtualSensor all dead. Closest alive: XposedFakeLocation (GPS only), LocationSpoofer (gait/step), PriviSense Frida toolkit (copy `dispatchSensorEvent` hook + OU noise yourself).
- SIM/teleop (new clones): `.devdocs/TelephonySpoofer` (brianwalczak, all-in-one cellular+eSIM), `.devdocs/Lsposed-SimSpoof` (K0rnhulio, hardcoded values — rebuild), `.devdocs/TikTokSimSpoof` (itachicoders, 40+ MCC/MNC presets, best code ref — fork scope to games).
- Network (new clone): `.devdocs/waydroid_network_spoof` (arkaroy14, Mar 2026, README-only repo — LSPosed module fakes eth0/TRANSPORT_ETHERNET/9 → wlan0/WIFI+CELLULAR/1/LTE13; fixes apps where `fake_wifi` fails).
- All-in-one alive: DeviceSpoofLab-Hooks + Magisk (local OK, v1.2 Jun 2026, Build+SystemProperties/Telephony/Network/Locale, no sensor noise). AVD_Rooted_Integrity (tanishmeh, AVD STRONG recipe, portable ideas only).
- Waydroid built-in: `persist.waydroid.fake_touch/fake_wifi` per-package; fake_wifi re-merged Oct 2025 (vendor_waydroid PR#44); virt_wifi PR#2211 (A13/A14); fake_touch buggy (#30). Sensor passthrough: `waydroid-sensors/sensord` exists but desktop x86_64 = no backend → must fake inside Android. `Quackdoc/spoof-device.sh` Pixel5 redfin 2023 print = dead for DEVICE.
- UNFAKEABLE: STRONG w/ HW keybox + locked chain; GPU string (ANGLE/SwiftShader); zero-variance sensors; eth0-only; <5 sensors; DroidGuard/Frida checks. Aim DEVICE + per-game shims; expect cat-mouse ~6wk.

## Frida verdict (production vs lab)

- Ship? No. Lab? Yes.
- Why no-ship: `frida-server` detectable via `/proc/self/maps` (`frida`, `linjector`), TCP 27042, DroidGuard/anti-cheat maps+port scan. No reboot persistence. Per-app Gadget = repack = signature break.
- Use Frida for: recon (which API game calls), prototype hook before writing Vector module, AndroGoat-style lab bypass (`objection` codeshare).
- Ship instead: own Vector/LSPosed module `sensor-noise` — copy PriviSense `dispatchSensorEvent` hook + OU-process noise (alpha 0.05, 4m clamp pattern). Zygisk in-zygote > frida-server stealth. Same for SIM/teleop: fork TikTokSimSpoof scope to games, not Frida.
- Stack order: props → ARM bridge → Magisk/Zygisk → Shamiko → PIFork → Vector modules (HMA + network_spoof + sensor-noise + telephony) → launch. Frida sits outside, dev-only sidecar.

## ZygiskNext closed-source note

- Yes: free but proprietary since v4-0.9.2, all rights reserved. Source hidden.
- Closed ≠ unpatchable. Google detects behavior (mounts, maps, syscalls, DroidGuard heuristics), not source text. Closed only slows signature/YARA + raises reverse cost. ZN Linker + SoInfo hiding + umount mgmt = real value, not secrecy alone.
- Open alt: ReZygisk v1.0.0 (local OK), API v4 parity. Pick ReZygisk if distrust blob; ZygiskNext if max hide.
- Emulator call: start built-in Magisk Zygisk + Shamiko. Add ZygiskNext/ReZygisk only on detection hit. ponytail: need it? no day-one. add on kick.

## Corrections to prior tablets

- 00/03 said "ZygiskNext URL wrong/missing" — corrected: repo exists, clone fails via git (case/branch oddity), fetch via release zip.
- 00/03 said "HMA only mirror" — corrected: source `Dr-TSNG/Hide-My-Applist`, mirror local is fine fallback, v3.8.3 verified.
- 00/03 said "LSPosed JingMatrix" — updated: renamed Vector, needs NeoZygisk, API 101.
- waydroid_script Magisk APK pin stale — action: swap to Magisk v30.7+/v31 APK, keep bootanim.rc injector.
