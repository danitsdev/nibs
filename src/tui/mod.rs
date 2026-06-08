pub mod model;
pub mod update;
pub mod view;

use crate::cli::ScanProfile;
use crate::safety::ScanScope;
use crate::scanner::ScanOptions;
use anyhow::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use model::TuiState;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::path::PathBuf;
use std::time::Duration;

/// Launches the cleaner TUI immediately and scans in the background.
pub fn run_tui_scanning(
    target_path: PathBuf,
    scope: ScanScope,
    options: ScanOptions,
    dry_run: bool,
    profile: ScanProfile,
) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = TuiState::new(target_path.clone(), scope, Vec::new(), Vec::new(), dry_run);
    state.is_smart_clean = profile.auto_selects_safe_items();
    start_background_scan(&mut state, target_path, options);

    let loop_res = run_loop(&mut terminal, &mut state);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    loop_res
}

/// Launches and runs the TUI starting at the Home picker menu dashboard.
pub fn run_tui_home() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = TuiState::new(
        PathBuf::from("."),
        ScanScope::DirectoryScan(PathBuf::from(".")),
        Vec::new(),
        Vec::new(),
        false,
    );
    state.screen = model::TuiScreen::Home;
    state.is_home_mode = true;

    let loop_res = run_loop(&mut terminal, &mut state);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    loop_res
}

/// Launches the TUI directly into the Trash Manager.
pub fn run_tui_trash_manager() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = TuiState::new(
        PathBuf::from("."),
        ScanScope::DirectoryScan(PathBuf::from(".")),
        Vec::new(),
        Vec::new(),
        false,
    );
    state.trash_items = crate::cleaner::trash::list_trash_items();
    state.trash_selected_idx = 0;
    state.screen = model::TuiScreen::TrashManager;
    state.status_message = format!(
        "Arrows/jk: navigate │ r: Restore │ e: Empty all │ Esc: Back  ({} items in trash)",
        state.trash_items.len(),
    );

    let loop_res = run_loop(&mut terminal, &mut state);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    loop_res
}

pub(crate) fn start_background_scan(
    state: &mut TuiState,
    target_path: PathBuf,
    options: ScanOptions,
) {
    let (tx, rx) = std::sync::mpsc::channel();
    state.scan_rx = Some(rx);
    state.scan_files_count = 0;
    state.scan_findings_count = 0;
    state.scan_total_size = 0;
    state.scan_current_path = target_path.clone();
    state.screen = model::TuiScreen::Scanning;
    state.status_message = format!("Scanning {}...", target_path.display());

    std::thread::spawn(move || {
        let rules = crate::rules::load_all_rules(options.include_deep_rules);
        let _ =
            crate::scanner::scan_directory_with_progress(&target_path, &rules, &options, Some(&tx));
    });
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    state: &mut TuiState,
) -> Result<()> {
    while !state.should_quit {
        // Poll scanner background progress updates if scanning is active
        if let Some(ref rx) = state.scan_rx {
            while let Ok(progress) = rx.try_recv() {
                match progress {
                    crate::scanner::walker::ScanProgress::FilesScanned(count) => {
                        state.scan_files_count = count;
                    }
                    crate::scanner::walker::ScanProgress::CurrentPath(path) => {
                        state.scan_current_path = path;
                    }
                    crate::scanner::walker::ScanProgress::FindingAdded { path, size } => {
                        state.scan_findings_count += 1;
                        state.scan_total_size += size;
                        state.scan_current_path = path;
                    }

                    crate::scanner::walker::ScanProgress::Finished { findings, warnings } => {
                        state.findings = findings;
                        state.warnings = warnings;
                        state.selected_idx = 0;
                        state.selected_findings.clear();
                        state.finding_filter = model::FindingFilter::All;
                        state.search_active = false;
                        state.search_query.clear();

                        state.scan_rx = None;

                        // Record last scan info for the Today panel
                        state.last_scan_time = Some(std::time::Instant::now());
                        state.last_scan_findings = state.findings.len();
                        state.last_scan_size = state.scan_total_size;

                        let next_screen = if state.is_smart_clean {
                            model::TuiScreen::SmartClean
                        } else {
                            model::TuiScreen::Dashboard
                        };

                        if state.findings.is_empty() {
                            state.screen = next_screen;
                            state.status_message =
                                "Scan complete. No developer junk found.".to_string();
                        } else {
                            state.screen = next_screen;
                            if state.is_smart_clean {
                                state.select_all_safe();
                                state.last_scan_recommended = state.selected_findings.len();
                                state.status_message = format!(
                                    "Scan complete: {} findings, {} recommended selected.",
                                    state.findings.len(),
                                    state.selected_findings.len()
                                );
                            } else {
                                state.last_scan_recommended = 0;
                                state.status_message = format!(
                                    "Deep scan complete: {} findings. Review and select manually.",
                                    state.findings.len()
                                );
                            }
                        }
                        break;
                    }
                }
            }
        }

        // Periodically update system stats if we are on the Status screen
        if state.screen == model::TuiScreen::Status
            && state.sys_last_update.elapsed() >= Duration::from_millis(1500)
        {
            state.update_system_stats();
            state.sys_last_update = std::time::Instant::now();
        }

        if state.screen == model::TuiScreen::Goodbye
            && state.start_time.elapsed() >= Duration::from_secs(5)
        {
            state.should_quit = true;
            continue;
        }

        terminal.draw(|f| view::draw(state, f))?;

        if event::poll(Duration::from_millis(20))?
            && let Event::Key(key) = event::read()?
        {
            // Focus on key press events
            if key.kind == event::KeyEventKind::Press {
                update::handle_key_event(state, key, terminal);
            }
        }
        state.tick();
    }
    Ok(())
}
