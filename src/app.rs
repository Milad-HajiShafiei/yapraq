use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::events::Event;
use crate::features::apps::{AppsMsg, AppsState};
use crate::features::devices::DevicesState;
use crate::features::files::{FileMsg, FilesState};
use crate::features::info::InfoState;
use crate::features::junk::{JunkMsg, JunkState};
use crate::features::monitor::MonitorData;
use crate::features::packages::{PackagesState, PkgsMsg};
use crate::features::storage::StorageData;
use crate::utils::{is_path_safe, sanitize_error_message, sanitize_filename};

/// The different main views/modules of the application.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum Tab {
    #[default]
    Monitor,
    Apps,
    Packages,
    Files,
    Storage,
    Junk,
    Devices,
    Info,
}

impl Tab {
    pub const ALL: [Self; 8] = [
        Self::Monitor,
        Self::Apps,
        Self::Packages,
        Self::Files,
        Self::Storage,
        Self::Junk,
        Self::Devices,
        Self::Info,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Monitor => "Dashboard",
            Self::Apps => "Applications",
            Self::Packages => "Packages",
            Self::Files => "Files",
            Self::Storage => "Storage",
            Self::Junk => "Junk Cleaner",
            Self::Devices => "Devices",
            Self::Info => "System Info",
        }
    }

    fn adjacent(self, delta: i32) -> Self {
        let index = Self::ALL.iter().position(|tab| *tab == self).unwrap_or(0) as i32;
        let next = (index + delta).rem_euclid(Self::ALL.len() as i32) as usize;
        Self::ALL[next]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileDialog {
    NewFile,
    NewFolder,
    Rename,
}

impl FileDialog {
    pub fn title(self) -> &'static str {
        match self {
            Self::NewFile => "Create file",
            Self::NewFolder => "Create folder",
            Self::Rename => "Rename item",
        }
    }

    pub fn prompt(self) -> &'static str {
        match self {
            Self::NewFile => "File name",
            Self::NewFolder => "Folder name",
            Self::Rename => "New name",
        }
    }
}

/// Sections within the settings modal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsSection {
    #[default]
    Theme,
    About,
}

impl SettingsSection {
    pub const ALL: [Self; 2] = [Self::Theme, Self::About];

    pub fn label(self) -> &'static str {
        match self {
            Self::Theme => "Theme",
            Self::About => "About",
        }
    }

    fn adjacent(self, delta: i32) -> Self {
        let idx = Self::ALL.iter().position(|&s| s == self).unwrap_or(0) as i32;
        let next = (idx + delta).rem_euclid(Self::ALL.len() as i32) as usize;
        Self::ALL[next]
    }
}

/// High-level Actions that the application can perform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Quit,
    Tick,
    SwitchTab(Tab),
    PreviousTab,
    NextTab,
    ToggleHelp,
    ToggleSettings,
    SettingsNextSection,
    SettingsPrevSection,
    SettingsNextItem,
    SettingsPrevItem,
    SettingsSelect,
    Navigate(PathBuf),
    OpenSelected,
    GoUp,
    SelectNext,
    SelectPrev,
    RefreshDevices,
    ScanJunk,
    DeleteJunk,
    DeleteAllJunk,
    ScanApps,
    ScanPackages,
    ShowUninstallCmd,
    StartNewFile,
    StartNewFolder,
    StartRename,
    StartDelete,
    RefreshFiles,
    PromptInput(char),
    PromptBackspace,
    SubmitPrompt,
    ConfirmDelete,
    CancelOverlay,
}

