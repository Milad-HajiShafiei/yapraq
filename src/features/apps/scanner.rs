use crate::features::apps::{AppItem, AppsMsg};
use std::path::PathBuf;
use tokio::sync::mpsc::UnboundedSender;
use walkdir::WalkDir;

pub fn scan_apps(tx: UnboundedSender<AppsMsg>) {
    let _ = tx.send(AppsMsg::Start);

    tokio::task::spawn_blocking(move || {
        let mut items = Vec::new();
        let mut targets = Vec::new();

        // OS-specific native app directories
        if cfg!(target_os = "macos") {
            targets.push(PathBuf::from("/Applications"));
            if let Some(home) = dirs::home_dir() {
                targets.push(home.join("Applications"));
            }
        } else if cfg!(target_os = "linux") {
            targets.push(PathBuf::from("/usr/share/applications"));
            targets.push(PathBuf::from("/var/lib/snapd/desktop/applications"));
        } else if cfg!(target_os = "windows") {
            targets.push(PathBuf::from("C:\\Program Files"));
            targets.push(PathBuf::from("C:\\Program Files (x86)"));
        }

        for target in targets {
            if !target.exists() {
                continue;
            }

            // Read only the top-level children (don't recurse deeply into /Applications)
            if let Ok(entries) = std::fs::read_dir(&target) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();

                    // Clean up names (e.g. remove ".app" or ".desktop")
                    let clean_name = name.replace(".app", "").replace(".desktop", "");

                    let size = if path.is_dir() {
                        WalkDir::new(&path)
                            .into_iter()
                            .filter_map(|e| e.ok())
                            .filter_map(|e| e.metadata().ok())
                            .filter(|m| m.is_file())
                            .map(|m| m.len())
                            .sum()
                    } else {
                        entry.metadata().map(|m| m.len()).unwrap_or(0)
                    };

                    items.push(AppItem {
                        name: clean_name,
                        path,
                        size,
                    });
                }
            }
        }

        items.sort_by_key(|item| item.name.to_lowercase());
        let _ = tx.send(AppsMsg::Done(items));
    });
}
