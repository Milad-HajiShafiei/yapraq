#[derive(Debug, Clone)]
pub struct PackageItem {
    pub name: String,
    pub version: String,
    pub manager: String,
}

#[derive(Debug, Clone)]
pub enum PkgsMsg {
    Start,
    Done(Vec<PackageItem>),
}

#[derive(Debug)]
pub struct PackagesState {
    pub items: Vec<PackageItem>,
    pub selected: usize,
    pub is_scanning: bool,
    pub uninstall_cmd: String,
}

impl PackagesState {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            is_scanning: false,
            uninstall_cmd: String::new(),
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

    pub fn generate_uninstall_cmd(&mut self) {
        if let Some(pkg) = self.items.get(self.selected) {
            self.uninstall_cmd = match pkg.manager.as_str() {
                "brew" => format!("brew uninstall {}", pkg.name),
                "apt" => format!("sudo apt remove {}", pkg.name),
                "pacman" => format!("sudo pacman -Rns {}", pkg.name),
                "winget" => format!("winget uninstall {}", pkg.name),
                _ => "Unknown package manager".to_string(),
            };
        } else {
            self.uninstall_cmd.clear();
        }
    }
}

impl Default for PackagesState {
    fn default() -> Self {
        Self::new()
    }
}
