//! Wait-ready poll helper. Pure logic, no spawn, no sleep.
//!
//! Ready = Wayland socket exists. Caller checks the path;
//! this module only decides. Ref: `.research/03-tech-stack.md:92-101`.

/// Default Wayland socket marking session ready.
pub const WAYLAND_SOCKET: &str = "/run/waydroid/wayland-0";

/// Default wait budget in seconds.
pub const DEFAULT_TIMEOUT_S: u64 = 60;

/// Poll gap in milliseconds. Cheap `Path::exists`, no busy loop.
pub const POLL_GAP_MS: u64 = 200;

/// Wait-ready descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaitReady {
    /// Socket path to poll.
    pub path: String,
    /// Budget in seconds.
    pub timeout_s: u64,
}

impl WaitReady {
    /// Default Wayland socket wait.
    #[must_use]
    pub fn wayland() -> Self {
        tracing::info!(path = WAYLAND_SOCKET, "ready: wait descriptor");
        Self {
            path: WAYLAND_SOCKET.to_owned(),
            timeout_s: DEFAULT_TIMEOUT_S,
        }
    }
}

/// Poll outcome for one tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitDecision {
    /// Socket present, stop polling.
    Ready,
    /// Keep polling.
    KeepWaiting,
    /// Budget spent, stop polling.
    TimedOut,
}

/// Decide one poll tick. Pure: pass `Path::exists` result in.
#[must_use]
pub fn decide(socket_exists: bool, elapsed_s: u64, timeout_s: u64) -> WaitDecision {
    tracing::debug!(socket_exists, elapsed_s, timeout_s, "ready: poll tick");
    if socket_exists {
        tracing::info!(elapsed_s, "ready: socket up");
        WaitDecision::Ready
    } else if elapsed_s >= timeout_s {
        tracing::warn!(elapsed_s, timeout_s, "ready: timed out");
        WaitDecision::TimedOut
    } else {
        WaitDecision::KeepWaiting
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticks() {
        assert_eq!(decide(true, 0, 60), WaitDecision::Ready);
        assert_eq!(decide(false, 5, 60), WaitDecision::KeepWaiting);
        assert_eq!(decide(false, 60, 60), WaitDecision::TimedOut);
        assert_eq!(decide(false, 99, 60), WaitDecision::TimedOut);
        assert_eq!(WaitReady::wayland().timeout_s, DEFAULT_TIMEOUT_S);
    }
}
