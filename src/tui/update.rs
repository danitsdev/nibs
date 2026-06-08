use crate::cleaner::clean_findings;
use crate::cleaner::trash::{empty_trash_directory, list_trash_items, restore_trash_item};
use crate::tui::model::{TuiScreen, TuiState};
use crate::tui::view::format_size;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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
        TuiScreen::AppUninstallSelector => {
            handle_uninstall_selector_keys(state, key_event, terminal);
        }
        TuiScreen::AppUninstallList => {
            handle_uninstall_list_keys(state, key_event);
        }
        TuiScreen::Wizard => {
            handle_wizard_keys(state, key_event);
        }
        TuiScreen::Dashboard | TuiScreen::SmartClean => {
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
        TuiScreen::CleanComplete => {
            handle_clean_complete_keys(state, key_event);
        }
        TuiScreen::TrashManager => {
            handle_trash_manager_keys(state, key_event);
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
                    // Smart Clean — scan home directly
                    state.is_smart_clean = true;
                    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                    state.target_path = std::path::PathBuf::from(&home);
                    state.scope = crate::safety::ScanScope::from_path(&state.target_path);
                    let options = crate::scanner::ScanOptions::default();
                    crate::tui::start_background_scan(state, state.target_path.clone(), options);
                }
                1 => {
                    // Deep Clean — scan home with brute mode for broader detection
                    state.is_smart_clean = false;
                    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                    state.target_path = std::path::PathBuf::from(&home);
                    state.scope = crate::safety::ScanScope::from_path(&state.target_path);
                    let options = crate::scanner::ScanOptions {
                        brute: true,
                        min_age_days: 7,
                        min_size_bytes: 0,
                        detect_duplicates: true,
                        include_deep_rules: true,
                        ..Default::default()
                    };
                    crate::tui::start_background_scan(state, state.target_path.clone(), options);
                }
                2 => {
                    // Analyze Disk
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
                3 => {
                    // Apps & Leftovers
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
                4 => {
                    // Optimize System — startup, logs, package cache and safe performance fixes
                    state.screen = TuiScreen::Optimize;
                    state.opt_selected_indices.clear();
                    state.opt_cursor_idx = 0;
                    state.opt_in_progress = false;
                    state.opt_results.clear();
                    state.status_message =
                        "Space: toggle option │ O: run optimization │ Esc: back".to_string();
                }
                5 => {
                    // Trash manager — list, restore, empty
                    state.trash_items = list_trash_items();
                    state.trash_selected_idx = 0;
                    state.screen = TuiScreen::TrashManager;
                    state.status_message = format!(
                        "Arrows/jk: navigate │ r: Restore │ e: Empty all │ Esc: Back  ({} items in trash)",
                        state.trash_items.len(),
                    );
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
                        "Thank you for using Nibs. Press any key to quit.".to_string();
                }
                _ => {}
            }
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            state.screen = TuiScreen::Goodbye;
            state.start_time = std::time::Instant::now();
            state.status_message = "Thank you for using Nibs. Press any key to quit.".to_string();
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
                if state.shred {
                    state.delete_directly = false;
                    state.shred = false;
                } else if state.delete_directly {
                    state.delete_directly = false;
                    state.shred = true;
                } else {
                    state.delete_directly = true;
                    state.shred = false;
                }
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
            if crate::cleaner::clean_path(&p, state.delete_directly, state.shred).is_ok() {
                success_count += 1;
            }
        }
        state.status_message = if state.shred {
            format!(
                "Securely shredded {}/{} remnants for application '{}'.",
                success_count, total_to_trash, app.name
            )
        } else if state.delete_directly {
            format!(
                "Permanently deleted {}/{} remnants for application '{}'.",
                success_count, total_to_trash, app.name
            )
        } else {
            format!(
                "Cleaned {}/{} remnants for application '{}'.",
                success_count, total_to_trash, app.name
            )
        };
    }
    state.show_confirmation = false;
    state.screen = TuiScreen::Home;
}

