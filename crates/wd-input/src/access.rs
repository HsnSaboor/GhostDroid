//! `Node` accessors: id, slots, layer.
#![deny(missing_docs)]

use crate::nodes::Node;

impl Node {
    /// Node id.
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::Tap(t) | Self::ToggleTap(t) => &t.id,
            Self::Joystick(j) => &j.id,
            Self::Drag(d) => &d.id,
            Self::Aim(a) => &a.id,
            Self::RepeatTap(r) => &r.id,
            Self::Wheel(w) => &w.id,
            Self::Macro(m) => &m.id,
            Self::LayerShift(s) => &s.id,
        }
    }

    /// Primary slot (wheel/macro/layershift special).
    #[must_use]
    pub const fn slot(&self) -> Option<u8> {
        match self {
            Self::Tap(t) | Self::ToggleTap(t) => Some(t.slot),
            Self::Joystick(j) => Some(j.slot),
            Self::Drag(d) => Some(d.slot),
            Self::Aim(a) => Some(a.slot),
            Self::RepeatTap(r) => Some(r.slot),
            Self::Wheel(_) | Self::Macro(_) | Self::LayerShift(_) => None,
        }
    }

    /// All slots touched by node.
    #[must_use]
    pub fn slots(&self) -> Vec<u8> {
        match self {
            Self::Wheel(w) => vec![w.up_slot, w.down_slot],
            Self::Macro(m) => m.sequence.iter().map(|s| s.slot).collect(),
            Self::LayerShift(_) => vec![],
            Self::Tap(t) | Self::ToggleTap(t) => vec![t.slot],
            Self::Joystick(j) => vec![j.slot],
            Self::Drag(d) => vec![d.slot],
            Self::Aim(a) => vec![a.slot],
            Self::RepeatTap(r) => vec![r.slot],
        }
    }

    /// Layer ("" = base).
    #[must_use]
    pub fn layer(&self) -> &str {
        match self {
            Self::Tap(t) | Self::ToggleTap(t) => &t.layer,
            Self::Joystick(j) => &j.layer,
            Self::Drag(d) => &d.layer,
            Self::Aim(a) => &a.layer,
            Self::RepeatTap(r) => &r.layer,
            Self::Wheel(w) => &w.layer,
            Self::Macro(m) => &m.layer,
            Self::LayerShift(_) => "",
        }
    }

    /// Short kind label.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Tap(_) => "tap",
            Self::ToggleTap(_) => "toggle_tap",
            Self::Joystick(_) => "joystick",
            Self::Drag(_) => "drag",
            Self::Aim(_) => "aim",
            Self::RepeatTap(_) => "repeat_tap",
            Self::Wheel(_) => "wheel",
            Self::Macro(_) => "macro",
            Self::LayerShift(_) => "layer_shift",
        }
    }
}
