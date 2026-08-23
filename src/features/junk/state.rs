use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct JunkItem {
    pub path: PathBuf,
    pub size: u64,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub enum JunkMsg {
    Start,
    Done(Vec<JunkItem>),
}

#[derive(Debug)]
pub struct JunkState {
    pub items: Vec<JunkItem>,
    pub selected: usize,
    pub is_scanning: bool,
    pub total_size: u64,
}

impl JunkState {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            is_scanning: false,
            total_size: 0,
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

    pub fn delete_selected(&mut self) -> bool {
        let Some(item) = self.items.get(self.selected) else {
            return false;
        };
        let path = item.path.clone();
        let size = item.size;
        let success = if path.is_dir() {
            std::fs::remove_dir_all(&path).is_ok()
        } else {
            std::fs::remove_file(&path).is_ok()
        };

        if !success {
            return false;
        }

        self.items.remove(self.selected);
        self.total_size = self.total_size.saturating_sub(size);
        self.clamp_selection();
        true
    }

    pub fn delete_all(&mut self) -> usize {
        let mut deleted_count = 0;
        let mut total_freed = 0_u64;

        self.items.retain(|item| {
            let success = if item.path.is_dir() {
                std::fs::remove_dir_all(&item.path).is_ok()
            } else {
                std::fs::remove_file(&item.path).is_ok()
            };

            if success {
                deleted_count += 1;
                total_freed = total_freed.saturating_add(item.size);
                false
            } else {
                true
            }
        });

        self.total_size = self.total_size.saturating_sub(total_freed);
        self.clamp_selection();
        deleted_count
    }
}

impl Default for JunkState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{JunkItem, JunkState};
    use std::path::PathBuf;

    #[test]
    fn deleting_missing_item_is_non_destructive() {
        let mut state = JunkState {
            items: vec![JunkItem {
                path: PathBuf::from("this-path-does-not-exist"),
                size: 10,
                reason: "test".to_string(),
            }],
            selected: 0,
            is_scanning: false,
            total_size: 10,
        };

        assert!(!state.delete_selected());
        assert_eq!(state.items.len(), 1);
        assert_eq!(state.total_size, 10);
    }
}
