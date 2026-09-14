//! Vector shims: sensor-noise + telephony + network tables.
//!
//! Pure data, no spawn. Refs: `.plans/05-root-hide-spoof.md:20-23`.

/// OU noise alpha (`PriviSense` pattern).
pub const SENSOR_ALPHA: f32 = 0.05;
/// OU clamp in meters.
pub const SENSOR_CLAMP_M: f32 = 4.0;
/// Noise floor low m/s^2 (variance test fail below).
pub const NOISE_FLOOR_LO: f32 = 0.01;
/// Noise floor high m/s^2.
pub const NOISE_FLOOR_HI: f32 = 0.05;
/// Hook target for sensor shim.
pub const SENSOR_HOOK: &str = "dispatchSensorEvent";

/// Sensor-noise spec text. Traced.
#[must_use]
pub fn sensor_spec() -> &'static str {
    tracing::info!(
        alpha = SENSOR_ALPHA,
        clamp = SENSOR_CLAMP_M,
        "spoof/shim: sensor spec"
    );
    tracing::debug!(
        hook = SENSOR_HOOK,
        lo = NOISE_FLOOR_LO,
        hi = NOISE_FLOOR_HI,
        "spoof/shim: detail"
    );
    "OU alpha 0.05, 4m clamp, floor 0.01-0.05; hook dispatchSensorEvent"
}

/// Default MCC+MNC (us `T-Mobile`).
pub const DEFAULT_MCC_MNC: &str = "31026";
/// LTE network type.
pub const NETWORK_TYPE_LTE: i32 = 13;

/// Carrier table (iso, mccmnc, name). Ref `SpoofConfig.kt` presets.
pub const MCC_TABLE: &[(&str, &str, &str)] = &[
    ("us", "31026", "T-Mobile"),
    ("us", "31101", "Verizon"),
    ("us", "310410", "AT&T"),
    ("gb", "23430", "EE"),
    ("fr", "20801", "Orange"),
];

/// Telephony shim summary. Traced.
#[must_use]
pub fn telephony_spec() -> &'static str {
    tracing::info!(
        entries = MCC_TABLE.len(),
        def = DEFAULT_MCC_MNC,
        "spoof/shim: telephony"
    );
    for (iso, mccmnc, name) in MCC_TABLE {
        tracing::debug!(iso, mccmnc, name, "spoof/shim: mcc");
    }
    "scope games; XSharedPrefs runtime-edit; default us 31026 LTE13"
}

/// Net map (api, real, spoofed). Ref `waydroid_network_spoof` table.
pub const NET_TABLE: &[(&str, &str, &str)] = &[
    ("hasTransport(ETHERNET)", "true", "false"),
    ("hasTransport(WIFI)", "false", "true"),
    ("hasTransport(CELLULAR)", "false", "true"),
    ("NetworkInfo.getType", "9 ETHERNET", "1 WIFI"),
    ("NetworkInfo.getTypeName", "Ethernet", "WIFI"),
    ("WifiManager.isWifiEnabled", "false", "true"),
    ("WifiManager.getWifiState", "1", "3 ENABLED"),
    ("TelephonyManager.getNetworkType", "0", "13 LTE"),
    ("TelephonyManager.getDataState", "-", "2 CONNECTED"),
    ("ActiveNetworkInfo.mType", "9", "1"),
    ("interface", "eth0", "wlan0"),
];

/// Network shim summary. Traced.
#[must_use]
pub fn network_spec() -> &'static str {
    tracing::info!(entries = NET_TABLE.len(), "spoof/shim: network");
    for (api, real, spoofed) in NET_TABLE {
        tracing::debug!(api, real, spoofed, "spoof/shim: net");
    }
    "eth0->wlan0; WIFI wins; restart required; NetworkCallback still real; DNS eth0"
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn specs_pinned() {
        assert!((SENSOR_ALPHA - 0.05).abs() < f32::EPSILON);
        assert!((SENSOR_CLAMP_M - 4.0).abs() < f32::EPSILON);
        assert!(sensor_spec().contains("0.05"));
        assert_eq!(DEFAULT_MCC_MNC, "31026");
        assert_eq!(NETWORK_TYPE_LTE, 13);
        assert!(MCC_TABLE.iter().any(|m| m.1 == "31026"));
        assert!(telephony_spec().contains("31026"));
        assert!(
            NET_TABLE
                .iter()
                .any(|n| n.0 == "interface" && n.2 == "wlan0")
        );
        assert!(network_spec().contains("wlan0"));
    }
}
