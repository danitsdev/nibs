use crate::cleaner::clean_findings;
use crate::cleaner::trash::empty_trash_directory;
use crate::tui::model::{TuiScreen, TuiState};
use crate::tui::view::format_size;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::path::PathBuf;

/// Handles keyboard inputs in the TUI loop.
pub fn handle_key_event(
    state: &mut TuiState,
    key_event: KeyEvent,
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) {
    // Check for Ctrl+C (always force quits)
    if key_event.modifiers.contains(KeyModifiers::CONTROL) && key_event.code == KeyCode::Char('c') {
        state.should_quit = true;
        return;
    }

    match state.screen {
        TuiScreen::Home => {
            handle_home_keys(state, key_event, terminal);
        }
        TuiScreen::HomeConfirmTrash => {
            handle_home_trash_confirm_keys(state, key_event);
        }
        TuiScreen::PathInput { is_analyze } => {
            handle_path_input_keys(state, key_event, is_analyze, terminal);
        }
        TuiScreen::AppUninstallSelector => {
            handle_uninstall_selector_keys(state, key_event, terminal);
        }
        TuiScreen::AppUninstallList => {
            handle_uninstall_list_keys(state, key_event);
        }
        TuiScreen::DoctorReport => {
            handle_doctor_keys(state, key_event);
        }
        TuiScreen::Wizard => {
            handle_wizard_keys(state, key_event);
        }
        TuiScreen::Dashboard => {
            handle_dashboard_keys(state, key_event);
        }
        TuiScreen::Scanning => {
            handle_scanning_keys(state, key_event);
        }
        TuiScreen::Optimize => {
            handle_optimize_keys(state, key_event);
        }
        TuiScreen::Analyze => {
            handle_analyze_keys(state, key_event);
        }
        TuiScreen::Status => {
            handle_status_keys(state, key_event);
        }
        TuiScreen::Settings => {
            handle_settings_keys(state, key_event);
        }
        TuiScreen::Goodbye => {
            state.should_quit = true;
        }
    }
}

fn handle_home_keys(
    state: &mut TuiState,
    key_event: KeyEvent,
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) {
    match key_event.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if state.home_selected_idx > 0 {
                state.home_selected_idx -= 1;
            } else {
                state.home_selected_idx = 7; // Wrap around (was 6)
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if state.home_selected_idx < 7 {
                // was 6
                state.home_selected_idx += 1;
            } else {
                state.home_selected_idx = 0; // Wrap around
            }
        }
        KeyCode::Enter => {
            match state.home_selected_idx {
                0 => {
                    // Clean
                    state.input_buffer = ".".to_string();
                    state.screen = TuiScreen::PathInput { is_analyze: false };
                    state.status_message = "Enter path to scan for developer junk".to_string();
                }
                1 => {
                    // Optimize
                    state.screen = TuiScreen::Optimize;
                    state.opt_selected_indices.clear();
                    state.opt_cursor_idx = 0;
                    state.opt_in_progress = false;
                    state.opt_results.clear();
                    state.status_message =
                        "Space: toggle option │ O: run optimization │ Esc: back".to_string();
                }
                2 => {
                    // Uninstall
                    state.status_message =
                        "Loading system applications. Please wait...".to_string();
                    let _ = terminal.draw(|f| crate::tui::view::draw(state, f));
                    state.installed_apps = crate::uninstall::apps::discover_installed_apps();
                    state.selected_app_idx = 0;
                    state.screen = TuiScreen::AppUninstallSelector;
                    state.status_message =
                        "Select an app. Enter: inspect leftovers │ C: clean all remnants"
                            .to_string();
                }
                3 => {
                    // Analyze (integrated disk tree browser!)
                    state.status_message =
                        "Scanning directory structure. Please wait...".to_string();
                    let _ = terminal.draw(|f| crate::tui::view::draw(state, f));
                    let (arena, warnings) =
                        crate::analyze::tree::build_disk_tree(std::path::Path::new("."));
                    state.analyze_arena = arena;
                    state.analyze_warnings = warnings;
                    state.analyze_current_dir_idx = 0;
                    state.analyze_selected_idx = 0;
                    state.analyze_history.clear();
                    state.analyze_show_confirmation = false;
                    state.screen = TuiScreen::Analyze;
                    state.status_message = "Use arrows/jk to navigate │ Enter: Open │ Backspace/u: Up │ d: Trash │ Esc: Back".to_string();
                }
                4 => {
                    // Status (integrated status telemetry dashboard!)
                    state.status_message =
                        "Gathering system specifications. Please wait...".to_string();
                    let _ = terminal.draw(|f| crate::tui::view::draw(state, f));
                    state.update_system_stats();
                    state.screen = TuiScreen::Status;
                    state.status_message =
                        "Arrows/jk: scroll processes │ k: Kill process │ r: Refresh │ Esc: Back"
                            .to_string();
                }
                5 => {
                    state.screen = TuiScreen::HomeConfirmTrash;
                    state.status_message =
                        "Confirm emptying the system trash. This deletes trash contents permanently."
                            .to_string();
                }
                6 => {
                    // Settings/Configuration screen
                    state.screen = TuiScreen::Settings;
                    state.settings_cursor_idx = 0;
                    state.status_message =
                        "Space/Enter: Toggle/cycle option │ Esc: Back to menu".to_string();
                }
                7 => {
                    // Goodbye exit
                    state.screen = TuiScreen::Goodbye;
                    state.start_time = std::time::Instant::now();
                    state.status_message =
                        "Thank you for using Nibble. Press any key to quit.".to_string();
                }
                _ => {}
            }
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            state.screen = TuiScreen::Goodbye;
            state.start_time = std::time::Instant::now();
            state.status_message = "Thank you for using Nibble. Press any key to quit.".to_string();
        }
        _ => {}
    }
}

