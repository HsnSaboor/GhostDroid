//! wd-spoof: per-game prop templates + snapshot + fake_* match.
//!
//! YAGNI: render + snapshot only, no LXC spawn, no keybox.
//! Refs: `.plans/04-waydroid-mgmt.md:18-20`,
//! `.devdocs/DeviceSpoofLab-Hooks/device_profile.conf:260`.
#![deny(missing_docs)]

pub mod apply;
pub mod fake_match;
pub mod guardrails;
pub mod mac;
pub mod modules;
pub mod policy;
pub mod profile;
pub mod render;
pub mod snapshot;

pub use apply::{merge_lines, verify_fingerprint};
pub use fake_match::{glob_match, list_match, touch_set_args, wifi_set_args};
pub use guardrails::{
    BANKING_DENY, CHEAT_HINTS, VERIFY_CHECKLIST, cheat_warning, is_banking, verify_checklist,
};
pub use mac::{
    format_mac, generate_stable_bssid, generate_stable_bt, generate_stable_mac, parse_mac,
    stable_seed,
};
pub use modules::{MCC_TABLE, NET_TABLE, network_spec, sensor_spec, telephony_spec};
pub use policy::{
    ALL, BUNDLED_SECRETS_ALLOWED, Extra, Kick, Layer, bundled_secrets_allowed, escalate_on_kick,
    extras_for_kick, minimal_start,
};
pub use profile::{REQUIRED_KEYS, SpoofProfile, StackPreset, has_keys};
pub use render::{BASE_PROP, render, swap_diff, try_render};
pub use snapshot::{PropSnapshot, parse_kv, restore, snapshot};
