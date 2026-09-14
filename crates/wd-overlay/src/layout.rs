//! Layout v1.3 doc: save/load + scale-on-load.
//!
//! Shape COPY `widget_layout_service.py:210`
//! `{version,screen_resolution,widgets[{type,x,y,w,h,text,keys,config}]}`.
//! Stored px at save screen; rects relative 0-1 in the editor (`QtScrcpy` rule).
#![deny(missing_docs)]

use crate::widgets::{WidgetKind, WidgetRect};

/// Layout doc version.
pub const LAYOUT_VERSION: &str = "1.3";

/// One saved widget row (px at `screen_resolution`).
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutRow {
    /// Palette index into [`WidgetKind::palette`].
    pub kind: WidgetKind,
    /// X px at save screen.
    pub x: f32,
    /// Y px at save screen.
    pub y: f32,
    /// W px at save screen.
    pub w: f32,
    /// H px at save screen.
    pub h: f32,
    /// Label text.
    pub text: String,
    /// Bound keys.
    pub keys: String,
}

/// Saved layout doc.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OverlayLayout {
    /// Save screen.
    pub screen: (u32, u32),
    /// Rows.
    pub rows: Vec<(String, LayoutRow)>,
}

impl OverlayLayout {
    /// Scale-on-load: px at saved screen → relative rects.
    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn to_rects(&self) -> Vec<WidgetRect> {
        tracing::info!(rows = self.rows.len(), "wd-overlay: layout load");
        let (sw, sh) = (self.screen.0.max(1) as f32, self.screen.1.max(1) as f32);
        self.rows
            .iter()
            .map(|(id, r)| WidgetRect {
                id: id.clone(),
                kind: r.kind,
                x: (r.x / sw).clamp(0.0, 1.0),
                y: (r.y / sh).clamp(0.0, 1.0),
                w: (r.w / sw).clamp(0.0, 1.0),
                h: (r.h / sh).clamp(0.0, 1.0),
                key: r.keys.clone(),
            })
            .collect()
    }

    /// Build a doc from live rects at `screen` (px = rel * screen).
    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn from_rects(rects: &[WidgetRect], screen: (u32, u32)) -> Self {
        tracing::info!(rows = rects.len(), "wd-overlay: layout save");
        let rows = rects
            .iter()
            .map(|r| {
                let row = LayoutRow {
                    kind: r.kind,
                    x: r.x * screen.0 as f32,
                    y: r.y * screen.1 as f32,
                    w: r.w * screen.0 as f32,
                    h: r.h * screen.1 as f32,
                    text: r.id.clone(),
                    keys: r.key.clone(),
                };
                (r.id.clone(), row)
            })
            .collect();
        Self { screen, rows }
    }
}
