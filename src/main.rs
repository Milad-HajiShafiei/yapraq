mod app;
mod events;
mod features;
mod tui;
mod utils;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{
    io::{self, stdout},
    panic,
};
use tokio::sync::mpsc;

use crate::app::{Action, App};
use crate::events::EventHandler;
use crate::features::files::scanner;

/// A wrapper to safely restore the terminal on panic.
fn setup_panic_hook() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_panic_hook();
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;

    let result = run_app().await;

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    result
}

async fn run_app() -> Result<()> {
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();

    let (tx, mut rx) = mpsc::unbounded_channel();
    EventHandler::start(tx);

    let (file_tx, mut file_rx) = mpsc::unbounded_channel();
    let (junk_tx, mut junk_rx) = mpsc::unbounded_channel();
    let (apps_tx, mut apps_rx) = mpsc::unbounded_channel();
    let (pkgs_tx, mut pkgs_rx) = mpsc::unbounded_channel();

    let generation = app.files.begin_scan();
    scanner::scan_directory(app.files.current_dir.clone(), generation, file_tx.clone());

    while app.running {
        terminal.draw(|frame| tui::UI::draw(frame, &app))?;

        tokio::select! {
            Some(event) = rx.recv() => {
                if let Some(action) = app.handle_event(event) {
                    let old_dir = app.files.current_dir.clone();

                    match &action {
                        Action::ScanJunk => crate::features::junk::scanner::scan_junk(junk_tx.clone()),
                        Action::ScanApps => crate::features::apps::scanner::scan_apps(apps_tx.clone()),
                        Action::ScanPackages => {
                            let tx = pkgs_tx.clone();
                            tokio::spawn(async move {
                                crate::features::packages::scanner::scan_packages(tx).await;
                            });
                        }
                        _ => {}
                    }

                    app.update(action);

                    if app.files.current_dir != old_dir || app.files_rescan_requested {
                        app.files_rescan_requested = false;
                        let generation = app.files.begin_scan();
                        scanner::scan_directory(
                            app.files.current_dir.clone(),
                            generation,
                            file_tx.clone(),
                        );
                    }
                }
            }
            Some(msg) = file_rx.recv() => app.handle_file_msg(msg),
            Some(msg) = junk_rx.recv() => app.handle_junk_msg(msg),
            Some(msg) = apps_rx.recv() => app.handle_apps_msg(msg),
            Some(msg) = pkgs_rx.recv() => app.handle_pkgs_msg(msg),
            else => break,
        }
    }

    terminal.show_cursor()?;
    Ok(())
}