/// The global application state.
#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub current_tab: Tab,
    pub monitor: MonitorData,
    pub storage: StorageData,
    pub files: FilesState,
    pub devices: DevicesState,
    pub info: InfoState,
    pub junk: JunkState,
    pub apps: AppsState,
    pub packages: PackagesState,
    pub show_help: bool,
    pub show_settings: bool,
    pub settings_section: SettingsSection,
    pub settings_selected: usize,
    pub file_dialog: Option<FileDialog>,
    pub dialog_input: String,
    pub dialog_target: Option<PathBuf>,
    pub delete_confirmation: Option<PathBuf>,
    pub status_message: String,
    pub files_rescan_requested: bool,
    pub scroll_offset: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            current_tab: Tab::default(),
            monitor: MonitorData::new(),
            storage: StorageData::new(),
            files: FilesState::new(),
            devices: DevicesState::new(),
            info: InfoState::new(),
            junk: JunkState::new(),
            apps: AppsState::new(),
            packages: PackagesState::new(),
            show_help: false,
            show_settings: false,
            settings_section: SettingsSection::Theme,
            settings_selected: 0,
            file_dialog: None,
            dialog_input: String::new(),
            dialog_target: None,
            delete_confirmation: None,
            status_message: "Ready".to_string(),
            files_rescan_requested: false,
            scroll_offset: 0,
        }
    }

    pub fn handle_event(&mut self, event: Event) -> Option<Action> {
        match event {
            Event::Tick => Some(Action::Tick),
            Event::Key(key) => self.handle_key_event(key),
            Event::Resize => Some(Action::Tick),
            Event::Error => Some(Action::Quit),
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Action> {
        if key.kind != KeyEventKind::Press {
            return None;
        }

        if self.show_settings {
            return match key.code {
                KeyCode::Esc => Some(Action::CancelOverlay),
                KeyCode::Up | KeyCode::Char('h') => Some(Action::SettingsPrevSection),
                KeyCode::Down | KeyCode::Char('l') => Some(Action::SettingsNextSection),
                KeyCode::Char('j') => Some(Action::SettingsNextItem),
                KeyCode::Char('k') => Some(Action::SettingsPrevItem),
                KeyCode::Enter | KeyCode::Char(' ') => Some(Action::SettingsSelect),
                _ => None,
            };
        }

        if self.show_help {
            return match key.code {
                KeyCode::Esc | KeyCode::Char('?') => Some(Action::ToggleHelp),
                _ => None,
            };
        }

        if self.file_dialog.is_some() {
            return match key.code {
                KeyCode::Esc => Some(Action::CancelOverlay),
                KeyCode::Enter => Some(Action::SubmitPrompt),
                KeyCode::Backspace => Some(Action::PromptBackspace),
                KeyCode::Char(character) => Some(Action::PromptInput(character)),
                _ => None,
            };
        }

        if self.delete_confirmation.is_some() {
            return match key.code {
                KeyCode::Char('y') | KeyCode::Enter => Some(Action::ConfirmDelete),
                KeyCode::Char('n') | KeyCode::Esc => Some(Action::CancelOverlay),
                _ => None,
            };
        }

        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::Quit)
            }
            KeyCode::Char('q') => Some(Action::Quit),
            KeyCode::Char('?') => Some(Action::ToggleHelp),
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Action::ToggleSettings)
            }
            KeyCode::Up => Some(Action::PreviousTab),
            KeyCode::Down => Some(Action::NextTab),

            KeyCode::Char('j') => Some(Action::SelectNext),
            KeyCode::Char('k') => Some(Action::SelectPrev),
            KeyCode::Char('r') if self.current_tab == Tab::Devices => Some(Action::RefreshDevices),
            KeyCode::Char('r') if self.current_tab == Tab::Files => Some(Action::RefreshFiles),
            KeyCode::Char('d') if self.current_tab == Tab::Junk => Some(Action::DeleteJunk),
            KeyCode::Char('D') if self.current_tab == Tab::Junk => Some(Action::DeleteAllJunk),
            KeyCode::Char('i') if self.current_tab == Tab::Packages => {
                Some(Action::ShowUninstallCmd)
            }
            KeyCode::Char('s') if self.current_tab == Tab::Apps && !self.apps.is_scanning => {
                Some(Action::ScanApps)
            }
            KeyCode::Char('s')
                if self.current_tab == Tab::Packages && !self.packages.is_scanning =>
            {
                Some(Action::ScanPackages)
            }
            KeyCode::Char('s') if self.current_tab == Tab::Junk && !self.junk.is_scanning => {
                Some(Action::ScanJunk)
            }
            KeyCode::Char('n') if self.current_tab == Tab::Files => Some(Action::StartNewFolder),
            KeyCode::Char('f') if self.current_tab == Tab::Files => Some(Action::StartNewFile),
            KeyCode::Char('e') if self.current_tab == Tab::Files => Some(Action::StartRename),
            KeyCode::Char('x') if self.current_tab == Tab::Files => Some(Action::StartDelete),
            KeyCode::Enter if self.current_tab == Tab::Files => {
                if self
                    .files
                    .items
                    .get(self.files.selected)
                    .is_some_and(|item| item.is_dir)
                {
                    self.files.enter_selected().map(Action::Navigate)
                } else {
                    Some(Action::OpenSelected)
                }
            }
            KeyCode::Backspace | KeyCode::Left if self.current_tab == Tab::Files => {
                Some(Action::GoUp)
            }
            KeyCode::Char('1') | KeyCode::F(1) => Some(Action::SwitchTab(Tab::Monitor)),
            KeyCode::Char('2') | KeyCode::F(2) => Some(Action::SwitchTab(Tab::Apps)),
            KeyCode::Char('3') | KeyCode::F(3) => Some(Action::SwitchTab(Tab::Packages)),
            KeyCode::Char('4') | KeyCode::F(4) => Some(Action::SwitchTab(Tab::Files)),
            KeyCode::Char('5') | KeyCode::F(5) => Some(Action::SwitchTab(Tab::Storage)),
            KeyCode::Char('6') | KeyCode::F(6) => Some(Action::SwitchTab(Tab::Junk)),
            KeyCode::Char('7') | KeyCode::F(7) => Some(Action::SwitchTab(Tab::Devices)),
            KeyCode::Char('8') | KeyCode::F(8) => Some(Action::SwitchTab(Tab::Info)),
            _ => None,
        }
    }

    pub fn update(&mut self, action: Action) {
        match action {
            Action::Quit => self.running = false,
            Action::SwitchTab(tab) => {
                self.current_tab = tab;
                self.scroll_offset = 0;
                match tab {
                    Tab::Apps if self.apps.items.is_empty() && !self.apps.is_scanning => {
                        self.apps.is_scanning = true;
                        self.apps.items.clear();
                        self.apps.selected = 0;
                        self.status_message = "Scanning installed applications".to_string();
                    }
                    Tab::Packages
                        if self.packages.items.is_empty() && !self.packages.is_scanning =>
                    {
                        self.packages.is_scanning = true;
                        self.packages.items.clear();
                        self.packages.selected = 0;
                        self.packages.uninstall_cmd.clear();
                        self.status_message = "Scanning installed packages".to_string();
                    }
                    Tab::Junk if self.junk.items.is_empty() && !self.junk.is_scanning => {
                        self.junk.is_scanning = true;
                        self.junk.items.clear();
                        self.junk.selected = 0;
                        self.junk.total_size = 0;
                        self.status_message = "Scanning for reclaimable files".to_string();
                    }
                    _ => {}
                }
            }
            Action::PreviousTab => {
                self.current_tab = self.current_tab.adjacent(-1);
                self.scroll_offset = 0;
            }
            Action::NextTab => {
                self.current_tab = self.current_tab.adjacent(1);
                self.scroll_offset = 0;
            }
            Action::ToggleHelp => self.show_help = !self.show_help,
            Action::ToggleSettings => {
                self.show_settings = !self.show_settings;
                self.settings_section = SettingsSection::Theme;
                self.settings_selected = 0;
            }
            Action::Tick => {
                self.monitor.update();
                if self.current_tab == Tab::Storage {
                    self.storage.update();
                }
                // Auto-scroll: Monitor follows bottom (latest entries), Info shows from top
                match self.current_tab {
                    Tab::Monitor => self.scroll_offset = usize::MAX,
                    Tab::Info => self.scroll_offset = 0,
                    _ => {}
                }
            }
            Action::SelectNext => match self.current_tab {
                Tab::Apps => self.apps.select_next(),
                Tab::Packages => self.packages.select_next(),
                Tab::Files => self.files.select_next(),
                Tab::Junk => self.junk.select_next(),
                Tab::Devices => self.devices.select_next(),
                _ => {}
            },
            Action::SelectPrev => match self.current_tab {
                Tab::Apps => self.apps.select_prev(),
                Tab::Packages => self.packages.select_prev(),
                Tab::Files => self.files.select_prev(),
                Tab::Junk => self.junk.select_prev(),
                Tab::Devices => self.devices.select_prev(),
                _ => {}
            },
            Action::Navigate(path) => {
                self.files.current_dir = path;
                self.files.selected = 0;
                self.status_message = "Opening folder".to_string();
            }
            Action::OpenSelected => self.open_selected_file(),
            Action::GoUp => {
                if let Some(parent) = self.files.go_up() {
                    self.files.current_dir = parent;
                    self.files.selected = 0;
                    self.status_message = "Moved to parent folder".to_string();
                }
            }
            Action::RefreshDevices => {
                self.devices.refresh();
                self.status_message = "USB device list refreshed".to_string();
            }
            Action::RefreshFiles => {
                self.files_rescan_requested = true;
                self.status_message = "Refreshing folder".to_string();
            }
            Action::ScanJunk => {
                self.junk.is_scanning = true;
                self.junk.items.clear();
                self.junk.selected = 0;
                self.junk.total_size = 0;
                self.status_message = "Scanning for reclaimable files".to_string();
            }
            Action::ScanApps => {
                self.apps.is_scanning = true;
                self.apps.items.clear();
                self.apps.selected = 0;
                self.status_message = "Scanning installed applications".to_string();
            }
            Action::ScanPackages => {
                self.packages.is_scanning = true;
                self.packages.items.clear();
                self.packages.selected = 0;
                self.packages.uninstall_cmd.clear();
                self.status_message = "Scanning installed packages".to_string();
            }
            Action::DeleteJunk => {
                self.junk.delete_selected();
                self.status_message = "Selected junk item removed when permitted".to_string();
            }
            Action::DeleteAllJunk => {
                self.junk.delete_all();
                self.status_message = "Junk cleanup completed".to_string();
            }
            Action::ShowUninstallCmd => self.packages.generate_uninstall_cmd(),
            Action::StartNewFile => self.open_file_dialog(FileDialog::NewFile, None),
            Action::StartNewFolder => self.open_file_dialog(FileDialog::NewFolder, None),
            Action::StartRename => {
                let target = self.selected_file_path();
                if let Some(target) = target {
                    let name = target
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned());
                    self.open_file_dialog(FileDialog::Rename, Some(target));
                    if let Some(name) = name {
                        self.dialog_input = name;
                    }
                } else {
                    self.status_message = "Select a file or folder to rename".to_string();
                }
            }
            Action::StartDelete => {
                if let Some(target) = self.selected_file_path() {
                    self.delete_confirmation = Some(target);
                } else {
                    self.status_message = "Select a file or folder to delete".to_string();
                }
            }
            Action::PromptInput(character) => {
                if !character.is_control() && self.dialog_input.chars().count() < 120 {
                    self.dialog_input.push(character);
                }
            }
            Action::PromptBackspace => {
                self.dialog_input.pop();
            }
            Action::SubmitPrompt => self.submit_file_dialog(),
            Action::ConfirmDelete => self.confirm_delete(),
            Action::CancelOverlay => self.close_overlay(),
            Action::SettingsNextSection => {
                self.settings_section = self.settings_section.adjacent(1);
                self.settings_selected = 0;
            }
            Action::SettingsPrevSection => {
                self.settings_section = self.settings_section.adjacent(-1);
                self.settings_selected = 0;
            }
            Action::SettingsNextItem => {
                let max = self.settings_max_item();
                self.settings_selected = if self.settings_selected >= max {
                    0
                } else {
                    self.settings_selected + 1
                };
            }
            Action::SettingsPrevItem => {
                let max = self.settings_max_item();
                self.settings_selected = if self.settings_selected == 0 {
                    max
                } else {
                    self.settings_selected.saturating_sub(1)
                };
            }
            Action::SettingsSelect => self.apply_settings_selection(),
        }
    }

    fn apply_settings_selection(&mut self) {
        match self.settings_section {
            SettingsSection::Theme => {
                use crate::tui::theme::{Theme, ThemeKind};
                let all = ThemeKind::ALL;
                let idx = self.settings_selected % all.len();
                Theme::set(all[idx]);
                self.status_message = format!("Theme changed to {}", all[idx].name());
            }
            _ => {}
        }
    }

    fn settings_max_item(&self) -> usize {
        match self.settings_section {
            SettingsSection::Theme => {
                use crate::tui::theme::ThemeKind;
                ThemeKind::ALL.len().saturating_sub(1)
            }
            _ => 0,
        }
    }

    fn selected_file_path(&self) -> Option<PathBuf> {
        self.files
            .items
            .get(self.files.selected)
            .map(|item| item.path.clone())
    }

    fn open_file_dialog(&mut self, dialog: FileDialog, target: Option<PathBuf>) {
        self.file_dialog = Some(dialog);
        self.dialog_target = target;
        self.dialog_input.clear();
    }

    fn close_overlay(&mut self) {
        self.file_dialog = None;
        self.dialog_input.clear();
        self.dialog_target = None;
        self.delete_confirmation = None;
        self.show_settings = false;
        self.settings_selected = 0;
    }

    fn submit_file_dialog(&mut self) {
        let Some(dialog) = self.file_dialog else {
            return;
        };
        let name = sanitize_filename(self.dialog_input.trim());
        if !is_valid_entry_name(&name) {
            self.status_message =
                "Use a simple file or folder name without path separators".to_string();
            return;
        }

        let result = match dialog {
            FileDialog::NewFile => fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(self.files.current_dir.join(name))
                .map(|_| "File created"),
            FileDialog::NewFolder => {
                fs::create_dir(self.files.current_dir.join(name)).map(|_| "Folder created")
            }
            FileDialog::Rename => {
                let Some(target) = self.dialog_target.clone() else {
                    self.status_message = "Nothing selected to rename".to_string();
                    self.close_overlay();
                    return;
                };
                let destination = self.files.current_dir.join(name);
                if target == destination {
                    self.status_message = "Name unchanged".to_string();
                    self.close_overlay();
                    return;
                }
                if destination.exists() {
                    self.status_message = "An item with that name already exists".to_string();
                    return;
                }
                fs::rename(target, destination).map(|_| "Item renamed")
            }
        };

        match result {
            Ok(message) => {
                self.status_message = message.to_string();
                self.files_rescan_requested = true;
                self.close_overlay();
            }
            Err(error) => {
                self.status_message = format!(
                    "Operation failed: {}",
                    sanitize_error_message(&error.to_string())
                );
            }
        }
    }

    fn open_selected_file(&mut self) {
        let Some(target) = self.selected_file_path() else {
            self.status_message = "Select a file or folder first".to_string();
            return;
        };
        let result = if cfg!(target_os = "macos") {
            std::process::Command::new("open").arg(&target).spawn()
        } else if cfg!(target_os = "windows") {
            std::process::Command::new("cmd")
                .args(["/C", "start", ""])
                .arg(&target)
                .spawn()
        } else {
            std::process::Command::new("xdg-open").arg(&target).spawn()
        };
        self.status_message = match result {
            Ok(_) => "Opened with the system default application".to_string(),
            Err(error) => format!(
                "Unable to open item: {}",
                sanitize_error_message(&error.to_string())
            ),
        };
    }

    fn confirm_delete(&mut self) {
        let Some(target) = self.delete_confirmation.take() else {
            return;
        };
        if !is_safe_child(&self.files.current_dir, &target) || !is_path_safe(&target) {
            self.status_message = "Refused to delete an unsafe path".to_string();
            return;
        }

        let result = match fs::symlink_metadata(&target) {
            Ok(metadata) if metadata.file_type().is_dir() => fs::remove_dir_all(&target),
            Ok(_) => fs::remove_file(&target),
            Err(error) => Err(error),
        };
        match result {
            Ok(()) => {
                self.status_message = "Item deleted".to_string();
                self.files_rescan_requested = true;
                self.files.selected = self.files.selected.saturating_sub(1);
            }
            Err(error) => {
                self.status_message = format!(
                    "Delete failed: {}",
                    sanitize_error_message(&error.to_string())
                )
            }
        }
    }

    pub fn handle_file_msg(&mut self, msg: FileMsg) {
        match msg {
            FileMsg::Start(path, generation)
                if path == self.files.current_dir && generation == self.files.scan_generation =>
            {
                self.files.is_scanning = true;
                self.files.items.clear();
                self.files.selected = 0;
            }
            FileMsg::Start(_, _) => {}
            FileMsg::Done(path, generation, items)
                if path == self.files.current_dir && generation == self.files.scan_generation =>
            {
                self.files.items = items;
                self.files.selected = self
                    .files
                    .selected
                    .min(self.files.items.len().saturating_sub(1));
                self.files.is_scanning = false;
            }
            FileMsg::Done(_, _, _) => {}
        }
    }

    pub fn handle_junk_msg(&mut self, msg: JunkMsg) {
        match msg {
            JunkMsg::Start => {
                self.junk.is_scanning = true;
                self.junk.items.clear();
                self.junk.selected = 0;
                self.junk.total_size = 0;
            }
            JunkMsg::Done(items) => {
                let total = items.iter().map(|i| i.size).sum();
                self.junk.items = items;
                self.junk.selected = 0;
                self.junk.total_size = total;
                self.junk.is_scanning = false;
            }
        }
    }

    pub fn handle_apps_msg(&mut self, msg: AppsMsg) {
        match msg {
            AppsMsg::Start => {
                self.apps.is_scanning = true;
                self.apps.items.clear();
                self.apps.selected = 0;
            }
            AppsMsg::Done(items) => {
                self.apps.items = items;
                self.apps.clamp_selection();
                self.apps.is_scanning = false;
            }
        }
    }

    pub fn handle_pkgs_msg(&mut self, msg: PkgsMsg) {
        match msg {
            PkgsMsg::Start => {
                self.packages.is_scanning = true;
                self.packages.items.clear();
                self.packages.selected = 0;
                self.packages.uninstall_cmd.clear();
            }
            PkgsMsg::Done(items) => {
                self.packages.items = items;
                self.packages.clamp_selection();
                self.packages.uninstall_cmd.clear();
                self.packages.is_scanning = false;
            }
        }
    }
}