fn handle_settings_keys(state: &mut TuiState, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if state.settings_cursor_idx > 0 {
                state.settings_cursor_idx -= 1;
            } else {
                state.settings_cursor_idx = 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if state.settings_cursor_idx < 1 {
                state.settings_cursor_idx += 1;
            } else {
                state.settings_cursor_idx = 0;
            }
        }
        KeyCode::Char(' ') | KeyCode::Enter => match state.settings_cursor_idx {
            0 => {
                state.delete_directly = !state.delete_directly;
            }
            1 => {
                state.theme = state.theme.next();
            }
            _ => {}
        },
        KeyCode::Esc | KeyCode::Char('q') => {
            state.screen = TuiScreen::Home;
            state.status_message = String::new();
        }
        _ => {}
    }
}

fn home_trash_root() -> Option<std::path::PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(std::path::PathBuf::from)
        .map(|home| home.join(".local/share/Trash"))
}
fn do_empty_trash(state: &mut TuiState) {
    match home_trash_root() {
        Some(trash_root) if trash_root.exists() => match empty_trash_directory(&trash_root) {
            Ok(_) => {
                state.status_message = "System trash emptied successfully.".to_string();
            }
            Err(err) => {
                state.status_message = format!("Failed to empty trash: {}", err);
            }
        },
        Some(_) => {
            state.status_message = "No system trash directory found for this user.".to_string();
        }
        None => {
            state.status_message =
                "Unable to locate the home directory for trash cleanup.".to_string();
        }
    }
    state.screen = TuiScreen::Home;
}

fn cancel_trash(state: &mut TuiState, msg: &str) {
    state.status_message = msg.to_string();
    state.screen = TuiScreen::Home;
}

fn do_app_clean_remnants(state: &mut TuiState) {
    if let Some(app) = state.installed_apps.get(state.selected_app_idx).cloned() {
        let mut paths = crate::uninstall::find_app_remnants(&app.exec);
        if !app.name.eq_ignore_ascii_case(&app.exec) {
            let mut other_paths = crate::uninstall::find_app_remnants(&app.name);
            paths.append(&mut other_paths);
        }
        if app.desktop_file.exists() {
            paths.push(app.desktop_file.clone());
        }
        paths.sort();
        paths.dedup();

        let mut filtered = Vec::new();
        for p in paths {
            let mut is_sub = false;
            for other in &filtered {
                if p.starts_with(other) {
                    is_sub = true;
                    break;
                }
            }
            if !is_sub {
                filtered.push(p);
            }
        }

        let mut success_count = 0;
        let total_to_trash = filtered.len();
        for p in filtered {
            if crate::cleaner::clean_path(&p, state.delete_directly).is_ok() {
                success_count += 1;
            }
        }
        state.status_message = format!(
            "Cleaned {}/{} remnants for application '{}'.",
            success_count, total_to_trash, app.name
        );
    }
    state.show_confirmation = false;
    state.screen = TuiScreen::Home;
}

