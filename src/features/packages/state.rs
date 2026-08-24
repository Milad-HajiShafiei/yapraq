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
                "macports" => format!("sudo port uninstall {}", pkg.name),
                "apt" => format!("sudo apt remove {}", pkg.name),
                "pacman" => format!("sudo pacman -Rns {}", pkg.name),
                "rpm" => format!("sudo dnf remove {}", pkg.name),
                "snap" => format!("sudo snap remove {}", pkg.name),
                "flatpak" => format!("flatpak uninstall {}", pkg.name),
                "nix" => format!("nix-env --uninstall {}", pkg.name),
                "winget" => format!("winget uninstall {}", pkg.name),
                "scoop" => format!("scoop uninstall {}", pkg.name),
                "choco" => format!("choco uninstall {}", pkg.name),
                "pip" => format!("pip uninstall {}", pkg.name),
                "uv" => format!("uv tool uninstall {}", pkg.name),
                "conda" => format!("conda remove {}", pkg.name),
                "npm" => format!("npm uninstall -g {}", pkg.name),
                "yarn" => format!("yarn global remove {}", pkg.name),
                "pnpm" => format!("pnpm remove -g {}", pkg.name),
                "cargo" => format!("cargo uninstall {}", pkg.name),
                "gem" => format!("gem uninstall {}", pkg.name),
                "go" => format!("go clean -modcache # {}", pkg.name),
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
