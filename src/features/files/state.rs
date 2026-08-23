use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileItem {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub enum FileMsg {
    Start(PathBuf, u64),
    Done(PathBuf, u64, Vec<FileItem>),
}

#[derive(Debug)]
pub struct FilesState {
    pub current_dir: PathBuf,
    pub items: Vec<FileItem>,
    pub selected: usize,
    pub is_scanning: bool,
    pub scan_generation: u64,
}

impl FilesState {
    pub fn new() -> Self {
        let start_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        Self {
            current_dir: start_dir,
            items: Vec::new(),
            selected: 0,
            is_scanning: false,
            scan_generation: 0,
        }
    }

    pub fn begin_scan(&mut self) -> u64 {
        self.scan_generation = self.scan_generation.wrapping_add(1);
        self.is_scanning = true;
        self.items.clear();
        self.selected = 0;
        self.scan_generation
    }

    pub fn select_next(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + 1).min(self.items.len() - 1);
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn enter_selected(&self) -> Option<PathBuf> {
        self.items
            .get(self.selected)
            .filter(|item| item.is_dir)
            .map(|item| item.path.clone())
    }

    pub fn go_up(&self) -> Option<PathBuf> {
        self.current_dir.parent().map(PathBuf::from)
    }
}

impl Default for FilesState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{FileItem, FilesState};
    use std::path::PathBuf;

    #[test]
    fn scan_generation_changes_and_resets_view_state() {
        let mut state = FilesState::new();
        state.items.push(FileItem {
            name: "old".to_string(),
            is_dir: false,
            size: 1,
            path: PathBuf::from("old"),
        });
        state.selected = 1;

        let generation = state.begin_scan();

        assert_eq!(generation, state.scan_generation);
        assert!(state.is_scanning);
        assert!(state.items.is_empty());
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn selection_stays_within_item_bounds() {
        let mut state = FilesState::new();
        state.items = vec![
            FileItem {
                name: "a".to_string(),
                is_dir: false,
                size: 0,
                path: PathBuf::from("a"),
            },
            FileItem {
                name: "b".to_string(),
                is_dir: true,
                size: 0,
                path: PathBuf::from("b"),
            },
        ];

        state.select_next();
        state.select_next();
        assert_eq!(state.selected, 1);
        state.select_prev();
        state.select_prev();
        assert_eq!(state.selected, 0);
        assert_eq!(state.enter_selected(), None);
    }
}