fn do_trash_remnants(state: &mut TuiState) {
    let mut success_count = 0;
    for &idx in &state.selected_remnants {
        if let Some((path, _)) = state.app_remnants.get(idx)
            && crate::cleaner::clean_path(path, state.delete_directly).is_ok()
        {
            success_count += 1;
        }
    }
    state.status_message = if state.delete_directly {
        format!(
            "Permanently deleted {}/{} remnants.",
            success_count,
            state.selected_remnants.len()
        )
    } else {
        format!(
            "Moved {}/{} remnants to system trash.",
            success_count,
            state.selected_remnants.len()
        )
    };
    state.app_remnants.clear();
    state.selected_remnants.clear();
    state.selected_idx = 0;
    state.show_confirmation = false;
    state.screen = TuiScreen::Home;
}

fn do_clean_findings(state: &mut TuiState) {
    let mut to_clean = Vec::new();
    let mut indices: Vec<usize> = state.selected_findings.iter().cloned().collect();
    indices.sort_by(|a, b| b.cmp(a));

    for &idx in &indices {
        if let Some(finding) = state.findings.get(idx) {
            to_clean.push(finding.clone());
        }
    }

    match clean_findings(&to_clean, state.dry_run, state.delete_directly) {
        Ok(results) => {
            let total_bytes: u64 = results.iter().map(|r| r.bytes_freed).sum();
            state.status_message = format!(
                "Reclaimed {} from {} items ({})",
                format_size(total_bytes),
                results.len(),
                if state.dry_run {
                    "Simulated"
                } else if state.delete_directly {
                    "Permanently Deleted"
                } else {
                    "System Trash"
                }
            );

            for idx in indices {
                state.findings.remove(idx);
            }
            state.selected_findings.clear();
            state.selected_idx = 0;
            state.clamp_selected_to_filter();
        }
        Err(e) => {
            state.status_message = format!("Error running cleanup: {}", e);
        }
    }
    state.show_confirmation = false;
}

fn handle_home_trash_confirm_keys(state: &mut TuiState, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            do_empty_trash(state);
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            cancel_trash(state, "Clean Trash cancelled.");
        }
        KeyCode::Left | KeyCode::Right => {
            state.confirm_idx = 1 - state.confirm_idx;
        }
        KeyCode::Enter => {
            if state.confirm_idx == 0 {
                do_empty_trash(state);
            } else {
                cancel_trash(state, "Clean Trash cancelled.");
            }
        }
        _ => {}
    }
}

fn handle_path_input_keys(
    state: &mut TuiState,
    key_event: KeyEvent,
    is_analyze: bool,
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) {
    match key_event.code {
        KeyCode::Char(c) => {
            state.input_buffer.push(c);
        }
        KeyCode::Backspace => {
            state.input_buffer.pop();
        }
        KeyCode::Esc => {
            state.screen = TuiScreen::Home;
            state.status_message = String::new();
        }
        KeyCode::Enter => {
            let path_str = state.input_buffer.trim();
            let path = if path_str.is_empty() {
                PathBuf::from(".")
            } else {
                PathBuf::from(path_str)
            };

            if is_analyze {
                if path.exists() {
                    // Suspend TUI, run analyzer, resume TUI
                    let _ = disable_raw_mode();
                    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
                    let _ = terminal.show_cursor();

                    let _ = crate::analyze::run_analyze_tui(&path);

                    let _ = enable_raw_mode();
                    let _ = execute!(std::io::stdout(), EnterAlternateScreen);
                    let _ = terminal.clear();
                    state.screen = TuiScreen::Home;
                    state.status_message = "Returned from Disk Analyzer.".to_string();
                } else {
                    state.status_message = format!("Error: Path does not exist: {:?}", path);
                }
            } else {
                // Initialize Asynchronous background scan
                state.scan_files_count = 0;
                state.scan_findings_count = 0;
                state.scan_total_size = 0;
                state.scan_current_path = path.clone();

                let (tx, rx) = std::sync::mpsc::channel();
                state.scan_rx = Some(rx);
                state.screen = TuiScreen::Scanning;
                state.status_message = format!("Running background filesystem scan on {:?}", path);

                // Spawn worker thread
                let target_path = path.clone();
                std::thread::spawn(move || {
                    let rules = crate::rules::load_all_rules();
                    let options = crate::scanner::ScanOptions {
                        detect_duplicates: false,
                        min_age_days: 0,
                        brute: false,
                        ..Default::default()
                    };

                    let _ = crate::scanner::scan_directory_with_progress(
                        &target_path,
                        &rules,
                        &options,
                        Some(&tx),
                    );
                });
            }
        }
        _ => {}
    }
}

