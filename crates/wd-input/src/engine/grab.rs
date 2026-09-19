//! Pointer grab state: lock edge toggled by Alt/Ctrl rising edge.
#![deny(missing_docs)]

/// Level check: true while Alt or Ctrl is held (not an edge).
///
/// Kept for backward compatibility. Prefer [`grab_rising_edge`] for toggle
/// decisions so holding the modifier does not re-fire every poll.
#[must_use]
pub fn grab_edge(alt: bool, ctrl: bool) -> bool {
    tracing::trace!(alt, ctrl, "wd-input: grab level");
    alt || ctrl
}

/// Rising-edge check: true only on the press transition.
///
/// `prev_alt`/`prev_ctrl` is the modifier state from the previous poll.
/// Holding Alt across polls fires exactly once.
#[must_use]
pub fn grab_rising_edge(alt: bool, ctrl: bool, prev_alt: bool, prev_ctrl: bool) -> bool {
    let now = grab_edge(alt, ctrl);
    let was = grab_edge(prev_alt, prev_ctrl);
    let edge = now && !was;
    tracing::trace!(alt, ctrl, prev_alt, prev_ctrl, edge, "wd-input: grab edge");
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
    pub fn on_state(&mut self, alt: bool, ctrl: bool, prev_alt: bool, prev_ctrl: bool) -> bool {
        if grab_rising_edge(alt, ctrl, prev_alt, prev_ctrl) {
            self.locked = !self.locked;
            tracing::info!(locked = self.locked, "wd-input: grab toggle");
        }
        self.held = grab_edge(alt, ctrl);
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
        assert!(grab_rising_edge(true, false, false, false));
        assert!(!grab_rising_edge(true, false, true, false));
        assert!(!grab_rising_edge(false, false, false, false));
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
