//! Pure app state for the GPUI shell. No Waydroid/process logic here.

use wd_shell::{DeviceState, GameRow, KeymapState, LogStream, SpoofProfile};

/// Top-level pages, mirroring the Slint nav rail order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    /// Game library grid.
    Library,
    /// Device state + scan.
    Devices,
    /// Spoof profile picker.
    Spoof,
    /// Keymap editor.
    Keys,
    /// Daemon log stream.
    Logs,
}

impl Page {
    /// All pages in nav order.
    pub const ALL: [Self; 5] = [
        Self::Library,
        Self::Devices,
        Self::Spoof,
        Self::Keys,
        Self::Logs,
    ];

    /// Nav label, matching the Slint rail text.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Library => "Library",
            Self::Devices => "Devices",
            Self::Spoof => "Spoof",
            Self::Keys => "Keys",
            Self::Logs => "Logs",
        }
    }
}

/// View-owned app state. Page agents read fields; shell mutates via methods.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Active page.
    pub page: Page,
    /// All game rows.
    pub games: Vec<GameRow>,
    /// Search text.
    pub query: String,
    /// Active filter index (mirrors `wd_shell::GameList::filter_index`).
    pub filter_index: i32,
    /// Device snapshot.
    pub device: DeviceState,
    /// Scan in flight.
    pub busy: bool,
    /// Spoof snapshot.
    pub spoof: SpoofProfile,
    /// Keymap snapshot.
    pub keymap: KeymapState,
    /// Log tail.
    pub logs: LogStream,
}

impl Default for AppState {
    fn default() -> Self {
        tracing::debug!("app state default (seeded demo fallback)");
        let mut logs = LogStream::default();
        logs.push("boot: ghostdroid 1.0 ready".to_owned(), 500);
        logs.push("spoof: gaming-full profile loaded".to_owned(), 500);
        logs.push("keymap: pubg.json validated".to_owned(), 500);
        Self {
            page: Page::Library,
            games: vec![
                GameRow {
                    title: "Clash Royale".to_owned(),
                    pkg: "com.supercell.clashroyale".to_owned(),
                    genre: "Strategy".to_owned(),
                    state: "ready".to_owned(),
                },
                GameRow {
                    title: "PUBG Mobile".to_owned(),
                    pkg: "com.tencent.ig".to_owned(),
                    genre: "FPS".to_owned(),
                    state: "ready".to_owned(),
                },
                GameRow {
                    title: "Genshin Impact".to_owned(),
                    pkg: "com.miHoYo.GenshinImpact".to_owned(),
                    genre: "RPG".to_owned(),
                    state: "ready".to_owned(),
                },
                GameRow {
                    title: "Roblox".to_owned(),
                    pkg: "com.roblox.client".to_owned(),
                    genre: "Sandbox".to_owned(),
                    state: "update".to_owned(),
                },
            ],
            query: String::new(),
            filter_index: 0,
            device: DeviceState {
                ready: false,
                frozen: false,
            },
            busy: false,
            spoof: SpoofProfile {
                id: "gaming-full".to_owned(),
                ids: vec!["gaming-full".to_owned(), "gaming-basic".to_owned()],
                selected: 0,
                props: vec![
                    ("model".to_owned(), "Pixel 7 Pro (cheetah)".to_owned()),
                    ("stack".to_owned(), "full".to_owned()),
                ],
            },
            keymap: KeymapState {
                profile: "pubg".to_owned(),
                fire_key: "MouseLeft".to_owned(),
                tab_index: 0,
            },
            logs,
        }
    }
}

impl AppState {
    /// Switch page.
    pub fn set_page(&mut self, page: Page) {
        tracing::debug!(page = ?page, "nav");
        self.page = page;
    }

    /// Replace the game list.
    pub fn set_games(&mut self, games: Vec<GameRow>) {
        tracing::debug!(count = games.len(), "games set");
        self.games = games;
    }

    /// Update the search query.
    pub fn set_query(&mut self, query: String) {
        tracing::debug!(query = %query, "query set");
        self.query = query;
    }

    /// Update the active filter index.
    pub fn set_filter(&mut self, index: i32) {
        tracing::debug!(index, "filter set");
        self.filter_index = index;
    }

    /// Mark the device busy/idle (scan progress).
    pub fn set_busy(&mut self, busy: bool) {
        tracing::debug!(busy, "busy set");
        self.busy = busy;
    }

    /// Push a daemon log line, keeping the tail capped.
    pub fn push_log(&mut self, line: String) {
        tracing::debug!(line = %line, "log push");
        self.logs.push(line, 500);
    }

    /// Clear the log stream.
    pub fn clear_logs(&mut self) {
        tracing::info!("logs cleared");
        self.logs.clear();
    }

    /// Games matching the current query (title or package, case-insensitive).
    #[must_use]
    pub fn visible_games(&self) -> Vec<&GameRow> {
        let q = self.query.to_lowercase();
        tracing::debug!(query = %self.query, total = self.games.len(), "filter games");
        self.games
            .iter()
            .filter(|g| {
                q.is_empty()
                    || g.title.to_lowercase().contains(&q)
                    || g.pkg.to_lowercase().contains(&q)
            })
            .collect()
    }
}