fn handle_uninstall_selector_keys(
    state: &mut TuiState,
    key_event: KeyEvent,
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) {
    if state.show_confirmation {
        match key_event.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                do_app_clean_remnants(state);
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                state.show_confirmation = false;
                state.status_message = "App leftovers cleaning cancelled.".to_string();
            }
            KeyCode::Left | KeyCode::Right => {
                state.confirm_idx = 1 - state.confirm_idx;
            }
            KeyCode::Enter => {
                if state.confirm_idx == 0 {
                    do_app_clean_remnants(state);
                } else {
                    state.show_confirmation = false;
                    state.status_message = "App leftovers cleaning cancelled.".to_string();
                }
            }
            _ => {}
        }
        return;
    }

    match key_event.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if state.selected_app_idx > 0 {
                state.selected_app_idx -= 1;
            } else if !state.installed_apps.is_empty() {
                state.selected_app_idx = state.installed_apps.len() - 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if !state.installed_apps.is_empty()
                && state.selected_app_idx < state.installed_apps.len() - 1
            {
                state.selected_app_idx += 1;
            } else {
                state.selected_app_idx = 0;
            }
        }
        KeyCode::Enter => {
            // Expand details and check leftovers breakdown
            if let Some(app) = state.installed_apps.get(state.selected_app_idx).cloned() {
                state.app_name = app.name.clone();
                state.status_message = format!("Searching remnants for {}...", app.name);
                state.screen = TuiScreen::AppUninstallList;
                let _ = terminal.draw(|f| crate::tui::view::draw(state, f));

                let mut paths = crate::uninstall::find_app_remnants(&app.exec);

                if !app.name.eq_ignore_ascii_case(&app.exec) {
                    let mut other_paths = crate::uninstall::find_app_remnants(&app.name);
                    paths.append(&mut other_paths);
                }
                if app.desktop_file.exists() {
                    paths.push(app.desktop_file.clone());
                }
                paths.sort();
                paths.dedup();

                let mut filtered = Vec::new();
                for p in paths {
                    let mut is_sub = false;
                    for other in &filtered {
                        if p.starts_with(other) {
                            is_sub = true;
                            break;
                        }
                    }
                    if !is_sub {
                        filtered.push(p);
                    }
                }

                let mut remnants = Vec::new();
                for p in filtered {
                    let (size, _, _) = crate::scanner::size::calculate_size(&p);
                    remnants.push((p, size));
                }
                state.app_remnants = remnants;
                state.selected_remnants.clear();
                for i in 0..state.app_remnants.len() {
                    state.selected_remnants.insert(i);
                }
                state.selected_idx = 0;
                state.status_message = format!(
                    "Found {} remnants. Space to toggle, C to trash.",
                    state.app_remnants.len()
                );
            }
        }
        KeyCode::Char('c') => {
            if !state.installed_apps.is_empty() {
                state.show_confirmation = true;
            }
        }
        KeyCode::Esc => {
            state.screen = TuiScreen::Home;
            state.status_message = String::new();
        }
        _ => {}
    }
}

fn handle_uninstall_list_keys(state: &mut TuiState, key_event: KeyEvent) {
    if state.show_confirmation {
        match key_event.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                do_trash_remnants(state);
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                state.show_confirmation = false;
                state.status_message = "Cleanup cancelled.".to_string();
            }
            KeyCode::Left | KeyCode::Right => {
                state.confirm_idx = 1 - state.confirm_idx;
            }
            KeyCode::Enter => {
                if state.confirm_idx == 0 {
                    do_trash_remnants(state);
                } else {
                    state.show_confirmation = false;
                    state.status_message = "Cleanup cancelled.".to_string();
                }
            }
            _ => {}
        }
        return;
    }

    match key_event.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if state.selected_idx > 0 {
                state.selected_idx -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if !state.app_remnants.is_empty() && state.selected_idx < state.app_remnants.len() - 1 {
                state.selected_idx += 1;
            }
        }
        KeyCode::Char(' ') => {
            if state.selected_remnants.contains(&state.selected_idx) {
                state.selected_remnants.remove(&state.selected_idx);
            } else {
                state.selected_remnants.insert(state.selected_idx);
            }
        }
        KeyCode::Char('a') => {
            if state.selected_remnants.len() == state.app_remnants.len() {
                state.selected_remnants.clear();
            } else {
                for i in 0..state.app_remnants.len() {
                    state.selected_remnants.insert(i);
                }
            }
        }
        KeyCode::Char('c') => {
            if state.selected_remnants.is_empty() {
                state.status_message = "Please select at least one item to clean.".to_string();
            } else {
                state.show_confirmation = true;
            }
        }
        KeyCode::Esc => {
            state.screen = TuiScreen::Home;
            state.status_message = String::new();
        }
        _ => {}
    }
}

