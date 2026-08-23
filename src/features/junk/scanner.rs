use crate::features::junk::{JunkItem, JunkMsg};
use directories::BaseDirs;
use std::{collections::HashSet, path::Path};
use tokio::sync::mpsc::UnboundedSender;
use walkdir::WalkDir;

pub fn scan_junk(tx: UnboundedSender<JunkMsg>) {
    let _ = tx.send(JunkMsg::Start);

    tokio::task::spawn_blocking(move || {
        let mut targets = Vec::new();
        if let Some(base_dirs) = BaseDirs::new() {
            targets.push(base_dirs.cache_dir().to_path_buf());
        }
        targets.push(std::env::temp_dir());

        let mut seen_targets = HashSet::new();
        targets.retain(|target| {
            let canonical = std::fs::canonicalize(target).unwrap_or_else(|_| target.clone());
            seen_targets.insert(canonical)
        });

        let junk_extensions = ["tmp", "log", "bak", "old", "dmp", "cache"];
        let junk_files = [
            "thumbs.db",
            ".ds_store",
            "desktop.ini",
            "npm-debug.log",
            "yarn-error.log",
        ];
        let mut seen_files = HashSet::new();
        let mut items = Vec::new();

        for target in targets {
            if !target.is_dir() {
                continue;
            }

            let walker = WalkDir::new(&target)
                .follow_links(false)
                .into_iter()
                .filter_entry(|entry| {
                    !is_skipped_directory(entry.path(), entry.file_type().is_dir())
                });

            for entry in walker.filter_map(Result::ok) {
                if !entry.file_type().is_file() {
                    continue;
                }

                let path = entry.path();
                let name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default()
                    .to_ascii_lowercase();
                let extension = path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .unwrap_or_default()
                    .to_ascii_lowercase();

                let reason = if junk_files.contains(&name.as_str()) {
                    Some("System/Cache File")
                } else if junk_extensions.contains(&extension.as_str()) {
                    Some("Temp/Log File")
                } else {
                    None
                };

                let Some(reason) = reason else { continue };
                let path = path.to_path_buf();
                if !seen_files.insert(path.clone()) {
                    continue;
                }

                if let Ok(metadata) = entry.metadata() {
                    items.push(JunkItem {
                        path,
                        size: metadata.len(),
                        reason: reason.to_string(),
                    });
                }
            }
        }

        items.sort_by(|left, right| left.path.cmp(&right.path));
        let _ = tx.send(JunkMsg::Done(items));
    });
}

fn is_skipped_directory(path: &Path, is_dir: bool) -> bool {
    if !is_dir {
        return false;
    }

    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| {
            matches!(
                name.to_ascii_lowercase().as_str(),
                ".git" | "node_modules" | "target" | "venv" | ".venv"
            )
        })
        .unwrap_or(false)
}
