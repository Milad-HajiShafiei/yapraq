use sysinfo::Disks;

#[derive(Debug)]
pub struct StorageData {
    pub disks: Disks,
}

impl StorageData {
    pub fn new() -> Self {
        Self {
            disks: Disks::new_with_refreshed_list(),
        }
    }

    pub fn update(&mut self) {
        self.disks.refresh(true);
    }
}
impl Default for StorageData {
    fn default() -> Self {
        Self::new()
    }
}

pub fn usage_percent(used: u64, total: u64) -> u16 {
    if total == 0 {
        0
    } else {
        ((used as f64 / total as f64) * 100.0)
            .round()
            .clamp(0.0, 100.0) as u16
    }
}

#[cfg(test)]
mod tests {
    use super::usage_percent;

    #[test]
    fn storage_percentage_handles_empty_and_full_volumes() {
        assert_eq!(usage_percent(0, 0), 0);
        assert_eq!(usage_percent(50, 100), 50);
        assert_eq!(usage_percent(120, 100), 100);
    }
}
