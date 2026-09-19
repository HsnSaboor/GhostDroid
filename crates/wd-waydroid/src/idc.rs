//! Phantom IDC digitizer profile: pure names, text, and overlay paths.
//!
//! Forces Android to classify the virtual device as an internal
//! touchscreen. Pure builders only: no fs writes, no spawn.
//! Template matches vendor
//! `.devdocs/phantom/contrib/waydroid/Vendor_1234_Product_5678.idc` and the
//! shipped doc `profiles/idc/Vendor_1234_Product_5678.idc`.
//! Overlay layout mirrors vendor `phantom_paths` (`<work>/overlay/usr/idc`).

/// USB vendor id of the virtual touch device.
pub const VENDOR_ID: u16 = 0x1234;

/// USB product id of the virtual touch device.
pub const PRODUCT_ID: u16 = 0x5678;

/// Device name (sanitized into `<name>.idc` with spaces → `_`).
pub const DEVICE_NAME: &str = "GhostDroid Virtual Touch";

/// IDC file text: internal touchscreen, orientation aware.
pub const IDC_TEXT: &str = "# GhostDroid Virtual Touch\n\
# Force Android to classify GhostDroid as an internal touchscreen.\n\
\n\
device.internal = 1\n\
touch.deviceType = touchScreen\n\
touch.orientationAware = 1\n";

/// `Vendor_1234_Product_5678.idc` (Android VID/PID lookup name).
#[must_use]
pub fn vendor_filename() -> String {
    format!("Vendor_{VENDOR_ID:04x}_Product_{PRODUCT_ID:04x}.idc")
}

/// `<Device_Name_With_Underscores>.idc` (Android device-name lookup).
#[must_use]
pub fn device_filename() -> String {
    device_filename_for(DEVICE_NAME)
}

/// Sanitize a device name into `<sanitized>.idc` (pure port of vendor
/// `device_name_idc_filename`: alnum/`-`/`_` kept, rest → `_`).
#[must_use]
pub fn device_filename_for(name: &str) -> String {
    let mut base: String = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    if base.is_empty() {
        base.push('_');
    }
    base.push_str(".idc");
    base
}

/// `<work>/overlay/usr/idc` join (pure string path, no fs touch).
#[must_use]
pub fn overlay_idc_dir(work_dir: &str) -> String {
    format!("{work_dir}/overlay/usr/idc")
}

/// Full vendor IDC destination under `<work>/overlay/usr/idc`.
#[must_use]
pub fn vendor_idc_path(work_dir: &str) -> String {
    format!("{}/{}/", work_dir, "overlay/usr/idc") + &vendor_filename()
}

/// Full device-name IDC destination under `<work>/overlay/usr/idc`.
#[must_use]
pub fn device_idc_path(work_dir: &str) -> String {
    format!("{}/{}/", work_dir, "overlay/usr/idc") + &device_filename()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vendor_name_matches_android_lookup() {
        assert_eq!(vendor_filename(), "Vendor_1234_Product_5678.idc");
    }

    #[test]
    fn device_name_is_sanitized() {
        assert_eq!(device_filename(), "GhostDroid_Virtual_Touch.idc");
        assert_eq!(
            device_filename_for("Phantom Virtual Touch"),
            "Phantom_Virtual_Touch.idc"
        );
    }

    #[test]
    fn text_has_required_keys() {
        for key in [
            "touch.deviceType = touchScreen",
            "device.internal = 1",
            "touch.orientationAware = 1",
        ] {
            assert!(IDC_TEXT.contains(key), "missing {key}");
        }
    }

    #[test]
    fn overlay_paths_resolve() {
        assert_eq!(
            overlay_idc_dir("/var/lib/waydroid"),
            "/var/lib/waydroid/overlay/usr/idc"
        );
        assert_eq!(
            vendor_idc_path("/var/lib/waydroid"),
            "/var/lib/waydroid/overlay/usr/idc/Vendor_1234_Product_5678.idc"
        );
        assert_eq!(
            device_idc_path("/var/lib/waydroid"),
            "/var/lib/waydroid/overlay/usr/idc/GhostDroid_Virtual_Touch.idc"
        );
    }
}