fn is_valid_entry_name(name: &str) -> bool {
    let path = Path::new(name);
    !name.is_empty()
        && name != "."
        && name != ".."
        && path.components().count() == 1
        && path.file_name().is_some()
}

fn is_safe_child(parent: &Path, child: &Path) -> bool {
    child.parent() == Some(parent) && child.file_name().is_some()
}

#[cfg(test)]
mod tests {
    use super::{Tab, is_safe_child, is_valid_entry_name};
    use std::path::Path;

    #[test]
    fn sidebar_navigation_wraps_at_both_ends() {
        assert_eq!(Tab::Info.adjacent(1), Tab::Monitor);
        assert_eq!(Tab::Monitor.adjacent(-1), Tab::Info);
    }

    #[test]
    fn file_names_cannot_escape_the_current_folder() {
        assert!(is_valid_entry_name("report.txt"));
        assert!(!is_valid_entry_name(""));
        assert!(!is_valid_entry_name("../report.txt"));
        assert!(!is_valid_entry_name("nested/report.txt"));
        assert!(is_safe_child(
            Path::new("/tmp/work"),
            Path::new("/tmp/work/file.txt")
        ));
        assert!(!is_safe_child(
            Path::new("/tmp/work"),
            Path::new("/tmp/file.txt")
        ));
    }
}
