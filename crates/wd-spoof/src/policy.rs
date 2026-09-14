//! FULL hide/spoof stack policy.
//!
//! Pure data, no spawn. Refs: `.plans/05-root-hide-spoof.md:8-16`.

/// Layers in apply order. Discriminant = position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(missing_docs)]
#[rustfmt::skip]
pub enum Layer { BaseProps, ArmBridge, Magisk, Shamiko, Pif, TrickyStore, VectorModules }

struct Meta {
    pin: Option<&'static str>,
    url: Option<&'static str>,
    enable: &'static str,
}
#[rustfmt::skip]
const METAS: [Meta; 7] = [
    Meta { pin: None, url: None, enable: "always" },
    Meta { pin: None, url: None, enable: "always" },
    Meta { pin: Some("v30.7+/v31"), url: None, enable: "minimal" },
    Meta { pin: Some("1.2.5"), url: Some("https://lsposed.io/Shamiko"), enable: "minimal" },
    Meta { pin: Some("v18/180001"), url: Some("https://github.com/osm0sis/PlayIntegrityFork"), enable: "minimal-autopif" },
    Meta { pin: Some("v1.4.1/245+addon-v4.4"), url: Some("https://github.com/5ec1cff/TrickyStore"), enable: "integrity-keybox-empty-user-only" },
    Meta { pin: Some("v2.2/3080+neo-v2.4/289+hma-3.8.3"), url: Some("https://github.com/JingMatrix/Vector"), enable: "kick-neo-nozygisk-hma-applist" },
];
#[rustfmt::skip]
impl Layer {
    /// Position (`BaseProps` = 0).
    #[must_use] pub fn order(self) -> u8 { tracing::debug!(layer = ?self, "spoof/policy: order"); self as u8 }
    fn meta(self) -> &'static Meta { tracing::debug!(layer = ?self, "spoof/policy: meta"); &METAS[self as usize] }
    /// Version pin (`None` = host-side).
    #[must_use] pub fn pin(self) -> Option<&'static str> { let p = self.meta().pin; tracing::debug!(layer = ?self, ?p, "spoof/policy: pin"); p }
    /// Fetch URL (`None` = host-side; `ZygiskNext` zip-only, clone fails).
    #[must_use] pub fn url(self) -> Option<&'static str> { let u = self.meta().url; tracing::debug!(layer = ?self, ?u, "spoof/policy: url"); u }
    /// Enable condition.
    #[must_use] pub fn enable(self) -> &'static str { let c = self.meta().enable; tracing::debug!(layer = ?self, c, "spoof/policy: enable"); c }
}
/// All layers in apply order.
#[rustfmt::skip]
pub const ALL: [Layer; 7] = [Layer::BaseProps, Layer::ArmBridge, Layer::Magisk, Layer::Shamiko, Layer::Pif, Layer::TrickyStore, Layer::VectorModules];
/// Escalation units: closed/open `Zygisk` swap + `HMA` + Vector shims.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
#[rustfmt::skip]
pub enum Extra { ZygiskNext, ReZygisk, Hma, SensorShim, TeleShim, NetShim }
#[rustfmt::skip]
const EXTRA_META: [(&str, &str, &str); 6] = [
    ("1.5.0-closed-zip", "https://github.com/Dr-TSNG/ZygiskNext/releases", "zygisk-hit-only"),
    ("1.0.0-open", "https://github.com/PerformanC/ReZygisk/releases", "zygisk-hit-only"),
    ("3.8.3-api101", "https://github.com/Dr-TSNG/Hide-My-Applist", "applist-kick-only"),
    ("ou-alpha-0.05", "local-vector-module", "sensor-kick"),
    ("mcc-us-31026", "local-vector-module", "carrier-kick"),
    ("eth0-wlan0", "local-vector-module", "eth-check-kick"),
];
#[rustfmt::skip]
impl Extra {
    fn meta(self) -> &'static (&'static str, &'static str, &'static str) { tracing::debug!(extra = ?self, "spoof/policy: extra meta"); &EXTRA_META[self as usize] }
    /// Version pin.
    #[must_use] pub fn pin(self) -> &'static str { let p = self.meta().0; tracing::debug!(extra = ?self, p, "spoof/policy: extra pin"); p }
    /// Fetch URL.
    #[must_use] pub fn url(self) -> &'static str { let u = self.meta().1; tracing::debug!(extra = ?self, u, "spoof/policy: extra url"); u }
    /// Enable condition.
    #[must_use] pub fn enable(self) -> &'static str { let c = self.meta().2; tracing::debug!(extra = ?self, c, "spoof/policy: extra enable"); c }
}
/// Minimal start: built-in `Zygisk` + `Shamiko` + `PIFork`.
#[must_use]
pub fn minimal_start() -> &'static [Layer] {
    let s = &ALL[2..5];
    tracing::info!(len = s.len(), "spoof/policy: minimal start");
    s
}
/// Kick driving escalation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
#[rustfmt::skip]
pub enum Kick { Integrity, Applist, Sensor, Carrier, ZygiskHit }
/// Layers to ADD (missing from `active` only). Pure.
#[must_use]
pub fn escalate_on_kick(active: &[Layer], kick: Kick) -> Vec<Layer> {
    tracing::info!(?kick, n = active.len(), "spoof/policy: escalate start");
    let want: &[Layer] = match kick {
        Kick::Integrity => &[Layer::TrickyStore],
        Kick::Applist | Kick::Sensor | Kick::Carrier | Kick::ZygiskHit => &[Layer::VectorModules],
    };
    let add: Vec<Layer> = want
        .iter()
        .copied()
        .filter(|l| {
            let m = !active.contains(l);
            tracing::debug!(layer = ?l, m, "spoof/policy: check");
            m
        })
        .collect();
    tracing::info!(add = ?add, "spoof/policy: escalate done");
    add
}
/// Extras for a kick. Pure.
#[must_use]
pub fn extras_for_kick(kick: Kick) -> &'static [Extra] {
    tracing::info!(?kick, "spoof/policy: extras start");
    let e: &[Extra] = match kick {
        Kick::Integrity | Kick::Sensor => &[Extra::SensorShim],
        Kick::Applist => &[Extra::Hma, Extra::NetShim],
        Kick::Carrier => &[Extra::TeleShim],
        Kick::ZygiskHit => &[Extra::ZygiskNext, Extra::ReZygisk],
    };
    tracing::info!(len = e.len(), "spoof/policy: extras done");
    e
}
/// NEVER bundle keybox/prints.
pub const BUNDLED_SECRETS_ALLOWED: bool = false;
/// Bundled secrets always forbidden.
#[must_use]
pub fn bundled_secrets_allowed() -> bool {
    tracing::warn!("spoof/policy: bundled secrets forbidden");
    BUNDLED_SECRETS_ALLOWED
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_stack() {
        assert!(ALL.windows(2).all(|w| w[0].order() < w[1].order()));
        assert_eq!(
            minimal_start(),
            &[Layer::Magisk, Layer::Shamiko, Layer::Pif]
        );
        assert_eq!(Layer::Pif.pin(), Some("v18/180001"));
        assert!(Layer::Pif.url().is_some());
        assert_eq!(Extra::ZygiskNext.pin(), "1.5.0-closed-zip");
        assert_eq!(extras_for_kick(Kick::ZygiskHit).len(), 2);
        assert!(extras_for_kick(Kick::Applist).contains(&Extra::Hma));
        assert!(!bundled_secrets_allowed());
    }
}
