use crate::analyze::tree::{ArenaNode, build_disk_tree, delete_node_from_tree};
use crate::cleaner::trash::move_to_trash;
use crate::tui::view::{format_size, get_mascot_lines};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use std::path::Path;

pub struct AnalyzeTuiState {
    pub arena: Vec<ArenaNode>,
    pub current_dir_idx: usize,
    pub selected_idx: usize,          // Index in the *sorted* children list
    pub history: Vec<(usize, usize)>, // Stack of (dir_idx, selected_idx) to go back
    pub show_confirmation: bool,
    pub status_message: String,
    pub should_quit: bool,
    pub warnings: Vec<String>,
    pub tick: usize,
    pub start_time: std::time::Instant,
}

impl AnalyzeTuiState {
    pub fn new(arena: Vec<ArenaNode>, warnings: Vec<String>) -> Self {
        Self {
            arena,
            current_dir_idx: 0,
            selected_idx: 0,
            history: Vec::new(),
            show_confirmation: false,
            status_message:
                "Use arrows/jk to navigate │ Enter: Open │ Backspace/u: Up │ d: Trash │ q: Quit"
                    .to_string(),
            should_quit: false,
            warnings,
            tick: 0,
            start_time: std::time::Instant::now(),
        }
    }

    pub fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    /// Returns the children of the current directory sorted by size descending.
    pub fn get_sorted_children(&self) -> Vec<usize> {
        let mut children = self.arena[self.current_dir_idx].children.clone();
        children.sort_by(|&a, &b| self.arena[b].size_bytes.cmp(&self.arena[a].size_bytes));
        children
    }
}

pub fn run_analyze_tui(root: &Path) -> Result<()> {
    // 1. Scan directory and build tree
    println!("Scanning filesystem at {:?}...", root);
    let (arena, warnings) = build_disk_tree(root);

    // 2. Initialize terminal
    let mut terminal = ratatui::init();
    terminal.clear()?;

    let mut state = AnalyzeTuiState::new(arena, warnings);

    let res = run_loop(&mut terminal, &mut state);

    ratatui::restore();

    res
}

fn run_loop(terminal: &mut DefaultTerminal, state: &mut AnalyzeTuiState) -> Result<()> {
    while !state.should_quit {
        terminal.draw(|f| draw_analyze(state, f))?;

        if event::poll(std::time::Duration::from_millis(250))?
            && let Event::Key(key) = event::read()?
        {
            // Ctrl+C to force exit
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                state.should_quit = true;
                break;
            }

            handle_key(state, key.code)?;
        }
        state.tick();
    }
    Ok(())
}