fn do_trash_remnants(state: &mut TuiState) {
    let mut success_count = 0;
    for &idx in &state.selected_remnants {
        if let Some((path, _)) = state.app_remnants.get(idx)
            && crate::cleaner::clean_path(path, state.delete_directly, state.shred).is_ok()
        {
            success_count += 1;
        }
    }
    state.status_message = if state.shred {
        format!(
            "Securely shredded {}/{} remnants.",
            success_count,
            state.selected_remnants.len()
        )
    } else if state.delete_directly {
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

    match clean_findings(&to_clean, state.dry_run, state.delete_directly, state.shred) {
        Ok(results) => {
            let total_bytes: u64 = results.iter().map(|r| r.bytes_freed).sum();
            let mode = if state.dry_run {
                "Simulated"
            } else if state.shred {
                "Securely Shredded"
            } else if state.delete_directly {
                "Permanently Deleted"
            } else {
                "System Trash"
            };

            state.cleaned_bytes = total_bytes;
            state.cleaned_count = results.len();
            state.cleaned_mode = mode.to_string();

            for idx in indices {
                state.findings.remove(idx);
            }
            state.selected_findings.clear();
            state.selected_idx = 0;
            state.clamp_selected_to_filter();

            state.screen = TuiScreen::CleanComplete;
            state.status_message = format!(
                "Reclaimed {} from {} items ({})",
                format_size(total_bytes),
                results.len(),
                mode,
            );
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

fn handle_clean_complete_keys(state: &mut TuiState, _key_event: KeyEvent) {
    // Any key returns to Home
    state.screen = TuiScreen::Home;
    state.status_message = format!(
        "Cleaned {} items, {} reclaimed ({}). Back to menu.",
        state.cleaned_count,
        format_size(state.cleaned_bytes),
        state.cleaned_mode,
    );
}

fn handle_trash_manager_keys(state: &mut TuiState, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if !state.trash_items.is_empty() {
                state.trash_selected_idx = state.trash_selected_idx.saturating_sub(1);
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let max = state.trash_items.len().saturating_sub(1);
            if state.trash_selected_idx < max {
                state.trash_selected_idx += 1;
            }
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            // Restore selected item
            if let Some(item) = state.trash_items.get(state.trash_selected_idx) {
                match restore_trash_item(item) {
                    Ok(()) => {
                        let name = item
                            .original_path
                            .as_ref()
                            .and_then(|p| p.file_name())
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        state.status_message = format!("Restored '{}' to original location.", name);
                        state.trash_items.remove(state.trash_selected_idx);
                        if state.trash_selected_idx >= state.trash_items.len()
                            && !state.trash_items.is_empty()
                        {
                            state.trash_selected_idx = state.trash_items.len() - 1;
                        }
                    }
                    Err(e) => {
                        state.status_message = format!("Failed to restore item: {}", e);
                    }
                }
            }
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            // Delete selected item permanently from trash
            if let Some(item) = state.trash_items.get(state.trash_selected_idx) {
                match crate::cleaner::trash::delete_trash_item(item) {
                    Ok(()) => {
                        let name = item
                            .trash_path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        state.status_message =
                            format!("Permanently deleted '{}' from trash.", name);
                        state.trash_items.remove(state.trash_selected_idx);
                        if state.trash_selected_idx >= state.trash_items.len()
                            && !state.trash_items.is_empty()
                        {
                            state.trash_selected_idx = state.trash_items.len() - 1;
                        }
                    }
                    Err(e) => {
                        state.status_message = format!("Failed to delete trash item: {}", e);
                    }
                }
            }
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            // Empty all trash
            let Some(home) = std::env::var("HOME").ok().map(PathBuf::from) else {
                state.status_message = "Could not determine home directory.".to_string();
                return;
            };
            let trash_root = home.join(".local/share/Trash");
            match empty_trash_directory(&trash_root) {
                Ok(()) => {
                    state.status_message = "Trash emptied permanently.".to_string();
                    state.trash_items.clear();
                    state.trash_selected_idx = 0;
                }
                Err(e) => {
                    state.status_message = format!("Failed to empty trash: {}", e);
                }
            }
        }
        KeyCode::Esc => {
            state.screen = TuiScreen::Home;
            state.status_message = "Returned to menu.".to_string();
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
                state.opt_cursor_idx = 5;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if state.opt_cursor_idx < 5 {
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

    let selected: Vec<usize> = state.opt_selected_indices.iter().copied().collect();
    for idx in selected {
        match idx {
            0 => {
                state.opt_results.push("Flushing DNS caches...".to_string());
                let mut flushed = false;
                if std::process::Command::new("resolvectl")
                    .arg("flush-caches")
                    .output()
                    .is_ok()
                {
                    state
                        .opt_results
                        .push("  -> Ran 'resolvectl flush-caches' successfully.".to_string());
                    flushed = true;
                } else if std::process::Command::new("systemd-resolve")
                    .arg("--flush-caches")
                    .output()
                    .is_ok()
                {
                    state.opt_results.push(
                        "  -> Ran 'systemd-resolve --flush-caches' successfully.".to_string(),
                    );
                    flushed = true;
                }
                if !flushed {
                    state.opt_results.push(
                        "  -> No systemd resolver commands found or failed to execute.".to_string(),
                    );
                }
            }
            1 => {
                state
                    .opt_results
                    .push("Rebuilding Font & MIME caches...".to_string());
                state
                    .opt_results
                    .push("  Running 'fc-cache -f' for fonts...".to_string());
                let fc_output = std::process::Command::new("fc-cache").arg("-f").output();
                match fc_output {
                    Ok(out) if out.status.success() => {
                        state
                            .opt_results
                            .push("    -> Font cache rebuilt successfully.".to_string());
                    }
                    _ => {
                        state
                            .opt_results
                            .push("    -> fc-cache execution skipped or failed.".to_string());
                    }
                }

                if let Some(home) = std::env::var("HOME").ok().map(PathBuf::from) {
                    let mime_dir = home.join(".local/share/mime");
                    if mime_dir.exists() {
                        state.opt_results.push(
                            "  Running 'update-mime-database' for user MIME associations..."
                                .to_string(),
                        );
                        let mime_output = std::process::Command::new("update-mime-database")
                            .arg(&mime_dir)
                            .output();
                        match mime_output {
                            Ok(out) if out.status.success() => {
                                state
                                    .opt_results
                                    .push("    -> MIME database updated successfully.".to_string());
                            }
                            _ => {
                                state.opt_results.push(
                                    "    -> update-mime-database execution skipped or failed."
                                        .to_string(),
                                );
                            }
                        }
                    } else {
                        state.opt_results.push(
                            "  -> Skipped MIME rebuild: ~/.local/share/mime does not exist."
                                .to_string(),
                        );
                    }
                }
            }
            2 => {
                state
                    .opt_results
                    .push("Vacuuming SQLite databases...".to_string());
                let (count, freed) = vacuum_sqlite_databases(state);
                state.opt_results.push(format!(
                    "  -> Successfully vacuumed {} SQLite databases (freed {}).",
                    count,
                    format_size(freed)
                ));
            }
            3 => {
                state
                    .opt_results
                    .push("Cleaning package manager cache...".to_string());
                let mut found = false;
                if std::process::Command::new("apt-get")
                    .arg("--version")
                    .output()
                    .is_ok()
                {
                    state
                        .opt_results
                        .push("Running 'sudo apt-get clean'...".to_string());
                    let output = std::process::Command::new("sudo")
                        .arg("apt-get")
                        .arg("clean")
                        .output();
                    match output {
                        Ok(out) if out.status.success() => {
                            state
                                .opt_results
                                .push("  -> Successfully cleaned APT package cache.".to_string());
                        }
                        _ => {
                            state
                                .opt_results
                                .push("  -> Failed to run apt-get clean.".to_string());
                        }
                    }
                    found = true;
                }
                if std::process::Command::new("dnf")
                    .arg("--version")
                    .output()
                    .is_ok()
                {
                    state
                        .opt_results
                        .push("Running 'sudo dnf clean all'...".to_string());
                    let output = std::process::Command::new("sudo")
                        .arg("dnf")
                        .arg("clean")
                        .arg("all")
                        .output();
                    match output {
                        Ok(out) if out.status.success() => {
                            state
                                .opt_results
                                .push("  -> Successfully cleaned DNF package cache.".to_string());
                        }
                        _ => {
                            state
                                .opt_results
                                .push("  -> Failed to run dnf clean.".to_string());
                        }
                    }
                    found = true;
                }
                if std::process::Command::new("pacman")
                    .arg("--version")
                    .output()
                    .is_ok()
                {
                    state
                        .opt_results
                        .push("Running 'sudo pacman -Sc --noconfirm'...".to_string());
                    let output = std::process::Command::new("sudo")
                        .arg("pacman")
                        .arg("-Sc")
                        .arg("--noconfirm")
                        .output();
                    match output {
                        Ok(out) if out.status.success() => {
                            state
                                .opt_results
                                .push("  -> Successfully cleaned Pacman cache.".to_string());
                        }
                        _ => {
                            state
                                .opt_results
                                .push("  -> Failed to run pacman clean.".to_string());
                        }
                    }
                    found = true;
                }
                if !found {
                    state.opt_results.push(
                        "  -> No supported package manager found to clean cache.".to_string(),
                    );
                }
            }
            4 => {
                state
                    .opt_results
                    .push("Cleaning orphan packages...".to_string());
                let (count, _) = run_orphan_cleanup(state);
                state
                    .opt_results
                    .push(format!("  -> Removed {} orphaned packages.", count));
            }
            5 => {
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

fn vacuum_sqlite_databases(state: &mut TuiState) -> (usize, u64) {
    let home = match std::env::var("HOME").ok().map(PathBuf::from) {
        Some(h) => h,
        None => return (0, 0),
    };

    let target_dirs = vec![
        home.join(".mozilla"),
        home.join(".config/google-chrome"),
        home.join(".config/chromium"),
        home.join(".config/BraveSoftware"),
        home.join(".config/microsoft-edge"),
        home.join(".config/Code"),
    ];

    let mut vacuumed_count = 0;
    let mut total_freed_bytes = 0;

    for dir in target_dirs {
        if !dir.exists() {
            continue;
        }

        let walker = walkdir::WalkDir::new(dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file());

        for entry in walker {
            let path = entry.path();
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if ext == "sqlite" || ext == "sqlite3" || ext == "db" || ext == "db3" || ext == "vscdb"
            {
                let orig_size = match path.metadata() {
                    Ok(m) => m.len(),
                    Err(_) => continue,
                };
                if orig_size == 0 {
                    continue;
                }

                let output = std::process::Command::new("sqlite3")
                    .arg(path)
                    .arg("VACUUM;")
                    .output();

                if let Ok(out) = output
                    && out.status.success()
                {
                    let new_size = path.metadata().map(|m| m.len()).unwrap_or(orig_size);
                    if new_size < orig_size {
                        let freed = orig_size - new_size;
                        if freed > 0 {
                            total_freed_bytes += freed;
                            vacuumed_count += 1;
                            state.opt_results.push(format!(
                                "  Vacuumed {:?}: reclaimed {}",
                                path.file_name().unwrap_or_default(),
                                format_size(freed)
                            ));
                        }
                    }
                }
            }
        }
    }

    (vacuumed_count, total_freed_bytes)
}

fn run_orphan_cleanup(state: &mut TuiState) -> (usize, u64) {
    let mut count = 0;
    let freed = 0;

    let mut pkg_mgr = "";
    if std::process::Command::new("apt-get")
        .arg("--version")
        .output()
        .is_ok()
    {
        pkg_mgr = "apt";
    } else if std::process::Command::new("dnf")
        .arg("--version")
        .output()
        .is_ok()
    {
        pkg_mgr = "dnf";
    } else if std::process::Command::new("pacman")
        .arg("--version")
        .output()
        .is_ok()
    {
        pkg_mgr = "pacman";
    }

    state
        .opt_results
        .push(format!("Detected package manager: {}", pkg_mgr));

    match pkg_mgr {
        "apt" => {
            state
                .opt_results
                .push("Running 'sudo apt-get autoremove -y'...".to_string());
            let output = std::process::Command::new("sudo")
                .arg("apt-get")
                .arg("autoremove")
                .arg("-y")
                .output();
            match output {
                Ok(out) if out.status.success() => {
                    state
                        .opt_results
                        .push("  -> Successfully removed orphaned APT packages.".to_string());
                    count += 1;
                }
                _ => {
                    state
                        .opt_results
                        .push("  -> Failed to run apt-get autoremove.".to_string());
                }
            }
        }
        "dnf" => {
            state
                .opt_results
                .push("Running 'sudo dnf autoremove -y'...".to_string());
            let output = std::process::Command::new("sudo")
                .arg("dnf")
                .arg("autoremove")
                .arg("-y")
                .output();
            match output {
                Ok(out) if out.status.success() => {
                    state
                        .opt_results
                        .push("  -> Successfully removed orphaned DNF packages.".to_string());
                    count += 1;
                }
                _ => {
                    state
                        .opt_results
                        .push("  -> Failed to run dnf autoremove.".to_string());
                }
            }
        }
        "pacman" => {
            let out_orphans = std::process::Command::new("pacman").arg("-Qtdq").output();
            if let Ok(out) = out_orphans {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let packages: Vec<&str> = stdout
                    .lines()
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();
                if packages.is_empty() {
                    state
                        .opt_results
                        .push("  -> No orphaned packages found.".to_string());
                } else {
                    state.opt_results.push(format!(
                        "Running 'sudo pacman -Rns {} --noconfirm'...",
                        packages.join(" ")
                    ));
                    let output = std::process::Command::new("sudo")
                        .arg("pacman")
                        .arg("-Rns")
                        .args(&packages)
                        .arg("--noconfirm")
                        .output();
                    match output {
                        Ok(out) if out.status.success() => {
                            state.opt_results.push(
                                "  -> Successfully removed orphaned Pacman packages.".to_string(),
                            );
                            count += packages.len();
                        }
                        _ => {
                            state
                                .opt_results
                                .push("  -> Failed to remove packages.".to_string());
                        }
                    }
                }
            } else {
                state
                    .opt_results
                    .push("  -> Failed to query orphaned packages.".to_string());
            }
        }
        _ => {
            state
                .opt_results
                .push("  -> No supported package manager found to clean orphans.".to_string());
        }
    }
    (count, freed)
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

                state.status_message = if state.shred {
                    format!("Securely shredding: {:?}", target_path)
                } else if state.delete_directly {
                    format!("Permanently deleting: {:?}", target_path)
                } else {
                    format!("Moving to trash: {:?}", target_path)
                };
                match crate::cleaner::clean_path(&target_path, state.delete_directly, state.shred) {
                    Ok(_) => {
                        state.status_message = if state.shred {
                            format!("Successfully shredded: {}", target_path.display())
                        } else if state.delete_directly {
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
