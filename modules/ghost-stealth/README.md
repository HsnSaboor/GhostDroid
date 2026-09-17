# GhostDroid Stealth — per-process spoof companion to TargetedFix.
#
# Gap closed: TargetedFix hooks only __system_property_read_callback
# (tfsrc main.cpp:77). reveny detector reads via __system_property_get
# (FileHelper.hpp:107), bypassing it. gs_native hooks get+find+read+
# read_callback (copied MIT from DeviceSpoofLab-Hooks cpp/, ns ds->gs).
#
# Layout (DRY, one concern per file):
#   native/      props+uname+mac hooks (gs_native.so, LSPlt, CI-built)
#   zygisk/      per-process gate (target.txt) + gs_install_buf feed
#   config/      spoof.conf (S26 map) + target.txt (5 detectors)
#   assets/      fake_cpuinfo (SD8 Elite, service.sh binds /proc/cpuinfo)
#
# Keep stack: ReZygisk (runtime), Shamiko (hide, denylist OFF),
# PIFork/TrickyStore (attestation layers), Vector+HMA (applist).
# This module replaces TargetedFix only.
#
# Build (CI only, never local i5): NDK r26+, APP_ABI
# x86_64+x86+arm64-v8a+armeabi-v7a -> zygisk/*.so + gs_native.so.
# Vector mining: DeviceInfoHW (apktool /tmp/diwh, Runtime.exec+SystemProperties
# +Os.uname+Build.SOC_MODEL+SUPPORTED_ABIS+sysfs), SPIC/KeyAtt/YASNAC
# (server verdicts, local JWS/chain parse only), emu-demo (7 checks, no extras).