fn handle_doctor_keys(state: &mut TuiState, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if state.doctor_selected_idx > 0 {
                state.doctor_selected_idx -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if !state.doctor_results.is_empty()
                && state.doctor_selected_idx < state.doctor_results.len() - 1
            {
                state.doctor_selected_idx += 1;
            }
        }
        KeyCode::Esc => {
            state.screen = TuiScreen::Home;
            state.status_message = String::new();
        }
        _ => {}
    }
}

fn handle_wizard_keys(state: &mut TuiState, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            if state.is_home_mode {
                state.screen = TuiScreen::Home;
            } else {
                state.should_quit = true;
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if state.wizard_idx > 0 {
                state.wizard_idx -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if state.wizard_idx < 2 {
                state.wizard_idx += 1;
            }
        }
        KeyCode::Enter => {
            state.apply_profile();
        }
        _ => {}
    }
}

fn handle_dashboard_keys(state: &mut TuiState, key_event: KeyEvent) {
    if state.search_active {
        match key_event.code {
            KeyCode::Esc => {
                state.search_active = false;
                state.status_message =
                    "Search kept. Press / to edit or Ctrl+U to clear.".to_string();
            }
            KeyCode::Enter => {
                state.search_active = false;
                state.clamp_selected_to_filter();
                state.status_message = format!(
                    "Search: '{}' ({} visible)",
                    state.search_query,
                    state.filter_count()
                );
            }
            KeyCode::Backspace => {
                state.search_query.pop();
                state.clamp_selected_to_filter();
            }
            KeyCode::Char('u') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                state.clear_search();
            }
            KeyCode::Char(c) => {
                state.search_query.push(c);
                state.clamp_selected_to_filter();
            }
            _ => {}
        }
        return;
    }

    if state.show_confirmation {
        match key_event.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                do_clean_findings(state);
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                state.show_confirmation = false;
                state.status_message = "Cleanup cancelled.".to_string();
            }
            KeyCode::Left | KeyCode::Right => {
                state.confirm_idx = 1 - state.confirm_idx;
            }
            KeyCode::Enter => {
                if state.confirm_idx == 0 {
                    do_clean_findings(state);
                } else {
                    state.show_confirmation = false;
                    state.status_message = "Cleanup cancelled.".to_string();
                }
            }
            _ => {}
        }
        return;
    }

    match key_event.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            if state.is_home_mode {
                state.screen = TuiScreen::Home;
            } else {
                state.should_quit = true;
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.move_finding_selection(-1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.move_finding_selection(1);
        }
        KeyCode::Char(' ') => {
            state.toggle_select_current();
        }
        KeyCode::Char('/') => {
            state.search_active = true;
            state.status_message = "Search findings by path, rule, category, or risk.".to_string();
        }
        KeyCode::Char('u') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            state.clear_search();
        }
        KeyCode::Char('f') | KeyCode::Char('F') => {
            state.cycle_finding_filter();
        }
        KeyCode::Char('v') | KeyCode::Char('V') => {
            state.toggle_view_mode();
        }
        KeyCode::Char('a') => {
            state.select_all_safe();
        }
        KeyCode::Char('c') => {
            if state.selected_findings.is_empty() {
                state.status_message =
                    "No items selected for cleaning. Space to select.".to_string();
                return;
            }
            state.show_confirmation = true;
        }
        _ => {}
    }
}

