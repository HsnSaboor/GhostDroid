//! Stable per-profile network identity (`wifi.mac`, BSSID, BT).
//!
//! Seed = profile name + `android_id` via FNV-1a 64 (mirrors
//! `RandomGenerator.stableSeed`), expanded with `SplitMix64`. Std-only, pure,
//! no IO. Every output is locally-administered unicast (`byte0 & 0xfe | 0x02`),
//! stable across runs, distinct per profile and per role. Replaces the pinned
//! `02:00:00:00:00:0x` placeholders; explicit config overrides still win in
//! [`crate::render`].

/// Role tags keep wifi / BSSID / BT streams distinct under one seed.
const ROLE_WIFI: &str = "wifi";
/// BSSID role tag.
const ROLE_BSSID: &str = "bssid";
/// Bluetooth role tag.
const ROLE_BT: &str = "bt";

/// FNV-1a 64 over `profile + NUL + android_id + NUL + role`.
#[must_use]
pub fn stable_seed(profile_name: &str, android_id: &str, role: &str) -> u64 {
    tracing::debug!("spoof/mac: seeding");
    let mut hash: u64 = 0xCBF2_9CE4_8422_2325;
    for byte in profile_name
        .bytes()
        .chain([0])
        .chain(android_id.bytes())
        .chain([0])
        .chain(role.bytes())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    tracing::debug!("spoof/mac: seeded");
    hash
}

/// One `SplitMix64` step.
const fn next_u64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Fill 6 bytes from `seed`, forcing locally-administered unicast
/// (`byte0 = (b & 0xfe) | 0x02`: multicast cleared, LA set).
fn bytes_for(seed: u64) -> [u8; 6] {
    let mut state = seed;
    let hi = next_u64(&mut state).to_be_bytes();
    let lo = next_u64(&mut state).to_be_bytes();
    let raw = [hi[0], hi[1], hi[2], hi[3], lo[0], lo[1]];
    let first = (raw[0] & 0xfe) | 0x02;
    tracing::debug!(first, "spoof/mac: bytes derived");
    [first, raw[1], raw[2], raw[3], raw[4], raw[5]]
}

/// Stable wifi MAC for `profile_name + android_id`. LA + unicast.
#[must_use]
pub fn generate_stable_mac(profile_name: &str, android_id: &str) -> [u8; 6] {
    tracing::info!("spoof/mac: wifi mac start");
    let out = bytes_for(stable_seed(profile_name, android_id, ROLE_WIFI));
    tracing::info!("spoof/mac: wifi mac done");
    out
}

/// Stable BSSID for `profile_name + android_id`. Unicast (`& 0xfe`), LA set.
#[must_use]
pub fn generate_stable_bssid(profile_name: &str, android_id: &str) -> [u8; 6] {
    tracing::info!("spoof/mac: bssid start");
    let out = bytes_for(stable_seed(profile_name, android_id, ROLE_BSSID));
    tracing::info!("spoof/mac: bssid done");
    out
}

/// Stable Bluetooth MAC for `profile_name + android_id`. LA + unicast,
/// distinct from wifi/BSSID streams.
#[must_use]
pub fn generate_stable_bt(profile_name: &str, android_id: &str) -> [u8; 6] {
    tracing::info!("spoof/mac: bt mac start");
    let out = bytes_for(stable_seed(profile_name, android_id, ROLE_BT));
    tracing::info!("spoof/mac: bt mac done");
    out
}

/// Render `xx:xx:xx:xx:xx:xx` lowercase.
#[must_use]
pub fn format_mac(bytes: &[u8; 6]) -> String {
    tracing::debug!("spoof/mac: format");
    format!(
        "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]
    )
}

/// Parse strict lowercase-or-upper `xx:xx:xx:xx:xx:xx`. `None` when malformed.
#[must_use]
pub fn parse_mac(text: &str) -> Option<[u8; 6]> {
    tracing::debug!("spoof/mac: parse check");
    let parts: Vec<&str> = text.trim().split(':').collect();
    if parts.len() != 6 {
        return None;
    }
    let mut out = [0_u8; 6];
    for (index, part) in parts.iter().enumerate() {
        if part.len() != 2 {
            return None;
        }
        out[index] = u8::from_str_radix(part, 16).ok()?;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NAME: &str = "s26-ultra";
    const AID: &str = "a1b2c3d4e5f60708";

    #[test]
    fn la_bit_set_unicast() {
        for mac in [
            generate_stable_mac(NAME, AID),
            generate_stable_bssid(NAME, AID),
            generate_stable_bt(NAME, AID),
        ] {
            assert_eq!(mac[0] & 0x01, 0, "unicast (multicast bit clear)");
            assert_eq!(mac[0] & 0x02, 0x02, "locally-administered bit set");
        }
    }

    #[test]
    fn bssid_unicast_mask() {
        let bssid = generate_stable_bssid(NAME, AID);
        assert_eq!(bssid[0] & 0xfe | 0x01, bssid[0] | 0x01);
        assert_eq!(bssid[0] & 0x01, 0);
    }

    #[test]
    fn stable_across_runs() {
        assert_eq!(
            generate_stable_mac(NAME, AID),
            generate_stable_mac(NAME, AID)
        );
        assert_eq!(
            generate_stable_bssid(NAME, AID),
            generate_stable_bssid(NAME, AID)
        );
        assert_eq!(generate_stable_bt(NAME, AID), generate_stable_bt(NAME, AID));
        assert_eq!(
            format_mac(&generate_stable_mac(NAME, AID)),
            format_mac(&generate_stable_mac(NAME, AID))
        );
    }

    #[test]
    fn distinct_per_profile() {
        let a = generate_stable_mac("s26-ultra", AID);
        let b = generate_stable_mac("cheetah", AID);
        assert_ne!(a, b, "profile name changes MAC");
        let c = generate_stable_mac(NAME, "0000000000000001");
        assert_ne!(a, c, "android_id changes MAC");
    }

    #[test]
    fn roles_distinct() {
        let wifi = generate_stable_mac(NAME, AID);
        let bssid = generate_stable_bssid(NAME, AID);
        let bt = generate_stable_bt(NAME, AID);
        assert_ne!(wifi, bssid);
        assert_ne!(wifi, bt);
        assert_ne!(bssid, bt);
    }

    #[test]
    fn format_shape() {
        let text = format_mac(&generate_stable_mac(NAME, AID));
        assert_eq!(text.len(), 17);
        assert_eq!(text.matches(':').count(), 5);
        assert!(text.chars().all(|c| c.is_ascii_hexdigit() || c == ':'));
        assert_eq!(parse_mac(&text), Some(generate_stable_mac(NAME, AID)));
    }

    #[test]
    fn parse_rejects_malformed() {
        assert!(parse_mac("").is_none());
        assert!(parse_mac("02:00:00:00:00").is_none());
        assert!(parse_mac("02:00:00:00:00:00:00").is_none());
        assert!(parse_mac("02-00-00-00-00-00").is_none());
        assert!(parse_mac("zz:00:00:00:00:00").is_none());
        assert!(parse_mac("2:00:00:00:00:00").is_none());
    }

    #[test]
    fn never_placeholder() {
        for mac in [
            generate_stable_mac(NAME, AID),
            generate_stable_bssid(NAME, AID),
            generate_stable_bt(NAME, AID),
        ] {
            assert_ne!(mac, [0x02, 0, 0, 0, 0, 0]);
            assert_ne!(mac, [0x02, 0, 0, 0, 0, 1]);
            assert_ne!(mac, [0x02, 0, 0, 0, 0, 2]);
        }
    }
}
