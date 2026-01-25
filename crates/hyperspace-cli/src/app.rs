use hyperspace_proto::hyperspace::SystemStats;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentTab {
    Overview,
    Storage,
    Admin,
}

impl CurrentTab {
    pub fn next(&self) -> Self {
        match self {
            Self::Overview => Self::Storage,
            Self::Storage => Self::Admin,
            Self::Admin => Self::Overview,
        }
    }
}

pub struct App {
    pub current_tab: CurrentTab,
    pub should_quit: bool,
    pub stats: SystemStats,
    pub logs: Vec<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            current_tab: CurrentTab::Overview,
            should_quit: false,
            stats: SystemStats::default(),
            logs: vec!["Ready. Waiting for connection...".to_string()],
        }
    }

    pub fn next_tab(&mut self) {
        self.current_tab = self.current_tab.next();
    }
}