fn handle_scanning_keys(state: &mut TuiState, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            // Drop channel receiver to early-exit thread scan loop
            state.scan_rx = None;
            state.screen = TuiScreen::Home;
            state.status_message = "Scan cancelled by user.".to_string();
        }
        _ => {}
    }
}

fn handle_optimize_keys(state: &mut TuiState, key_event: KeyEvent) {
    if state.opt_in_progress {
        return;
    }
    match key_event.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if state.opt_cursor_idx > 0 {
                state.opt_cursor_idx -= 1;
            } else {
                state.opt_cursor_idx = 4;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if state.opt_cursor_idx < 4 {
                state.opt_cursor_idx += 1;
            } else {
                state.opt_cursor_idx = 0;
            }
        }
        KeyCode::Char(' ') => {
            if state.opt_selected_indices.contains(&state.opt_cursor_idx) {
                state.opt_selected_indices.remove(&state.opt_cursor_idx);
            } else {
                state.opt_selected_indices.insert(state.opt_cursor_idx);
            }
        }
        KeyCode::Char('o') | KeyCode::Char('O') => {
            if state.opt_selected_indices.is_empty() {
                state.status_message =
                    "Select at least one option to run optimization!".to_string();
            } else {
                run_optimizations(state);
            }
        }
        KeyCode::Esc => {
            state.screen = TuiScreen::Home;
            state.status_message = String::new();
        }
        _ => {}
    }
}

fn run_optimizations(state: &mut TuiState) {
    state.opt_results.clear();
    state.opt_in_progress = true;

    for &idx in &state.opt_selected_indices {
        match idx {
            0 => {
                state
                    .opt_results
                    .push("Purging RAM PageCache...".to_string());
                if std::fs::write("/proc/sys/vm/drop_caches", "3").is_ok() {
                    state
                        .opt_results
                        .push("  -> Successfully purged RAM cache.".to_string());
                } else {
                    let output = std::process::Command::new("sudo")
                        .arg("sysctl")
                        .arg("-w")
                        .arg("vm.drop_caches=3")
                        .output();
                    match output {
                        Ok(out) if out.status.success() => {
                            state
                                .opt_results
                                .push("  -> Successfully purged RAM cache (via sudo).".to_string());
                        }
                        _ => {
                            state.opt_results.push("  -> Error: Permission denied. Run Nibble as root/sudo to purge PageCache.".to_string());
                        }
                    }
                }
            }
            1 => {
                state
                    .opt_results
                    .push("Vacuuming systemd journal logs to 100MB...".to_string());
                let output = std::process::Command::new("journalctl")
                    .arg("--vacuum-size=100M")
                    .output();
                match output {
                    Ok(out) if out.status.success() => {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        let last_line = stdout.lines().last().unwrap_or("Vacuum complete.");
                        state.opt_results.push(format!("  -> {}", last_line.trim()));
                    }
                    _ => {
                        let output_sudo = std::process::Command::new("sudo")
                            .arg("journalctl")
                            .arg("--vacuum-size=100M")
                            .output();
                        match output_sudo {
                            Ok(out) if out.status.success() => {
                                let stdout = String::from_utf8_lossy(&out.stdout);
                                let last_line = stdout.lines().last().unwrap_or("Vacuum complete.");
                                state.opt_results.push(format!("  -> {}", last_line.trim()));
                            }
                            _ => {
                                state
                                    .opt_results
                                    .push("  -> Error running journalctl vacuum.".to_string());
                            }
                        }
                    }
                }
            }
            2 => {
                state
                    .opt_results
                    .push("Cleaning package manager cache...".to_string());
                let mut cleaned = false;
                if std::process::Command::new("apt-get")
                    .arg("--version")
                    .output()
                    .is_ok()
                {
                    let _ = std::process::Command::new("sudo")
                        .arg("apt-get")
                        .arg("clean")
                        .output();
                    state
                        .opt_results
                        .push("  -> Executed 'apt-get clean'.".to_string());
                    cleaned = true;
                }
                if std::process::Command::new("dnf")
                    .arg("--version")
                    .output()
                    .is_ok()
                {
                    let _ = std::process::Command::new("sudo")
                        .arg("dnf")
                        .arg("clean")
                        .arg("all")
                        .output();
                    state
                        .opt_results
                        .push("  -> Executed 'dnf clean all'.".to_string());
                    cleaned = true;
                }
                if std::process::Command::new("pacman")
                    .arg("--version")
                    .output()
                    .is_ok()
                {
                    let _ = std::process::Command::new("sudo")
                        .arg("pacman")
                        .arg("-Sc")
                        .arg("--noconfirm")
                        .output();
                    state
                        .opt_results
                        .push("  -> Executed 'pacman -Sc'.".to_string());
                    cleaned = true;
                }
                if !cleaned {
                    state
                        .opt_results
                        .push("  -> No supported package manager found.".to_string());
                }
            }
            3 => {
                state
                    .opt_results
                    .push("Removing rotated log archives from /var/log...".to_string());
                let mut count = 0;
                let mut freed = 0;
                if let Ok(entries) = std::fs::read_dir("/var/log") {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            let name = path.file_name().unwrap_or_default().to_string_lossy();
                            if name.ends_with(".gz")
                                || name.chars().last().unwrap_or(' ').is_numeric()
                            {
                                if let Ok(meta) = path.metadata() {
                                    freed += meta.len();
                                }
                                if std::fs::remove_file(&path).is_ok() {
                                    count += 1;
                                } else {
                                    let _ = std::process::Command::new("sudo")
                                        .arg("rm")
                                        .arg("-f")
                                        .arg(&path)
                                        .output();
                                    count += 1;
                                }
                            }
                        }
                    }
                }
                state.opt_results.push(format!(
                    "  -> Successfully removed {} rotated log files (freed {}).",
                    count,
                    format_size(freed)
                ));
            }
            4 => {
                state
                    .opt_results
                    .push("Syncing filesystem disk buffers...".to_string());
                let _ = std::process::Command::new("sync").output();
                state
                    .opt_results
                    .push("  -> Done (sync command completed).".to_string());
            }
            _ => {}
        }
    }
    state.opt_in_progress = false;
    state.status_message = "Optimization complete! Esc: back".to_string();
}

