//! Pointer grab state: lock edge toggled by Alt/Ctrl rising edge.
#![deny(missing_docs)]

/// Modifier pair (Alt/Ctrl level) for grab decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
    /// Alt held.
    pub alt: bool,
    /// Ctrl held.
    pub ctrl: bool,
}

impl Modifiers {
    /// New pair.
    #[must_use]
    pub const fn new(alt: bool, ctrl: bool) -> Self {
        Self { alt, ctrl }
    }
}

/// Level check: true while Alt or Ctrl is held (not an edge).
///
/// Kept for backward compatibility. Prefer [`grab_rising_edge`] for toggle
/// decisions so holding the modifier does not re-fire every poll.
#[must_use]
pub fn grab_edge(alt: bool, ctrl: bool) -> bool {
    tracing::trace!(alt, ctrl, "wd-input: grab level");
    alt || ctrl
}

/// Level check over a pair.
#[must_use]
pub fn grab_level(mods: Modifiers) -> bool {
    grab_edge(mods.alt, mods.ctrl)
}

/// Rising-edge check: true only on the press transition.
///
/// `prev` is the modifier state from the previous poll. Holding Alt across
/// polls fires exactly once.
#[must_use]
pub fn grab_rising_edge(mods: Modifiers, prev: Modifiers) -> bool {
    let now = grab_level(mods);
    let was = grab_level(prev);
    let edge = now && !was;
    tracing::trace!(?mods, ?prev, edge, "wd-input: grab edge");
    edge
}

/// Pointer-lock state machine (edge-triggered).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GrabState {
    /// Locked (raw deltas flow to aim).
    pub locked: bool,
    /// Modifier level seen on the previous poll (for edge detection).
    pub held: bool,
}

impl GrabState {
    /// Toggle on an Alt/Ctrl rising edge. Returns new state.
    ///
    /// Holding Alt across polls toggles exactly once; release + press again
    /// toggles back. `held` tracks the previous level.
    pub fn on_edge(&mut self, alt: bool, ctrl: bool) -> bool {
        let now = grab_edge(alt, ctrl);
        if now && !self.held {
            self.locked = !self.locked;
            tracing::info!(locked = self.locked, "wd-input: grab toggle");
        }
        self.held = now;
        self.locked
    }

    /// Toggle with an explicit previous state (stateless helper).
    pub fn on_state(&mut self, mods: Modifiers, prev: Modifiers) -> bool {
        if grab_rising_edge(mods, prev) {
            self.locked = !self.locked;
            tracing::info!(locked = self.locked, "wd-input: grab toggle");
        }
        self.held = grab_level(mods);
        self.locked
    }

    /// Force unlock (e.g. pause / F9 emergency release).
    pub fn force_unlock(&mut self) {
        self.locked = false;
        tracing::info!("wd-input: grab released");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_toggles_once_per_press() {
        let mut g = GrabState::default();
        assert!(g.on_edge(true, false));
        // Holding still pressed: no re-toggle.
        assert!(g.on_edge(true, false));
        assert!(g.locked);
        // Different modifier while still held: still held, no toggle.
        assert!(g.on_edge(true, true));
        assert!(g.locked);
        // Release then press again toggles off.
        assert!(g.locked);
        g.on_edge(false, false);
        assert!(!g.on_edge(false, true));
        assert!(!g.locked);
    }

    #[test]
    fn rising_edge_helper() {
        use super::Modifiers;
        assert!(grab_rising_edge(
            Modifiers::new(true, false),
            Modifiers::new(false, false)
        ));
        assert!(!grab_rising_edge(
            Modifiers::new(true, false),
            Modifiers::new(true, false)
        ));
        assert!(!grab_rising_edge(
            Modifiers::new(false, false),
            Modifiers::new(false, false)
        ));
        assert!(grab_edge(true, false));
    }

    #[test]
    fn force_unlock() {
        let mut g = GrabState {
            held: false,
            locked: true,
        };
        g.force_unlock();
        assert!(!g.locked);
    }
}
