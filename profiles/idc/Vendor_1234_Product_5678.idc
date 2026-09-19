# GhostDroid Virtual Touch
# Force Android to classify GhostDroid as an internal touchscreen.
# Ref: `.devdocs/phantom/contrib/waydroid/Vendor_1234_Product_5678.idc`.
# Installed to `<waydroid-work>/overlay/usr/idc/` (see `wd-waydroid::idc`).

device.internal = 1
touch.deviceType = touchScreen
touch.orientationAware = 1