fn handle_analyze_keys(state: &mut TuiState, key_event: KeyEvent) {
    let mut sorted_children = {
        let mut children = if state.analyze_arena.is_empty() {
            Vec::new()
        } else {
            state.analyze_arena[state.analyze_current_dir_idx]
                .children
                .clone()
        };
        children.sort_by(|&a, &b| {
            state.analyze_arena[b]
                .size_bytes
                .cmp(&state.analyze_arena[a].size_bytes)
        });
        children
    };

    if state.analyze_show_confirmation {
        let do_trash = |state: &mut TuiState, children: &mut Vec<usize>| {
            if !children.is_empty() && state.analyze_selected_idx < children.len() {
                let target_idx = children[state.analyze_selected_idx];
                let target_path = state.analyze_arena[target_idx].path.clone();

                state.status_message = if state.delete_directly {
                    format!("Permanently deleting: {:?}", target_path)
                } else {
                    format!("Moving to trash: {:?}", target_path)
                };
                match crate::cleaner::clean_path(&target_path, state.delete_directly) {
                    Ok(_) => {
                        state.status_message = if state.delete_directly {
                            format!("Successfully deleted: {}", target_path.display())
                        } else {
                            format!("Successfully moved to trash: {}", target_path.display())
                        };
                        crate::analyze::tree::delete_node_from_tree(
                            &mut state.analyze_arena,
                            target_idx,
                        );

                        let new_children = state.analyze_arena[state.analyze_current_dir_idx]
                            .children
                            .clone();
                        *children = new_children;
                        children.sort_by(|&a, &b| {
                            state.analyze_arena[b]
                                .size_bytes
                                .cmp(&state.analyze_arena[a].size_bytes)
                        });

                        if state.analyze_selected_idx > 0
                            && state.analyze_selected_idx >= children.len()
                        {
                            state.analyze_selected_idx = children.len().saturating_sub(1);
                        }
                    }
                    Err(e) => {
                        state.status_message = format!("Error: {}", e);
                    }
                }
            }
            state.analyze_show_confirmation = false;
        };
        match key_event.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                do_trash(state, &mut sorted_children);
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                state.analyze_show_confirmation = false;
                state.status_message = "Deletion cancelled.".to_string();
            }
            KeyCode::Left | KeyCode::Right => {
                state.confirm_idx = 1 - state.confirm_idx;
            }
            KeyCode::Enter => {
                if state.confirm_idx == 0 {
                    do_trash(state, &mut sorted_children);
                } else {
                    state.analyze_show_confirmation = false;
                    state.status_message = "Deletion cancelled.".to_string();
                }
            }
            _ => {}
        }
        return;
    }

    match key_event.code {
        KeyCode::Esc => {
            state.screen = TuiScreen::Home;
            state.status_message = String::new();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if state.analyze_selected_idx > 0 {
                state.analyze_selected_idx -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if !sorted_children.is_empty() && state.analyze_selected_idx < sorted_children.len() - 1
            {
                state.analyze_selected_idx += 1;
            }
        }
        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
            if !sorted_children.is_empty() && state.analyze_selected_idx < sorted_children.len() {
                let target_idx = sorted_children[state.analyze_selected_idx];
                if state.analyze_arena[target_idx].is_dir {
                    state
                        .analyze_history
                        .push((state.analyze_current_dir_idx, state.analyze_selected_idx));
                    state.analyze_current_dir_idx = target_idx;
                    state.analyze_selected_idx = 0;
                    state.status_message = "Opened folder.".to_string();
                } else {
                    state.status_message = "Selected file is not a directory.".to_string();
                }
            }
        }
        KeyCode::Backspace | KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('u') => {
            if let Some((parent_idx, prev_selected_idx)) = state.analyze_history.pop() {
                state.analyze_current_dir_idx = parent_idx;
                state.analyze_selected_idx = prev_selected_idx;
                state.status_message = "Navigated up.".to_string();
            } else {
                state.status_message = "Already at root directory scan limit.".to_string();
            }
        }
        KeyCode::Char('d') | KeyCode::Char('x') => {
            if !sorted_children.is_empty() && state.analyze_selected_idx < sorted_children.len() {
                state.analyze_show_confirmation = true;
            } else {
                state.status_message = "Nothing selected to delete.".to_string();
            }
        }
        _ => {}
    }
}