fn handle_key(state: &mut AnalyzeTuiState, code: KeyCode) -> Result<()> {
    let sorted_children = state.get_sorted_children();

    if state.show_confirmation {
        match code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if !sorted_children.is_empty() && state.selected_idx < sorted_children.len() {
                    let target_idx = sorted_children[state.selected_idx];
                    let target_path = state.arena[target_idx].path.clone();

                    state.status_message = format!("Moving to trash: {:?}", target_path);
                    match move_to_trash(&target_path) {
                        Ok(_) => {
                            state.status_message =
                                format!("Successfully moved to trash: {}", target_path.display());
                            delete_node_from_tree(&mut state.arena, target_idx);

                            // Adjust selection index
                            if state.selected_idx > 0
                                && state.selected_idx
                                    >= state.arena[state.current_dir_idx].children.len()
                            {
                                state.selected_idx -= 1;
                            }
                        }
                        Err(e) => {
                            state.status_message = format!("Error: {}", e);
                        }
                    }
                }
                state.show_confirmation = false;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                state.show_confirmation = false;
                state.status_message = "Deletion cancelled.".to_string();
            }
            _ => {}
        }
        return Ok(());
    }

    match code {
        KeyCode::Char('q') | KeyCode::Esc => {
            state.should_quit = true;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if state.selected_idx > 0 {
                state.selected_idx -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if !sorted_children.is_empty() && state.selected_idx < sorted_children.len() - 1 {
                state.selected_idx += 1;
            }
        }
        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
            if !sorted_children.is_empty() && state.selected_idx < sorted_children.len() {
                let target_idx = sorted_children[state.selected_idx];
                if state.arena[target_idx].is_dir {
                    state
                        .history
                        .push((state.current_dir_idx, state.selected_idx));
                    state.current_dir_idx = target_idx;
                    state.selected_idx = 0;
                    state.status_message = "Opened folder.".to_string();
                } else {
                    state.status_message = "Selected file is not a directory.".to_string();
                }
            }
        }
        KeyCode::Backspace | KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('u') => {
            if let Some((parent_idx, prev_selected_idx)) = state.history.pop() {
                state.current_dir_idx = parent_idx;
                state.selected_idx = prev_selected_idx;
                state.status_message = "Navigated up.".to_string();
            } else {
                state.status_message = "Already at root directory scan limit.".to_string();
            }
        }
        KeyCode::Char('d') | KeyCode::Char('x') => {
            if !sorted_children.is_empty() && state.selected_idx < sorted_children.len() {
                state.show_confirmation = true;
            } else {
                state.status_message = "Nothing selected to delete.".to_string();
            }
        }
        _ => {}
    }

    Ok(())
}

