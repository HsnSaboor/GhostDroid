//! OSD state: pause / FPS / save-state toast (plan 07).
//!
//! Headless Mapping mode shows minimal indicators (no GTK overhead).
#![deny(missing_docs)]

/// OSD toast lifetime frames (renderer consumes; logic stores only).
pub const TOAST_FRAMES: u32 = 120;

/// On-screen display state. Drawn by egui, owned here.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OsdState {
    /// Paused (F9).
    pub paused: bool,
    /// Last FPS sample.
    pub fps: f32,
    /// Active toast text.
    pub toast: String,
    /// Toast frames left.
    pub toast_left: u32,
    /// Mapping-mode minimal indicators on.
    pub minimal: bool,
}

impl OsdState {
    /// Set pause flag (F9). Logs each change.
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
        tracing::info!(paused, "wd-overlay: osd pause");
    }

    /// Push an FPS sample.
    pub fn set_fps(&mut self, fps: f32) {
        self.fps = fps.max(0.0);
        tracing::debug!(fps = self.fps, "wd-overlay: osd fps");
    }

    /// Show a toast (`pause/FPS/save-state`).
    pub fn toast(&mut self, msg: &str) {
        self.toast = msg.to_string();
        self.toast_left = TOAST_FRAMES;
        tracing::info!(msg, "wd-overlay: osd toast");
    }

    /// Tick one frame; expires the toast.
    pub fn tick(&mut self) {
        if self.toast_left > 0 {
            self.toast_left -= 1;
            if self.toast_left == 0 {
                self.toast.clear();
                tracing::debug!("wd-overlay: osd toast expired");
            }
        }
    }

    /// Mapping mode shows minimal indicators only.
    pub fn set_minimal(&mut self, minimal: bool) {
        self.minimal = minimal;
        tracing::info!(minimal, "wd-overlay: osd minimal");
    }
}
