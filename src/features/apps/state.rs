use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppItem {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub enum AppsMsg {
    Start,
    Done(Vec<AppItem>),
}

#[derive(Debug)]
pub struct AppsState {
    pub items: Vec<AppItem>,
    pub selected: usize,
    pub is_scanning: bool,
}

impl AppsState {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            is_scanning: false,
        }
    }

    pub fn select_next(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + 1).min(self.items.len() - 1);
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn clamp_selection(&mut self) {
        self.selected = self.selected.min(self.items.len().saturating_sub(1));
    }
}

impl Default for AppsState {
    fn default() -> Self {
        Self::new()
    }
}
