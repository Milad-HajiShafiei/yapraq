use crate::features::files::{FileItem, FileMsg};
use std::path::PathBuf;
use tokio::sync::mpsc::UnboundedSender;
use walkdir::WalkDir;

pub fn scan_directory(path: PathBuf, generation: u64, tx: UnboundedSender<FileMsg>) {
    let _ = tx.send(FileMsg::Start(path.clone(), generation));

    tokio::task::spawn_blocking(move || {
        let mut items = Vec::new();

        let entries = match std::fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(_) => {
                let _ = tx.send(FileMsg::Done(path, generation, items));
                return;
            }
        };

        for entry in entries.flatten() {
            let item_path = entry.path();
            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };
            let is_dir = metadata.is_dir();
            let size = if is_dir {
                WalkDir::new(&item_path)
                    .min_depth(1)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter_map(|entry| entry.metadata().ok())
                    .filter(|metadata| metadata.is_file())
                    .fold(0_u64, |total, metadata| {
                        total.saturating_add(metadata.len())
                    })
            } else {
                metadata.len()
            };

            items.push(FileItem {
                name: entry.file_name().to_string_lossy().into_owned(),
                is_dir,
                size,
                path: item_path,
            });
        }

        items.sort_by(|left, right| {
            right
                .is_dir
                .cmp(&left.is_dir)
                .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
                .then_with(|| left.name.cmp(&right.name))
        });

        let _ = tx.send(FileMsg::Done(path, generation, items));
    });
}
