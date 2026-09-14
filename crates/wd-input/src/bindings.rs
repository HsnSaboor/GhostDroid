//! `Node` audit helpers: bound keys, positions.
#![deny(missing_docs)]

use wd_core::RelPos;

use crate::nodes::Node;

impl Node {
    /// Bound keys for dup-key audit.
    #[must_use]
    pub fn bound_keys(&self) -> Vec<String> {
        match self {
            Self::Tap(t) | Self::ToggleTap(t) => vec![t.key.clone()],
            Self::Drag(d) => vec![d.key.clone()],
            Self::RepeatTap(r) => vec![r.key.clone()],
            Self::Macro(m) => vec![m.key.clone()],
            Self::LayerShift(s) => vec![s.key.clone()],
            Self::Joystick(j) => {
                vec![
                    j.keys.up.clone(),
                    j.keys.down.clone(),
                    j.keys.left.clone(),
                    j.keys.right.clone(),
                ]
            }
            Self::Wheel(_) => vec!["WheelUp".into(), "WheelDown".into()],
            Self::Aim(a) => {
                let mut out = vec!["MouseMove".into()];
                if let Some(name) = &a.activation_key {
                    out.push(name.clone());
                }
                out
            }
        }
    }

    /// All rel positions for off-screen audit.
    #[must_use]
    pub fn positions(&self) -> Vec<RelPos> {
        match self {
            Self::Tap(t) | Self::ToggleTap(t) => vec![t.pos],
            Self::RepeatTap(r) => vec![r.pos],
            Self::Joystick(j) => vec![j.pos],
            Self::Drag(d) => vec![d.start, d.end],
            Self::Aim(a) => vec![a.anchor],
            Self::Wheel(w) => vec![w.up_pos, w.down_pos],
            Self::Macro(m) => m.sequence.iter().filter_map(|s| s.pos).collect(),
            Self::LayerShift(_) => vec![],
        }
    }
}