fn draw_analyze(state: &mut AnalyzeTuiState, frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Header
            Constraint::Min(8),    // Body
            Constraint::Length(3), // Footer
        ])
        .split(area);

    // --- RENDER HEADER ---
    let current_path = &state.arena[state.current_dir_idx].path;
    let current_size = state.arena[state.current_dir_idx].size_bytes;

    // Draw header block with bottom border
    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));
    let header_inner = header_block.inner(chunks[0]);
    frame.render_widget(header_block, chunks[0]);

    // Split inner area horizontally
    let header_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(30), Constraint::Length(13)])
        .split(header_inner);

    let header_lines = vec![
        Line::from(vec![
            Span::styled(
                " Nibble Disk Analyzer ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("— Find & Reclaim Space", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled("Current Directory: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                current_path.to_string_lossy().to_string(),
                Style::default().fg(Color::White),
            ),
            Span::styled(" │ Total Size: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format_size(current_size),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ Warnings: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                state.warnings.len().to_string(),
                Style::default().fg(if state.warnings.is_empty() {
                    Color::Gray
                } else {
                    Color::Red
                }),
            ),
        ]),
    ];

    let header_widget = Paragraph::new(header_lines);
    frame.render_widget(header_widget, header_split[0]);

    // Render animated mascot (using telemetry dashboard style since it fits analyzer info)
    let elapsed_ms = state.start_time.elapsed().as_millis() as u64;
    let mascot_lines = get_mascot_lines(elapsed_ms, "telemetry", &crate::theme::NORD);
    let mascot_widget = Paragraph::new(mascot_lines).alignment(Alignment::Right);
    frame.render_widget(mascot_widget, header_split[1]);

    // --- RENDER BODY ---
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70), // Left: List of files/dirs
            Constraint::Percentage(30), // Right: Detailed Info
        ])
        .split(chunks[1]);

    let sorted_children = state.get_sorted_children();

    // Render list
    if sorted_children.is_empty() {
        let empty_widget = Paragraph::new("\n\n This folder is completely empty.")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(" Folder Contents "),
            );
        frame.render_widget(empty_widget, body_chunks[0]);
    } else {
        let parent_size = current_size.max(1);
        let items: Vec<ListItem> = sorted_children
            .iter()
            .enumerate()
            .map(|(idx, &child_idx)| {
                let child = &state.arena[child_idx];

                // Generate visual percent bar
                let percentage = (child.size_bytes as f64 / parent_size as f64) * 100.0;
                let bar_length = ((percentage / 10.0).round() as usize).min(10);
                let bar = format!(
                    "[{}{}] {:>3.0}%",
                    "■".repeat(bar_length),
                    " ".repeat(10 - bar_length),
                    percentage
                );

                let type_prefix = if child.is_dir { "/" } else { " " };
                let name_label = format!("{}{}", child.name, type_prefix);
                let size_label = format_size(child.size_bytes);

                let style = if idx == state.selected_idx {
                    Style::default().bg(Color::DarkGray).fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{:<8} ", size_label),
                        Style::default().fg(Color::Green),
                    ),
                    Span::styled(
                        format!("{:<18}  ", bar),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(name_label, style),
                ]))
            })
            .collect();

        let list_widget = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(" Files & Directories "),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        let mut list_state = ListState::default();
        list_state.select(Some(state.selected_idx));
        frame.render_stateful_widget(list_widget, body_chunks[0], &mut list_state);
    }

    // Render Right Panel: Info
    let right_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Selected Item Info ");
    if !sorted_children.is_empty() && state.selected_idx < sorted_children.len() {
        let child_idx = sorted_children[state.selected_idx];
        let child = &state.arena[child_idx];

        let mut info_lines = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    &child.name,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Type: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    if child.is_dir { "Directory" } else { "File" },
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("Size: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format_size(child.size_bytes),
                    Style::default().fg(Color::Green),
                ),
            ]),
        ];

        if child.is_dir {
            info_lines.push(Line::from(vec![
                Span::styled("Items: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    child.children.len().to_string(),
                    Style::default().fg(Color::White),
                ),
            ]));
        }

        info_lines.push(Line::from(""));
        info_lines.push(Line::from(Span::styled(
            "Full Path:",
            Style::default().fg(Color::DarkGray),
        )));
        info_lines.push(Line::from(Span::styled(
            child.path.to_string_lossy().to_string(),
            Style::default().fg(Color::Gray),
        )));

        let info_widget = Paragraph::new(info_lines)
            .block(right_block)
            .wrap(Wrap { trim: true });
        frame.render_widget(info_widget, body_chunks[1]);
    } else {
        let info_widget = Paragraph::new("\n No selection.")
            .block(right_block)
            .alignment(Alignment::Center);
        frame.render_widget(info_widget, body_chunks[1]);
    }

    // --- RENDER FOOTER ---
    let footer_text = Line::from(vec![Span::styled(
        format!(" {} ", state.status_message),
        Style::default().fg(Color::White),
    )]);

    let footer_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Line::from(vec![
            Span::styled(
                " Q ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Quit │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                " Arrows/JK ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Navigate │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                " Enter ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Open │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                " Backspace/U ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Up │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                " D/X ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Move to Trash",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]));

    let footer_widget = Paragraph::new(footer_text).block(footer_block);
    frame.render_widget(footer_widget, chunks[2]);

    // --- DRAW CONFIRMATION OVERLAY ---
    if state.show_confirmation {
        let popup_area = centered_rect(55, 30, area);
        frame.render_widget(Clear, popup_area);

        if let Some(&child_idx) = sorted_children.get(state.selected_idx) {
            let child = &state.arena[child_idx];
            let popup_lines = vec![
                Line::from(""),
                Line::from(vec![Span::styled(
                    "[!] CONFIRM DELETION",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from("Are you sure you want to move this item to trash?"),
                Line::from(vec![
                    Span::styled(
                        &child.name,
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!(" ({})", format_size(child.size_bytes))),
                ]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    " [y] Yes, move to Trash  │  [n] Cancel ",
                    Style::default().bg(Color::DarkGray).fg(Color::White),
                )]),
            ];

            let popup_widget = Paragraph::new(popup_lines)
                .block(
                    Block::default()
                        .title(" Safe Trash Confirmation ")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Red)),
                )
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            frame.render_widget(popup_widget, popup_area);
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
