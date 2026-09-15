//! wd-shell: viewmodels only. Zero Waydroid logic. Slint reads these.
#![deny(missing_docs)]

/// One row in the launcher grid. Mirrors `GameRow` in `ui/ui-shared.slint`.
#[derive(Debug, Clone)]
pub struct GameRow {
    /// Display title.
    pub title: String,
    /// Android package id.
    pub pkg: String,
    /// Genre chip text.
    pub genre: String,
    /// State chip text (e.g. ready, running).
    pub state: String,
}

/// Game list viewmodel (grid source).
#[derive(Debug, Clone, Default)]
pub struct GameList {
    /// All rows; shell filters by search text locally.
    pub rows: Vec<GameRow>,
    /// Current search text (mirrors shell `search-text`).
    pub search_text: String,
    /// Active filter index (mirrors `ButtonGroup.current-index`).
    pub filter_index: i32,
}

/// Device state for `DeviceDot` (`ui/ui-shared.slint`).
#[derive(Debug, Clone)]
pub struct DeviceState {
    /// Waydroid session up.
    pub ready: bool,
    /// Session frozen (`waydroid session stop` keeps container).
    pub frozen: bool,
}

/// Spoof profile viewmodel (page 2 picker + prop rows).
#[derive(Debug, Clone)]
pub struct SpoofProfile {
    /// Profile id (e.g. `cheetah`).
    pub id: String,
    /// All known profile ids for the dropdown.
    pub ids: Vec<String>,
    /// Selected index into `ids`.
    pub selected: i32,
    /// Key/value prop rows shown under the picker.
    pub props: Vec<(String, String)>,
}

/// Keymap state (page 3 editor summary; canvas lives in overlay, plan 07).
#[derive(Debug, Clone)]
pub struct KeymapState {
    /// Profile name (e.g. `pubg`).
    pub profile: String,
    /// Fire key label.
    pub fire_key: String,
    /// Active editor tab index (Map/Aim/DPad).
    pub tab_index: i32,
    /// Node count from `profiles/keymap/<profile>.json` (pubg = 21).
    pub node_count: usize,
}

/// Log stream viewmodel (page 4; daemon pushes lines, shell keeps tail).
#[derive(Debug, Clone, Default)]
pub struct LogStream {
    /// Visible tail of daemon log lines.
    pub lines: Vec<String>,
}

impl LogStream {
    /// Push a line, keeping at most `cap` lines.
    pub fn push(&mut self, line: String, cap: usize) {
        self.lines.push(line);
        if self.lines.len() > cap {
            let drop = self.lines.len() - cap;
            self.lines.drain(..drop);
        }
    }

    /// Clear all lines.
    pub fn clear(&mut self) {
        self.lines.clear();
    }
}
