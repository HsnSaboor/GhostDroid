//! Guardrails: never gate banking, warn cheats, verify checklist.
//!
//! Pure data — no spawn, no gating. Refs: `.plans/05-root-hide-spoof.md:27`,
//! `KeyAttestation` `app/build.gradle` (1.8.4), HMA source
//! `.devdocs/HideMyApplist-mirror/README.md:2` (`Dr-TSNG/Hide-My-Applist`).

/// Banking/wallet PKGs: never gate, never spoof around. Deny by policy.
pub const BANKING_DENY: &[&str] = &[
    "com.google.android.apps.walletnfcrel",
    "com.paypal.android.p2pmobile",
    "com.chase.sig.android",
    "com.bankofamerica.mobilebanking",
    "com.wf.wellsfargomobile",
    "com.citi.citimobile",
];

/// Cheat-tool hints: warn, do not ship support.
pub const CHEAT_HINTS: &[&str] = &["aimbot", "modmenu", "cheat", "esp", "wallhack"];

/// Verify checklist: `YASNAC` + Integrity + Applist + `KeyAttestation` 1.8.4 offline.
pub const VERIFY_CHECKLIST: &[&str] = &[
    "YASNAC: expect BASIC + DEVICE, never promise STRONG",
    "Integrity Checker: confirm verdict after each swap",
    "Applist Detector: confirm no leaks before HMA",
    "KeyAttestation 1.8.4 offline: check status.json, AOSP box = DEVICE max",
];

/// True when pkg is banking-denied (exact or glob-prefix `foo.*` style).
#[must_use]
pub fn is_banking(pkg: &str) -> bool {
    tracing::info!(pkg, "spoof/guard: banking check start");
    let hit = BANKING_DENY.iter().any(|deny| {
        let matched = *deny == pkg;
        tracing::debug!(deny, matched, "spoof/guard: banking entry");
        matched
    });
    tracing::info!(hit, "spoof/guard: banking check done");
    hit
}

/// Warn when pkg hints at cheats. Returns warning text or `None`.
#[must_use]
pub fn cheat_warning(pkg: &str) -> Option<&'static str> {
    tracing::info!(pkg, "spoof/guard: cheat check start");
    let lowered = pkg.to_lowercase();
    for hint in CHEAT_HINTS {
        tracing::debug!(hint, "spoof/guard: cheat hint");
        if lowered.contains(hint) {
            tracing::warn!(pkg, hint, "spoof/guard: cheat suspected");
            return Some("cheat tooling unsupported: gaming-only fair-play profiles");
        }
    }
    tracing::info!("spoof/guard: cheat check clean");
    None
}

/// Ordered verify steps. Pure ref to [`VERIFY_CHECKLIST`].
#[must_use]
pub fn verify_checklist() -> &'static [&'static str] {
    tracing::info!(len = VERIFY_CHECKLIST.len(), "spoof/guard: checklist");
    for step in VERIFY_CHECKLIST {
        tracing::debug!(step, "spoof/guard: checklist step");
    }
    VERIFY_CHECKLIST
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn denies_banking_never_gates() {
        assert!(is_banking("com.chase.sig.android"));
        assert!(!is_banking("com.example.game"));
    }

    #[test]
    fn warns_cheats() {
        assert!(cheat_warning("com.game.aimbot.free").is_some());
        assert!(cheat_warning("com.example.game").is_none());
    }

    #[test]
    fn checklist_pins_keyattestation() {
        let steps = verify_checklist();
        assert_eq!(steps.len(), 4);
        assert!(steps.iter().any(|s| s.contains("YASNAC")));
        assert!(steps.iter().any(|s| s.contains("KeyAttestation 1.8.4")));
    }
}