fn handle_status_keys(state: &mut TuiState, key_event: KeyEvent) {
    let process_count = state.sys_top_processes.len();
    match key_event.code {
        KeyCode::Esc => {
            state.screen = TuiScreen::Home;
            state.status_message = String::new();
        }
        KeyCode::Up => {
            if process_count > 0 {
                if state.sys_process_cursor_idx > 0 {
                    state.sys_process_cursor_idx -= 1;
                } else {
                    state.sys_process_cursor_idx = process_count - 1;
                }
            }
        }
        KeyCode::Down => {
            if process_count > 0 {
                if state.sys_process_cursor_idx < process_count - 1 {
                    state.sys_process_cursor_idx += 1;
                } else {
                    state.sys_process_cursor_idx = 0;
                }
            }
        }
        KeyCode::Char('k') | KeyCode::Char('K') => {
            if process_count > 0 && state.sys_process_cursor_idx < process_count {
                let proc = &state.sys_top_processes[state.sys_process_cursor_idx];
                state.status_message =
                    format!("Killing process {} (PID: {})...", proc.command, proc.pid);
                let output = std::process::Command::new("kill")
                    .arg("-9")
                    .arg(proc.pid.to_string())
                    .output();
                match output {
                    Ok(out) if out.status.success() => {
                        state.status_message =
                            format!("Successfully killed process (PID: {}).", proc.pid);
                        state.update_system_stats();
                        if state.sys_process_cursor_idx >= state.sys_top_processes.len() {
                            state.sys_process_cursor_idx =
                                state.sys_top_processes.len().saturating_sub(1);
                        }
                    }
                    _ => {
                        let output_sudo = std::process::Command::new("sudo")
                            .arg("kill")
                            .arg("-9")
                            .arg(proc.pid.to_string())
                            .output();
                        match output_sudo {
                            Ok(out) if out.status.success() => {
                                state.status_message = format!(
                                    "Successfully killed process (PID: {}, via sudo).",
                                    proc.pid
                                );
                                state.update_system_stats();
                                if state.sys_process_cursor_idx >= state.sys_top_processes.len() {
                                    state.sys_process_cursor_idx =
                                        state.sys_top_processes.len().saturating_sub(1);
                                }
                            }
                            _ => {
                                state.status_message = format!(
                                    "Failed to kill process (PID: {}): Permission denied.",
                                    proc.pid
                                );
                            }
                        }
                    }
                }
            }
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            state.update_system_stats();
            state.status_message = "Telemetry stats refreshed.".to_string();
        }
        _ => {}
    }
}
