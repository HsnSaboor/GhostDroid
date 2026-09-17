# DeviceSpoofLab-Hooks

LSPosed module that spoofs device identifiers, build properties, telephony,
network, locale, WebView fingerprinting surfaces, and native libc property
reads inside hooked target processes.

## Requirements

- Rooted Android with LSPosed or Fork.
- Android 8.0+ / SDK 26+ on the target device.

## Install

1. Install LSPosed via Magisk, KernelSU, or another supported root manager.
2. Install the `app-debug.apk` from releases.
3. Enable the module in LSPosed Manager, and add your target apps to the scope. Additionally, add the Google Play Services app for GAID spoofing.
4. Open the DeviceSpoofLab-Hooks app, randomize identifiers to liking, then force-stop and relaunch the target apps.

## Troubleshooting

If the target app isn't seeing the spoofed values:

- Confirm the target app is in the module's scope and the module is enabled.
  For updates, uninstall the com.devicespooflab.hooks app fully then reinstall the app-debug.apk from releases.
- Force-stop the target app afterwards so it picks up the new values
  cleanly instead of caching whatever it read on startup.

Inadequate hooks or Xposed detected:

- Try using an LSPosed fork, such as Vector by Jing Matrix that has more undetectable injection.
- Make an Issue on Github and explain the issue your experiencing.

## License

This project is licensed under the [MIT License](https://opensource.org/licenses/MIT).
