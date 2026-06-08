use crate::findings::RiskLevel;
use crate::theme::NibsTheme;
use crate::tui::model::{TuiScreen, TuiState};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

const MASCOT_WIDTH: u16 = 12;
const MASCOT_WIDTH_USIZE: usize = MASCOT_WIDTH as usize;
type MascotFrame = [&'static str; 3];

fn mascot_style(c: char, theme: &NibsTheme) -> Style {
    match c {
        '(' | ')' | '_' | '[' | ']' | '~' | 'W' => Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD),
        '|' | '/' | '\\' => Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD),
        '#' => Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
        'o' | 'O' | '-' | '^' | '<' | '>' => Style::default()
            .fg(Color::Reset)
            .add_modifier(Modifier::BOLD),
        '.' | '*' | '!' => Style::default()
            .fg(theme.warning)
            .add_modifier(Modifier::BOLD),
        '+' => Style::default()
            .fg(theme.error)
            .add_modifier(Modifier::BOLD),
        _ => Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    }
}

/// Helper function to format sizes in bytes into human-readable strings.
pub fn format_size(bytes: u64) -> String {
    let kib = 1024.0;
    let mib = kib * 1024.0;
    let gib = mib * 1024.0;

    if bytes as f64 >= gib {
        format!("{:.2} GiB", bytes as f64 / gib)
    } else if bytes as f64 >= mib {
        format!("{:.2} MiB", bytes as f64 / mib)
    } else if bytes as f64 >= kib {
        format!("{:.2} KiB", bytes as f64 / kib)
    } else {
        format!("{} B", bytes)
    }
}

/// Dynamic fun analogy for reclaimed space.
pub fn get_fun_analogy(bytes: u64) -> String {
    if bytes == 0 {
        return "equivalent to nothing (yet!)".to_string();
    }
    const DOOM: u64 = 2_300_000; // 2.3 MB
    const MOVIE: u64 = 1_500_000_000; // 1.5 GB
    const SONG: u64 = 5_000_000; // 5 MB
    const GTA: u64 = 100_000_000_000; // 100 GB

    if bytes >= GTA {
        let count = bytes as f64 / GTA as f64;
        format!("{:.1} installs of GTA V (AAA game)", count)
    } else if bytes >= MOVIE {
        let count = bytes as f64 / MOVIE as f64;
        format!(
            "{:.1} full HD movies (or {:.0} copies of DOOM 1993!)",
            count,
            bytes as f64 / DOOM as f64
        )
    } else if bytes >= SONG {
        let count = bytes as f64 / SONG as f64;
        format!(
            "{:.0} MP3 songs (or {:.0} copies of DOOM!)",
            count,
            bytes as f64 / DOOM as f64
        )
    } else {
        let count = bytes as f64 / DOOM as f64;
        if count >= 1.0 {
            format!("{:.1} copies of retro DOOM 1993!", count)
        } else {
            format!("{:.0} digital Kindle books!", bytes as f64 / 2_000_000.0)
        }
    }
}

fn get_home_disk_snapshot() -> Option<(String, u64, u64, u64)> {
    let output = Command::new("df")
        .arg("-B1")
        .arg("--output=target,size,used,avail")
        .arg(".")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let target = parts[0].to_string();
        let total = parts[1].parse::<u64>().ok()?;
        let used = parts[2].parse::<u64>().ok()?;
        let avail = parts[3].parse::<u64>().ok()?;
        if total > 0 {
            return Some((target, total, used, avail));
        }
    }

    None
}

fn get_user_trash_snapshot() -> (Option<PathBuf>, usize, u64) {
    let Some(home) = std::env::var("HOME").ok().map(PathBuf::from) else {
        return (None, 0, 0);
    };

    let trash_root = home.join(".local/share/Trash");
    let files_dir = trash_root.join("files");
    if !files_dir.exists() {
        return (Some(trash_root), 0, 0);
    }

    let mut item_count = 0;
    let mut total_bytes: u64 = 0;
    if let Ok(entries) = std::fs::read_dir(&files_dir) {
        for entry in entries.filter_map(Result::ok) {
            item_count += 1;
            let path = entry.path();
            if path.is_dir() {
                for nested in WalkDir::new(&path).into_iter().filter_map(Result::ok) {
                    if let Ok(meta) = nested.metadata()
                        && meta.is_file()
                    {
                        total_bytes = total_bytes.saturating_add(meta.len());
                    }
                }
            } else if let Ok(meta) = entry.metadata()
                && meta.is_file()
            {
                total_bytes = total_bytes.saturating_add(meta.len());
            }
        }
    }

    (Some(trash_root), item_count, total_bytes)
}

fn mascot_frame(elapsed_ms: u64, state_type: &str) -> MascotFrame {
    match state_type {
        "search" | "scanning" | "telemetry" => match (elapsed_ms / 320) % 4 {
            0 => [r"      ()_()", r"     -(O.o)-", r"#---- (_W_)~"],
            1 => [r"      ()_()", r"     -(o.O)-", r"#---- (_W_)~"],
            2 => [r"      ()_()", r"     -(O.o)-", r"#---- (_W_)~"],
            _ => [r"      ()_()", r"     -(o.O)-", r"#---- (_W_)~"],
        },
        "happy" | "wave" => match (elapsed_ms / 500) % 6 {
            4 | 5 => [r"      ()_()", r"     -(-.-)-", r"#---- (_W_)~"],
            _ => [r"      ()_()", r"     -(o.o)-", r"#---- (_W_)~"],
        },
        "sweeping" | "sweep" => match (elapsed_ms / 260) % 4 {
            0 => [r"  /   ()_()", r" /   -(o.o)-", r"#     (_W_)~"],
            1 => [r"  |   ()_()", r"  |  -(o.o)-", r"  #   (_W_)~"],
            2 => [r"  \   ()_()", r"   \ -(o.o)-", r"    # (_W_)~"],
            _ => [r"  |   ()_()", r"  |  -(o.o)-", r"  #   (_W_)~"],
        },
        "celebrate" => match (elapsed_ms / 180) % 4 {
            0 => [r"  /   ()_()", r" /   -(^.^)-", r"#     (_W_)~"],
            1 => [r"  |   ()_()", r"  |  -(^.^)-", r"  #   (_W_)~"],
            2 => [r"  \   ()_()", r"   \ -(^.^)-", r"    # (_W_)~"],
            _ => [r"  |   ()_()", r"  |  -(^.^)-", r"  #   (_W_)~"],
        },
        "box" => [r"      ()_()", r"     -(o.o)-", r"#---- [_W_]~"],
        "doc" => [r"      ()_()", r"     -(o.o)-", r"#---- (_W_)~"],
        _ => match elapsed_ms % 4000 {
            3000..=3999 => [r"      ()_()", r"     -(-.-)-", r"#---- (_W_)~"],
            _ => [r"      ()_()", r"     -(o.o)-", r"#---- (_W_)~"],
        },
    }
}

fn render_mascot_with_margins(frame: &mut Frame, area: Rect, mascot_lines: Vec<Line<'static>>) {
    let top_margin = if area.height >= 9 {
        (area.height.saturating_sub(3)) / 2
    } else if area.height >= 7 {
        2
    } else if area.height >= 4 {
        1
    } else {
        0
    };

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_margin),
            Constraint::Length(3), // Mascot height
            Constraint::Min(0),    // Bottom margin
        ])
        .split(area);

    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(MASCOT_WIDTH), // Mascot width (12)
            Constraint::Length(3),            // Right margin
        ])
        .split(v_chunks[1]);

    frame.render_widget(Paragraph::new(mascot_lines), h_chunks[1]);
}

pub fn get_mascot_lines(
    elapsed_ms: u64,
    state_type: &str,
    theme: &NibsTheme,
) -> Vec<Line<'static>> {
    mascot_frame(elapsed_ms, state_type)
        .into_iter()
        .map(|row| {
            let mut spans = Vec::new();
            let row_width = row.chars().count();
            let padding = MASCOT_WIDTH_USIZE.saturating_sub(row_width);
            for c in row.chars().chain(std::iter::repeat_n(' ', padding)) {
                spans.push(Span::styled(c.to_string(), mascot_style(c, theme)));
            }
            Line::from(spans)
        })
        .collect()
}

fn draw_settings(state: &TuiState, frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Header
            Constraint::Min(10),   // Options and details
            Constraint::Length(3), // Footer Help
        ])
        .split(area);

    let primary = state.theme.primary;

    // --- RENDER HEADER ---
    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));
    let header_inner = header_block.inner(chunks[0]);
    frame.render_widget(header_block, chunks[0]);

    let header_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(30), Constraint::Length(MASCOT_WIDTH + 4)])
        .split(header_inner);

    let header_text = vec![
        Line::from(vec![
            Span::styled(
                " Settings & Preferences ",
                Style::default()
                    .fg(state.theme.ink)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "— Customize cleanup actions, themes, and behavior",
                Style::default().fg(state.theme.ink),
            ),
        ]),
        Line::from(vec![Span::styled(
            "Configure how Nibs performs cleaning actions and personalize your UI theme.",
            Style::default().fg(Color::DarkGray),
        )]),
    ];
    frame.render_widget(Paragraph::new(header_text), header_split[0]);

    let elapsed_ms = state.start_time.elapsed().as_millis() as u64;
    let mascot_lines = get_mascot_lines(elapsed_ms, "happy", state.theme);
    render_mascot_with_margins(frame, header_split[1], mascot_lines);

    // --- RENDER BODY ---
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Left: Settings list
            Constraint::Percentage(50), // Right: Setting description card
        ])
        .split(chunks[1]);

    // Left Panel: Options list
    let left_block = Block::default()
        .title(" Options ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(primary));

    let option_items = [
        (
            "Cleanup Method",
            if state.shred {
                "Securely Shred (Zero Overwrite)"
            } else if state.delete_directly {
                "Delete Directly (Permanent)"
            } else {
                "Move to Trash (Standard)"
            },
        ),
        ("Theme", state.theme.name),
    ];

    let items: Vec<ListItem> = option_items
        .iter()
        .enumerate()
        .map(|(idx, (name, val))| {
            let is_selected = idx == state.settings_cursor_idx;
            let prefix = if is_selected { "● " } else { "○ " };

            let style = if is_selected {
                Style::default()
                    .bg(state.theme.select_bg)
                    .fg(state.theme.select_fg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(state.theme.ink)
            };

            let prefix_style = if is_selected {
                Style::default()
                    .fg(state.theme.select_fg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(primary).add_modifier(Modifier::BOLD)
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(prefix, prefix_style),
                    Span::styled(format!("{}: ", name), style),
                    Span::styled(
                        *val,
                        if is_selected {
                            style
                        } else {
                            Style::default()
                                .fg(state.theme.accent)
                                .add_modifier(Modifier::BOLD)
                        },
                    ),
                ]),
                Line::from(""),
            ])
        })
        .collect();

    frame.render_widget(List::new(items).block(left_block), body_chunks[0]);

    // Right Panel: Details panel
    let right_block = Block::default()
        .title(" Description ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let details = match state.settings_cursor_idx {
        0 => vec![
            Line::from(""),
            Line::from(Span::styled(
                "Cleanup Method",
                Style::default()
                    .fg(state.theme.ink)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(
                "Choose whether Nibs should move files to standard system trash, delete them permanently, or securely shred them.",
            ),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  Move to Trash (Default): ",
                    Style::default()
                        .fg(state.theme.success)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Safely moves files to standard system Trash. Highly recommended for safety.",
                    Style::default().fg(state.theme.ink),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  Delete Directly: ",
                    Style::default()
                        .fg(state.theme.error)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Deletes files permanently using file system unlink. Useful to bypass Trash limits.",
                    Style::default().fg(state.theme.ink),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  Securely Shred: ",
                    Style::default()
                        .fg(state.theme.warning)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Overwrites files with zero bytes and flushes cache to disk before unlinking, making forensic recovery extremely difficult.",
                    Style::default().fg(state.theme.ink),
                ),
            ]),
        ],
        1 => vec![
            Line::from(""),
            Line::from(Span::styled(
                "Color Theme",
                Style::default()
                    .fg(state.theme.ink)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Customize Nibs's color scheme. Choose from built-in popular themes."),
            Line::from(""),
            Line::from("Available themes (cycle with Space/Enter):"),
            Line::from(""),
            Line::from(Span::styled(
                format!("  {} available themes", crate::theme::ALL_THEMES.len()),
                Style::default().fg(state.theme.ink),
            )),
            Line::from(Span::styled(
                "  The 'System' theme uses your terminal's native colors.",
                Style::default().fg(state.theme.ink),
            )),
        ],
        _ => vec![],
    };

    frame.render_widget(
        Paragraph::new(details)
            .block(right_block)
            .wrap(Wrap { trim: true }),
        body_chunks[1],
    );

    // --- RENDER FOOTER HELP ---
    let footer_text = Line::from(vec![
        Span::styled(
            " ↑/↓ or j/k ",
            Style::default().fg(primary).add_modifier(Modifier::BOLD),
        ),
        Span::styled("Navigate Options │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " Space / Enter ",
            Style::default().fg(primary).add_modifier(Modifier::BOLD),
        ),
        Span::styled("Toggle/Cycle │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " Esc / Q ",
            Style::default()
                .fg(state.theme.error)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Save & Back to Menu", Style::default().fg(Color::DarkGray)),
    ]);
    let footer_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(Paragraph::new(footer_text).block(footer_block), chunks[2]);
}

pub fn draw(state: &mut TuiState, frame: &mut Frame) {
    match state.screen {
        TuiScreen::Home | TuiScreen::HomeConfirmTrash => draw_home(state, frame),
        TuiScreen::Wizard => draw_wizard(state, frame),
        TuiScreen::Dashboard | TuiScreen::SmartClean => draw_dashboard(state, frame),
        TuiScreen::AppUninstallSelector => draw_app_uninstall_selector(state, frame),
        TuiScreen::AppUninstallList => draw_app_uninstall_list(state, frame),
        TuiScreen::Scanning => draw_scanning(state, frame),
        TuiScreen::Optimize => draw_optimize(state, frame),
        TuiScreen::Analyze => draw_analyze_integrated(state, frame),
        TuiScreen::Status => draw_status_integrated(state, frame),
        TuiScreen::Settings => draw_settings(state, frame),
        TuiScreen::Goodbye => draw_goodbye(state, frame),
        TuiScreen::CleanComplete => draw_clean_complete(state, frame),
        TuiScreen::TrashManager => draw_trash_manager(state, frame),
    }
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;

    fn mascot_rows(state_type: &str, elapsed_ms: u64) -> Vec<String> {
        get_mascot_lines(elapsed_ms, state_type, &crate::theme::NORD)
            .into_iter()
            .map(|line| {
                line.spans
                    .into_iter()
                    .map(|span| span.content.into_owned())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .collect()
    }

    #[test]
    fn mascot_frames_keep_a_stable_ascii_silhouette() {
        let states = [
            "search",
            "scanning",
            "telemetry",
            "happy",
            "wave",
            "sweeping",
            "sweep",
            "celebrate",
            "box",
            "doc",
            "unknown",
        ];
        let elapsed_values = [0, 260, 300, 560, 840, 1_200, 3_700, 4_200];

        for state in states {
            for elapsed_ms in elapsed_values {
                let rows = mascot_rows(state, elapsed_ms);
                assert_eq!(rows.len(), 3, "{state} at {elapsed_ms}ms");

                for row in &rows {
                    assert_eq!(
                        row.chars().count(),
                        MASCOT_WIDTH_USIZE,
                        "{state} at {elapsed_ms}ms rendered as {row:?}"
                    );
                    if !matches!(state, "sweeping" | "sweep" | "celebrate") {
                        assert!(
                            !row.contains('/') && !row.contains('\\'),
                            "{state} at {elapsed_ms}ms has a diagonal limb/tool in {row:?}"
                        );
                    }
                }

                let face_row = &rows[1];
                assert!(
                    face_row.contains("-(") && face_row.contains(")-"),
                    "{state} at {elapsed_ms}ms moved or removed the fixed whiskers: {face_row:?}"
                );

                let head_idx = rows[0]
                    .find("()_()")
                    .expect("mascot frame should include ears/head");
                let face_idx = face_row
                    .find('(')
                    .expect("mascot frame should include a face");
                let body_idx = rows[2]
                    .find("_W_")
                    .expect("mascot frame should include the body")
                    .saturating_sub(1);
                assert_eq!(
                    head_idx, face_idx,
                    "{state} at {elapsed_ms}ms has a crooked head: {rows:?}"
                );
                assert_eq!(
                    body_idx, face_idx,
                    "{state} at {elapsed_ms}ms has a misaligned body: {rows:?}"
                );

                if matches!(state, "sweeping" | "sweep" | "celebrate") {
                    let face_idx = face_row
                        .find("-(")
                        .expect("sweeping frame should include the mascot face");
                    for marker in ["|", "/", "\\"] {
                        if let Some(tool_idx) = face_row.find(marker) {
                            assert!(
                                tool_idx < face_idx,
                                "{state} at {elapsed_ms}ms attached the broom to the mascot: {face_row:?}"
                            );
                        }
                    }

                    let broom_base_idx = rows[2]
                        .find('#')
                        .expect("animated broom should include a bristle marker");
                    if face_row.contains('/') {
                        assert_eq!(
                            broom_base_idx, 0,
                            "{state} at {elapsed_ms}ms has an incoherent left broom base: {rows:?}"
                        );
                    } else if face_row.contains('|') {
                        assert_eq!(
                            broom_base_idx, 2,
                            "{state} at {elapsed_ms}ms has an off-center vertical broom base: {rows:?}"
                        );
                    } else if face_row.contains('\\') {
                        assert_eq!(
                            broom_base_idx, 4,
                            "{state} at {elapsed_ms}ms has an incoherent right broom base: {rows:?}"
                        );
                    }
                }

                let body_row = &rows[2];
                if let Some(tail_idx) = body_row.find('~') {
                    let body_idx = body_row
                        .find("_W_")
                        .expect("tail frame should still include the mascot body");
                    assert!(
                        tail_idx > body_idx,
                        "{state} at {elapsed_ms}ms moved the tail before the body: {body_row:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn broom_bristles_do_not_share_the_handle_color() {
        assert_ne!(
            mascot_style('#', &crate::theme::NORD).fg,
            mascot_style('|', &crate::theme::NORD).fg
        );
        assert_eq!(
            mascot_style('/', &crate::theme::NORD).fg,
            mascot_style('|', &crate::theme::NORD).fg
        );
        assert_eq!(
            mascot_style('\\', &crate::theme::NORD).fg,
            mascot_style('|', &crate::theme::NORD).fg
        );
    }
}

fn draw_home(state: &mut TuiState, frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // ASCII Banner Header
            Constraint::Min(10),   // Menu Selection and Info Pane
            Constraint::Length(3), // Footer Help
        ])
        .split(area);

    // --- RENDER ASCII BANNER HEADER ---
    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray)); // Slate Gray border
    let header_inner = header_block.inner(chunks[0]);
    frame.render_widget(header_block, chunks[0]);

    let banner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(40), Constraint::Length(MASCOT_WIDTH + 4)])
        .split(header_inner);

    let primary = state.theme.primary;
    let banner_text = vec![
        Line::from(""), // Skip 1 from top
        Line::from(Span::styled(
            "█▄ █ █ █▄▄ █▄▄ █   ██▀",
            Style::default().fg(primary).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "█ ▀█ █ █▄█ █▄█ █   █▄▄",
            Style::default().fg(primary).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "▀  ▀ ▀ ▀▀▀ ▀▀▀ ▀▀▀ ▀▀▀",
            Style::default().fg(primary).add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                format!("v{}", env!("CARGO_PKG_VERSION")),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                " │ Safe Linux Terminal Cleaner",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ];
    frame.render_widget(Paragraph::new(banner_text), banner_layout[0]);

    // Mascot
    let elapsed_ms = state.start_time.elapsed().as_millis() as u64;
    let mascot_lines = get_mascot_lines(elapsed_ms, "happy", state.theme);
    render_mascot_with_margins(frame, banner_layout[1], mascot_lines);

    // --- RENDER BODY LAYOUT ---
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(55), // Menu Selection
            Constraint::Percentage(45), // Live Telemetry & Info
        ])
        .split(chunks[1]);

    // Left Panel: Menu picker
    let menu_block = Block::default()
        .title(" Toolbox Menu ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(primary));

    let menu_items = [
        (
            "Smart Clean",
            "Find safe junk across apps, system, packages, projects and trash",
        ),
        (
            "Deep Clean",
            "Review large caches, old downloads, Docker, games and unused data",
        ),
        (
            "Analyze Disk",
            "Walk through folders and find what is eating your space",
        ),
        (
            "Apps & Leftovers",
            "Find large apps, unused apps and orphaned config/cache",
        ),
        (
            "Optimize System",
            "Startup, logs, package cache and safe performance fixes",
        ),
        ("Trash", "Review, restore or empty files moved by Nibs"),
        ("Settings", "Safety mode, cleanup method and recipes"),
        (
            "Exit Nibs",
            "Return to your terminal with a friendly goodbye",
        ),
    ];

    let items: Vec<ListItem> = menu_items
        .iter()
        .enumerate()
        .map(|(idx, (title, desc))| {
            let is_selected = idx == state.home_selected_idx;
            let prefix = if is_selected { "● " } else { "○ " };

            let style = if is_selected {
                Style::default()
                    .bg(state.theme.select_bg)
                    .fg(state.theme.select_fg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(state.theme.ink)
            };

            let (prefix_style, title_style, desc_style) = if is_selected {
                (
                    Style::default()
                        .fg(state.theme.select_fg)
                        .add_modifier(Modifier::BOLD),
                    Style::default()
                        .fg(state.theme.select_fg)
                        .add_modifier(Modifier::BOLD),
                    Style::default().fg(state.theme.select_fg),
                )
            } else {
                (
                    Style::default().fg(primary).add_modifier(Modifier::BOLD),
                    Style::default().fg(state.theme.ink),
                    Style::default().fg(Color::DarkGray),
                )
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(prefix, prefix_style),
                    Span::styled(*title, title_style),
                ]),
                Line::from(Span::styled(format!("    {}", desc), desc_style)),
            ])
            .style(style)
        })
        .collect();

    frame.render_widget(List::new(items).block(menu_block), body_chunks[0]);

    // Right Panel: Today — live system state and last scan summary
    let stats_block = Block::default()
        .title(" Today ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let mut stats_lines = Vec::new();

    // Disk usage
    if let Some((mount, total, used, avail)) = get_home_disk_snapshot() {
        let pct = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        let bar_len = ((pct / 10.0).round() as usize).min(10);
        stats_lines.push(Line::from(vec![
            Span::styled("  Mount   : ", Style::default().fg(Color::DarkGray)),
            Span::styled(mount, Style::default().fg(state.theme.ink)),
        ]));
        stats_lines.push(Line::from(vec![
            Span::styled("  Free    : ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} free of {}", format_size(avail), format_size(total)),
                Style::default().fg(state.theme.ink),
            ),
        ]));
        stats_lines.push(Line::from(vec![
            Span::styled("  Usage   : ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!(
                    "[{}{}] {:.1}%",
                    "■".repeat(bar_len),
                    " ".repeat(10 - bar_len),
                    pct
                ),
                Style::default().fg(state.theme.success),
            ),
        ]));
    } else {
        stats_lines.push(Line::from(vec![
            Span::styled("  Mount   : ", Style::default().fg(Color::DarkGray)),
            Span::styled("Unknown", Style::default().fg(state.theme.ink)),
        ]));
    }

    // Last scan info
    stats_lines.push(Line::from(""));
    stats_lines.push(Line::from(Span::styled(
        "  Last Scan",
        Style::default()
            .fg(state.theme.ink)
            .add_modifier(Modifier::UNDERLINED),
    )));
    if let Some(last_time) = state.last_scan_time {
        let ago = last_time.elapsed();
        let ago_str = if ago.as_secs() < 60 {
            format!("{}s ago", ago.as_secs())
        } else if ago.as_secs() < 3600 {
            format!("{}m ago", ago.as_secs() / 60)
        } else {
            format!(
                "{}h {}m ago",
                ago.as_secs() / 3600,
                (ago.as_secs() % 3600) / 60
            )
        };
        stats_lines.push(Line::from(vec![
            Span::styled("    Time  : ", Style::default().fg(Color::DarkGray)),
            Span::styled(ago_str, Style::default().fg(state.theme.accent)),
        ]));
        stats_lines.push(Line::from(vec![
            Span::styled("    Items : ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} findings", state.last_scan_findings),
                Style::default().fg(state.theme.ink),
            ),
        ]));
        stats_lines.push(Line::from(vec![
            Span::styled("    Safe  : ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} recommended", state.last_scan_recommended),
                Style::default().fg(state.theme.success),
            ),
        ]));
        stats_lines.push(Line::from(vec![
            Span::styled("    Size  : ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format_size(state.last_scan_size),
                Style::default().fg(state.theme.warning),
            ),
        ]));
    } else {
        stats_lines.push(Line::from(vec![
            Span::styled("    ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "No scan yet — start with Smart Clean",
                Style::default().fg(state.theme.ink),
            ),
        ]));
    }

    // Trash summary
    stats_lines.push(Line::from(""));
    let (trash_root, trash_count, trash_bytes) = get_user_trash_snapshot();
    stats_lines.push(Line::from(vec![
        Span::styled("  Trash   : ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            match trash_root {
                Some(_) => format!("{} items / {}", trash_count, format_size(trash_bytes)),
                None => "No home trash detected".to_string(),
            },
            Style::default().fg(state.theme.warning),
        ),
    ]));

    // Safety mode & clean defaults
    stats_lines.push(Line::from(""));
    stats_lines.push(Line::from(Span::styled(
        "  Safety & Defaults",
        Style::default()
            .fg(state.theme.ink)
            .add_modifier(Modifier::UNDERLINED),
    )));
    let safety_label = if state.delete_directly {
        "Direct delete (permanent)"
    } else {
        "Trash routing (recoverable)"
    };
    stats_lines.push(Line::from(vec![
        Span::styled("    Mode  : ", Style::default().fg(Color::DarkGray)),
        Span::styled(safety_label, Style::default().fg(state.theme.success)),
    ]));
    stats_lines.push(Line::from(vec![
        Span::styled("    Scope : ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "Home directory (safe paths only)",
            Style::default().fg(state.theme.ink),
        ),
    ]));
    stats_lines.push(Line::from(vec![
        Span::styled("    Trash : ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "Open manager to review/restore/empty",
            Style::default().fg(state.theme.ink),
        ),
    ]));

    frame.render_widget(
        Paragraph::new(stats_lines).block(stats_block),
        body_chunks[1],
    );

    // --- RENDER FOOTER HELP ---
    let footer_text = Line::from(vec![
        Span::styled(
            " ↑/↓ ",
            Style::default().fg(primary).add_modifier(Modifier::BOLD),
        ),
        Span::styled("Navigate │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " Enter ",
            Style::default().fg(primary).add_modifier(Modifier::BOLD),
        ),
        Span::styled("Open │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " Q/Esc ",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Span::styled("Quit ", Style::default().fg(Color::DarkGray)),
    ]);
    let footer_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(Paragraph::new(footer_text).block(footer_block), chunks[2]);

    if state.screen == TuiScreen::HomeConfirmTrash {
        let confirm_area = centered_rect_fixed(66, 11, area);
        frame.render_widget(Clear, confirm_area);
        let confirm_block = Block::default()
            .title(" Empty Trash? ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Red));
        let inner = confirm_block.inner(confirm_area);
        frame.render_widget(confirm_block, confirm_area);

        let confirm_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "This will permanently delete the contents of ",
                    Style::default(),
                ),
                Span::styled(
                    "~/.local/share/Trash",
                    Style::default().fg(state.theme.warning),
                ),
                Span::styled(".", Style::default()),
            ]),
            Line::from(""),
            render_confirm_bar(
                state.theme,
                state.confirm_idx,
                "y",
                "Empty Trash",
                "n",
                "Cancel",
            ),
        ];
        frame.render_widget(
            Paragraph::new(confirm_lines)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true }),
            inner,
        );
    }
}

fn draw_app_uninstall_selector(state: &mut TuiState, frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Header
            Constraint::Min(8),    // Body
            Constraint::Length(3), // Footer
        ])
        .split(area);

    // --- RENDER HEADER ---
    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));
    let header_inner = header_block.inner(chunks[0]);
    frame.render_widget(header_block, chunks[0]);

    let header_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(30), Constraint::Length(MASCOT_WIDTH + 4)])
        .split(header_inner);

    let header_text = vec![
        Line::from(vec![
            Span::styled(
                " Smart App Uninstaller ",
                Style::default()
                    .fg(state.theme.ink)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "— Unified Linux Uninstaller & Remnants Cleaner",
                Style::default().fg(state.theme.ink),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Total Programs Detected: ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                state.installed_apps.len().to_string(),
                Style::default()
                    .fg(state.theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " │ Select an app to search for left-over directories, configuration files and shortcuts.",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ];
    frame.render_widget(Paragraph::new(header_text), header_split[0]);

    let elapsed_ms = state.start_time.elapsed().as_millis() as u64;
    let mascot_state =
        if state.show_confirmation || state.status_message.starts_with("Searching remnants") {
            "sweeping"
        } else {
            "happy"
        };
    let mascot_lines = get_mascot_lines(elapsed_ms, mascot_state, state.theme);
    render_mascot_with_margins(frame, header_split[1], mascot_lines);

    // --- RENDER BODY LAYOUT ---
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(55), // Left: list of apps
            Constraint::Percentage(45), // Right: detail pane
        ])
        .split(chunks[1]);

    let primary = state.theme.primary;
    let accent = state.theme.accent;

    // Left List
    let list_block = Block::default()
        .title(" Installed Graphical Applications ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(primary));

    if state.installed_apps.is_empty() {
        let empty_widget = Paragraph::new(
            "\n\n No graphical applications found under standard /usr/share/applications path.",
        )
        .alignment(Alignment::Center);
        frame.render_widget(empty_widget.block(list_block), body_chunks[0]);
    } else {
        let items: Vec<ListItem> = state
            .installed_apps
            .iter()
            .enumerate()
            .map(|(idx, app)| {
                let is_selected = idx == state.selected_app_idx;
                let prefix = if is_selected { "> " } else { "  " };

                let style = if is_selected {
                    Style::default()
                        .bg(state.theme.select_bg)
                        .fg(state.theme.select_fg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(state.theme.ink)
                };

                let (prefix_style, name_style, exec_style) = if is_selected {
                    (
                        Style::default()
                            .fg(state.theme.select_fg)
                            .add_modifier(Modifier::BOLD),
                        Style::default()
                            .fg(state.theme.select_fg)
                            .add_modifier(Modifier::BOLD),
                        Style::default().fg(state.theme.select_fg),
                    )
                } else {
                    (
                        Style::default().fg(accent).add_modifier(Modifier::BOLD),
                        Style::default().fg(state.theme.ink),
                        Style::default().fg(Color::DarkGray),
                    )
                };

                ListItem::new(
                    Line::from(vec![
                        Span::styled(prefix, prefix_style),
                        Span::styled(format!("{}  ", app.name), name_style),
                        Span::styled(format!("({})", app.exec), exec_style),
                    ])
                    .style(style),
                )
            })
            .collect();

        let mut list_state = ListState::default();
        list_state.select(Some(state.selected_app_idx));
        frame.render_stateful_widget(
            List::new(items).block(list_block),
            body_chunks[0],
            &mut list_state,
        );
    }

    // Right Detail Panel
    let detail_block = Block::default()
        .title(" Application Info ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let detail_text = if let Some(app) = state.installed_apps.get(state.selected_app_idx) {
        let app_name = &app.name;
        let exec_name = &app.exec;
        let desktop_file_str = app.desktop_file.to_string_lossy().to_string();

        vec![
            Line::from(vec![
                Span::styled("App Name:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    app_name,
                    Style::default()
                        .fg(state.theme.ink)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Executable: ", Style::default().fg(Color::DarkGray)),
                Span::styled(exec_name, Style::default().fg(state.theme.accent)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Desktop Launcher File Path:",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                desktop_file_str,
                Style::default().fg(state.theme.ink),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press ENTER to inspect and checklist leftovers/remnants before trashing them.",
                Style::default().fg(state.theme.warning),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press C to immediately request trashing of ALL located remnants for this application.",
                Style::default().fg(state.theme.error),
            )),
        ]
    } else {
        vec![Line::from("\n Select an application to view details.")]
    };
    frame.render_widget(
        Paragraph::new(detail_text)
            .block(detail_block)
            .wrap(Wrap { trim: true }),
        body_chunks[1],
    );

    // --- RENDER FOOTER ---
    let footer_text = Line::from(vec![Span::styled(
        format!(" {} ", state.status_message),
        Style::default().fg(state.theme.ink),
    )]);
    let footer_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Line::from(vec![
            Span::styled(
                " Enter ",
                Style::default()
                    .fg(state.theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Inspect Leftovers │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                " C ",
                Style::default()
                    .fg(state.theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Clean All Remnants │ ",
                Style::default()
                    .fg(state.theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Esc ",
                Style::default()
                    .fg(state.theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Back to Menu", Style::default().fg(Color::DarkGray)),
        ]));
    frame.render_widget(Paragraph::new(footer_text).block(footer_block), chunks[2]);

    // --- DELETION CONFIRMATION OVERLAY ---
    if state.show_confirmation {
        let popup_area = centered_rect(55, 30, area);
        frame.render_widget(Clear, popup_area);

        let app_name = state
            .installed_apps
            .get(state.selected_app_idx)
            .map(|a| a.name.as_str())
            .unwrap_or("");

        let popup_lines = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "[!] CONFIRM FULL APP REMOVAL",
                Style::default()
                    .fg(state.theme.error)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(format!(
                "Are you sure you want to move all launcher shortcuts, config files, binaries and cached remnants of '{}' to the system trash?",
                app_name
            )),
            Line::from(""),
            render_confirm_bar(
                state.theme,
                state.confirm_idx,
                "y",
                "Trash Everything",
                "n",
                "Cancel",
            ),
        ];

        let popup_widget = Paragraph::new(popup_lines)
            .block(
                Block::default()
                    .title(" Safe Deletion Confirmation ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(state.theme.error)),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(popup_widget, popup_area);
    }
}

fn draw_scanning(state: &mut TuiState, frame: &mut Frame) {
    let area = frame.area();

    // --- Color palette ---
    let c_border = state.theme.primary;
    let c_label = state.theme.ink;
    let c_value = state.theme.ink;
    let c_accent = state.theme.accent;
    let c_green = state.theme.success;
    let c_amber = state.theme.warning;
    let c_dim = state.theme.ink;
    let c_shortcut_bg = Color::DarkGray;

    let popup_area = centered_rect_fixed(80, 11, area);
    frame.render_widget(Clear, popup_area);

    let elapsed_ms = state.start_time.elapsed().as_millis() as u64;
    let elapsed_secs = elapsed_ms / 1000;
    let elapsed_label = format!("{}:{:02}", elapsed_secs / 60, elapsed_secs % 60);

    // Pulsing title
    let dot_count = (elapsed_ms / 400) % 4;
    let dots = ".".repeat(dot_count as usize);
    let title_padded = format!(" Nibs is scanning for dust{} ", dots);

    let scanning_block = Block::default()
        .title(Line::from(vec![Span::styled(
            title_padded,
            Style::default().fg(c_value).add_modifier(Modifier::BOLD),
        )]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(c_border));
    let inner_area = scanning_block.inner(popup_area);
    frame.render_widget(scanning_block, popup_area);

    // Horizontal: padding, details, mascot, padding
    let h_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(58),
            Constraint::Length(MASCOT_WIDTH + 4),
            Constraint::Length(1),
        ])
        .split(inner_area);

    // Vertical: padding, live count, found, elapsed, active, spacer, shortcut
    let detail_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Top padding
            Constraint::Length(1), // Live scanned count
            Constraint::Length(1), // Found / Reclaimable
            Constraint::Length(1), // Elapsed
            Constraint::Length(1), // Active path
            Constraint::Min(0),    // Spacer
            Constraint::Length(1), // Esc Cancel (bottom of popup)
        ])
        .split(h_layout[1]);

    // Live scanned count (like ncdu — no percentage, just what we've processed)
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Scanned      ", Style::default().fg(c_label)),
            Span::styled(
                format!("{} entries", state.scan_files_count),
                Style::default().fg(c_accent).add_modifier(Modifier::BOLD),
            ),
        ])),
        detail_layout[1],
    );

    // Found / Reclaimable
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Found        ", Style::default().fg(c_label)),
            Span::styled(
                format!("{}", state.scan_findings_count),
                Style::default().fg(c_amber).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" targets  │  ", Style::default().fg(c_dim)),
            Span::styled(
                format_size(state.scan_total_size),
                Style::default().fg(c_green).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" reclaimable", Style::default().fg(c_label)),
        ])),
        detail_layout[2],
    );

    // Elapsed
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Elapsed      ", Style::default().fg(c_label)),
            Span::styled(elapsed_label, Style::default().fg(c_value)),
        ])),
        detail_layout[3],
    );

    // Active Path
    let path_str = state.scan_current_path.to_string_lossy().to_string();
    let max_path_len = (detail_layout[4].width as usize).saturating_sub(14);
    let display_path = if path_str.len() > max_path_len {
        format!(
            "...{}",
            &path_str[path_str
                .len()
                .saturating_sub(max_path_len.saturating_sub(3))..]
        )
    } else {
        path_str
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Active Path  ", Style::default().fg(c_label)),
            Span::styled(display_path, Style::default().fg(c_dim)),
        ])),
        detail_layout[4],
    );

    // Mascot
    let mascot_lines = get_mascot_lines(elapsed_ms, "search", state.theme);
    render_mascot_with_margins(frame, h_layout[2], mascot_lines);

    // Esc Cancel — bottom-left corner inside the popup
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" Esc ", Style::default().bg(c_shortcut_bg).fg(c_value)),
            Span::styled(" Cancel  ", Style::default().fg(c_label)),
        ])),
        detail_layout[6],
    );
}

fn draw_app_uninstall_list(state: &TuiState, frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Header
            Constraint::Min(8),    // Body
            Constraint::Length(3), // Footer
        ])
        .split(area);

    // Dynamic calculations for remnants
    let total_leftovers = state.app_remnants.len();
    let selected_count = state.selected_remnants.len();
    let total_selected_size: u64 = state
        .selected_remnants
        .iter()
        .flat_map(|&idx| state.app_remnants.get(idx).map(|(_, size)| size))
        .sum();

    // --- RENDER HEADER ---
    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));
    let header_inner = header_block.inner(chunks[0]);
    frame.render_widget(header_block, chunks[0]);

    let header_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(30), Constraint::Length(MASCOT_WIDTH + 4)])
        .split(header_inner);

    let header_lines = vec![
        Line::from(vec![
            Span::styled(
                " Smart Uninstaller remnants checklist ",
                Style::default()
                    .fg(state.theme.ink)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("— Query: '{}'", state.app_name),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Selected : ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}/{}", selected_count, total_leftovers),
                Style::default()
                    .fg(state.theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " │ Space to reclaim: ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format_size(total_selected_size),
                Style::default()
                    .fg(state.theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" ({})", get_fun_analogy(total_selected_size)),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ];
    frame.render_widget(Paragraph::new(header_lines), header_split[0]);

    let elapsed_ms = state.start_time.elapsed().as_millis() as u64;
    let mascot_lines = get_mascot_lines(elapsed_ms, "sweeping", state.theme);
    render_mascot_with_margins(frame, header_split[1], mascot_lines);

    // --- RENDER BODY LAYOUT ---
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(65), // Left: remnant paths
            Constraint::Percentage(35), // Right: detail pane
        ])
        .split(chunks[1]);

    let primary = state.theme.primary;

    // Left List
    let list_block = Block::default()
        .title(" Remnants Located ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(primary));

    if state.app_remnants.is_empty() {
        let empty_widget = Paragraph::new("\n\n No leftovers or remnants found. System is clean!")
            .alignment(Alignment::Center);
        frame.render_widget(empty_widget.block(list_block), body_chunks[0]);
    } else {
        let items: Vec<ListItem> = state
            .app_remnants
            .iter()
            .enumerate()
            .map(|(idx, (path, size))| {
                let is_checked = state.selected_remnants.contains(&idx);
                let checkbox = if is_checked { "[x]" } else { "[ ]" };
                let size_label = format!("({})", format_size(*size));
                let name_label = path.to_string_lossy().to_string();

                let item_style = if idx == state.selected_idx {
                    Style::default()
                        .bg(state.theme.select_bg)
                        .fg(state.theme.select_fg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(state.theme.ink)
                };

                let is_selected = idx == state.selected_idx;
                let (check_style, name_style, size_style) = if is_selected {
                    (
                        Style::default().fg(state.theme.select_fg),
                        Style::default()
                            .fg(state.theme.select_fg)
                            .add_modifier(Modifier::BOLD),
                        Style::default().fg(state.theme.select_fg),
                    )
                } else {
                    (
                        if is_checked {
                            Style::default().fg(state.theme.success)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        },
                        Style::default().fg(state.theme.ink),
                        Style::default().fg(Color::DarkGray),
                    )
                };

                ListItem::new(
                    Line::from(vec![
                        Span::styled(format!("{}  ", checkbox), check_style),
                        Span::styled(name_label, name_style),
                        Span::styled(format!("  {}", size_label), size_style),
                    ])
                    .style(item_style),
                )
            })
            .collect();

        let mut list_state = ListState::default();
        list_state.select(Some(state.selected_idx));
        frame.render_stateful_widget(
            List::new(items).block(list_block),
            body_chunks[0],
            &mut list_state,
        );
    }

    // Right Detail Panel
    let detail_block = Block::default()
        .title(" Remnant Details ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let detail_text = if let Some((path, size)) = state.app_remnants.get(state.selected_idx) {
        let filename = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        let path_str = path.to_string_lossy().to_string();
        let is_shortcut = path_str.contains("applications") || path_str.ends_with(".desktop");
        let is_config = path_str.contains(".config");
        let is_cache = path_str.contains(".cache");

        let mut details = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    filename,
                    Style::default()
                        .fg(state.theme.ink)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Size: ", Style::default().fg(Color::DarkGray)),
                Span::styled(format_size(*size), Style::default().fg(state.theme.success)),
            ]),
            Line::from(vec![
                Span::styled("Type: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    if path.is_dir() { "Directory" } else { "File" },
                    Style::default().fg(state.theme.accent),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Full System Path:",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(path_str, Style::default().fg(state.theme.ink))),
            Line::from(""),
        ];

        // Specific warning context for shortcut launchers
        if is_shortcut {
            details.push(Line::from(Span::styled("[Shortcut Launcher] This is a desktop system shortcut file. Deleting this cleans the app from launcher dashboards.", Style::default().fg(state.theme.warning))));
        } else if is_config {
            details.push(Line::from(Span::styled("[User Preferences] Contains user preferences, theme configuration, or saved logins for the application.", Style::default().fg(state.theme.accent))));
        } else if is_cache {
            details.push(Line::from(Span::styled("[Application Cache] Cache registry directory. Safe to clean, but will re-download assets on reinstall.", Style::default().fg(state.theme.success))));
        }

        details
    } else {
        vec![Line::from("\n Select a path to review details.")]
    };
    frame.render_widget(
        Paragraph::new(detail_text)
            .block(detail_block)
            .wrap(Wrap { trim: true }),
        body_chunks[1],
    );

    // --- RENDER FOOTER ---
    let footer_text = Line::from(vec![Span::styled(
        format!(" {} ", state.status_message),
        Style::default().fg(state.theme.ink),
    )]);
    let footer_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Line::from(vec![
            Span::styled(
                " Space ",
                Style::default()
                    .fg(state.theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Toggle │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                " A ",
                Style::default()
                    .fg(state.theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Toggle All │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                " C ",
                Style::default()
                    .fg(state.theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Trash Selected │ ",
                Style::default()
                    .fg(state.theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Esc ",
                Style::default()
                    .fg(state.theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Back to Menu", Style::default().fg(Color::DarkGray)),
        ]));
    frame.render_widget(Paragraph::new(footer_text).block(footer_block), chunks[2]);

    // --- DELETION CONFIRMATION OVERLAY ---
    if state.show_confirmation {
        let popup_area = centered_rect(55, 30, area);
        frame.render_widget(Clear, popup_area);

        let popup_lines = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "[!] CONFIRM UNINSTALL CLEAN",
                Style::default()
                    .fg(state.theme.error)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(
                "Are you sure you want to move the selected application remnants and desktop shortcuts to the system trash?",
            ),
            Line::from(""),
            Line::from(vec![Span::styled(
                format!(
                    " {} selected items (freed: {}) ",
                    selected_count,
                    format_size(total_selected_size)
                ),
                Style::default()
                    .fg(state.theme.accent)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            render_confirm_bar(
                state.theme,
                state.confirm_idx,
                "y",
                "Trash Selected",
                "n",
                "Cancel",
            ),
        ];

        let popup_widget = Paragraph::new(popup_lines)
            .block(
                Block::default()
                    .title(" Safe Deletion Confirmation ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(state.theme.error)),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(popup_widget, popup_area);
    }
}

fn draw_wizard(state: &TuiState, frame: &mut Frame) {
    let area = frame.area();
    let popup_area = centered_rect(75, 65, area);
    frame.render_widget(Clear, popup_area);

    let primary = state.theme.primary;

    let wizard_block = Block::default()
        .title(" Nibs Cleaning Profile Wizard ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(primary));
    let wizard_inner = wizard_block.inner(popup_area);
    frame.render_widget(wizard_block, popup_area);

    let wizard_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(MASCOT_WIDTH + 4), // Left: mascot
            Constraint::Min(20),                  // Right: profile list and info
        ])
        .split(wizard_inner);

    let elapsed_ms = state.start_time.elapsed().as_millis() as u64;
    let mascot_lines = get_mascot_lines(elapsed_ms, "happy", state.theme);
    render_mascot_with_margins(frame, wizard_layout[0], mascot_lines);

    let total_bytes: u64 = state.findings.iter().map(|f| f.size_bytes).sum();
    let total_reclaimable = format_size(total_bytes);

    let mut wizard_lines = vec![
        Line::from(vec![Span::styled(
            "Welcome to Nibs!",
            Style::default().fg(primary).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::raw("Nibs scanned and found "),
            Span::styled(
                format!("{} reclaimable items", state.findings.len()),
                Style::default().fg(primary).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" totaling "),
            Span::styled(
                total_reclaimable.clone(),
                Style::default()
                    .fg(state.theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("."),
        ]),
        Line::from(vec![
            Span::styled("💡 Fun Fact: ", Style::default().fg(state.theme.warning)),
            Span::styled(
                format!("This reclaimed space is {}", get_fun_analogy(total_bytes)),
                Style::default().fg(state.theme.ink),
            ),
        ]),
        Line::from("Choose how assertive Nibs should be for this cleanup pass:"),
        Line::from(""),
    ];

    let profiles = [
        (
            "Recommended Defaults",
            "Selects only rules marked default_action=clean and risk=safe.",
            "Best for daily use. Review items, duplicates, and dependency folders stay unselected.",
        ),
        (
            "Safe Review Sweep",
            "Selects every safe finding, including rebuildable caches that may take time to restore.",
            "Still excludes review/risky/info findings. Good before a bigger cleanup session.",
        ),
        (
            "Manual Checklist",
            "Starts with nothing selected so you can decide item by item.",
            "Use this for dependency folders, duplicates, broad caches, and anything suspicious.",
        ),
    ];

    for (idx, &(title, desc, risk)) in profiles.iter().enumerate() {
        let is_selected = idx == state.wizard_idx;
        let selector = if is_selected { "●  " } else { "○  " };
        let style = if is_selected {
            Style::default().fg(primary).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(state.theme.ink)
        };

        wizard_lines.push(Line::from(vec![
            Span::styled(selector, style),
            Span::styled(title, style),
        ]));
        wizard_lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(desc, Style::default().fg(Color::Gray)),
        ]));
        wizard_lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(risk, Style::default().fg(Color::DarkGray)),
        ]));
        wizard_lines.push(Line::from(""));
    }

    wizard_lines.push(Line::from(""));
    wizard_lines.push(Line::from(vec![Span::styled(
        " ↑/↓ or j/k: Select Profile  │  Enter: Proceed  │  q: Quit ",
        Style::default().bg(Color::DarkGray).fg(state.theme.ink),
    )]));

    let wizard_widget = Paragraph::new(wizard_lines).wrap(Wrap { trim: true });
    frame.render_widget(wizard_widget, wizard_layout[1]);
}

fn draw_dashboard(state: &mut TuiState, frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Header
            Constraint::Min(8),    // Body
            Constraint::Length(3), // Footer
        ])
        .split(area);

    let is_smart = state.screen == TuiScreen::SmartClean;

    // --- RENDER HEADER ---
    let total_bytes: u64 = state.findings.iter().map(|f| f.size_bytes).sum();
    let total_reclaimable = format_size(total_bytes);
    let selected_count = state.selected_findings.len();
    let selected_size = state.selected_size_bytes();
    let (recommended_count, recommended_size) = state.recommended_summary();
    let visible_count = state.filter_count();

    if is_smart {
        // Calculate breakdown for Smart Clean
        let mut needs_review_bytes = 0u64;
        let mut protected_bytes = 0u64;
        for f in &state.findings {
            match f.risk {
                crate::findings::RiskLevel::Review => needs_review_bytes += f.size_bytes,
                crate::findings::RiskLevel::Risky | crate::findings::RiskLevel::Info => {
                    protected_bytes += f.size_bytes
                }
                _ => {}
            }
        }
        let potential_cleanup = total_bytes.saturating_sub(protected_bytes);

        let header_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(state.theme.primary));
        let header_inner = header_block.inner(chunks[0]);
        frame.render_widget(header_block, chunks[0]);

        let header_split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(30), Constraint::Length(MASCOT_WIDTH + 4)])
            .split(header_inner);

        let header_text = vec![
            Line::from(vec![
                Span::styled(
                    " Smart Clean ",
                    Style::default()
                        .fg(state.theme.primary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "— safe junk across apps, caches, packages and projects",
                    Style::default().fg(state.theme.ink),
                ),
            ]),
            Line::from(vec![
                Span::styled("Potential cleanup  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{}  ", format_size(potential_cleanup)),
                    Style::default()
                        .fg(state.theme.ink)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("│ Recommended  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{}  ", format_size(recommended_size)),
                    Style::default()
                        .fg(state.theme.success)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("│ Needs review  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{}  ", format_size(needs_review_bytes)),
                    Style::default()
                        .fg(state.theme.warning)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("│ Protected  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format_size(protected_bytes),
                    Style::default()
                        .fg(state.theme.error)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Selected: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} / {}  ", selected_count, format_size(selected_size)),
                    Style::default()
                        .fg(state.theme.warning)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("│ Filter: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(
                        "{} {} ({}/{})",
                        state.finding_view_mode.label(),
                        state.finding_filter.label(),
                        visible_count,
                        state.findings.len()
                    ),
                    Style::default()
                        .fg(state.theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];
        let header_widget = Paragraph::new(header_text);
        frame.render_widget(header_widget, header_split[0]);

        let mascot_lines = get_mascot_lines(
            state.start_time.elapsed().as_millis() as u64,
            "sweeping",
            state.theme,
        );
        render_mascot_with_margins(frame, header_split[1], mascot_lines);
    } else {
        let scope_name = match &state.scope {
            crate::safety::ScanScope::ProjectScan(_) => "Project Scan",
            crate::safety::ScanScope::DirectoryScan(_) => "Directory Scan",
            crate::safety::ScanScope::SystemSafeScan => "System Safe Scan [!]",
        };

        let header_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray));
        let header_inner = header_block.inner(chunks[0]);
        frame.render_widget(header_block, chunks[0]);

        let header_split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(30), Constraint::Length(MASCOT_WIDTH + 4)])
            .split(header_inner);

        let header_text = vec![
            Line::from(vec![
                Span::styled(
                    " Nibs ",
                    Style::default()
                        .fg(state.theme.ink)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "— Terminal Cleaner for Developers",
                    Style::default().fg(state.theme.ink),
                ),
            ]),
            Line::from(vec![
                Span::styled("Scope: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} ", scope_name),
                    Style::default()
                        .fg(state.theme.warning)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("│ Path: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    state.target_path.to_string_lossy().to_string(),
                    Style::default().fg(state.theme.ink),
                ),
            ]),
            Line::from(vec![
                Span::styled("Reclaimable Space: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    total_reclaimable,
                    Style::default()
                        .fg(state.theme.success)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" │ Warnings: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    state.warnings.len().to_string(),
                    Style::default().fg(if state.warnings.is_empty() {
                        state.theme.ink
                    } else {
                        state.theme.error
                    }),
                ),
            ]),
            Line::from(vec![
                Span::styled("Filter: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(
                        "{} {} ({}/{})",
                        state.finding_view_mode.label(),
                        state.finding_filter.label(),
                        visible_count,
                        state.findings.len()
                    ),
                    Style::default()
                        .fg(state.theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" │ Recommended: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} / {}", recommended_count, format_size(recommended_size)),
                    Style::default().fg(state.theme.success),
                ),
                Span::styled(" │ Selected: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} / {}", selected_count, format_size(selected_size)),
                    Style::default()
                        .fg(state.theme.warning)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];
        let header_widget = Paragraph::new(header_text);
        frame.render_widget(header_widget, header_split[0]);

        let mascot_state = if state.findings.is_empty() {
            "happy"
        } else {
            "sweeping"
        };
        let elapsed_ms = state.start_time.elapsed().as_millis() as u64;
        let mascot_lines = get_mascot_lines(elapsed_ms, mascot_state, state.theme);
        render_mascot_with_margins(frame, header_split[1], mascot_lines);
    }
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Left: Findings List
            Constraint::Percentage(40), // Right: Details/Explanation
        ])
        .split(chunks[1]);

    // Left Panel: Findings List
    let visible_finding_groups = state.visible_finding_groups();
    let visible_finding_indices = state.visible_finding_indices();
    if state.findings.is_empty() {
        let empty_widget = Paragraph::new("\n\n No reclaimable files found in this scope!")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(" Findings "),
            );
        frame.render_widget(empty_widget, body_chunks[0]);
    } else if state.filter_count() == 0 {
        let empty_widget = Paragraph::new(format!(
            "\n\nNo findings match the {} filter.",
            state.finding_filter.label()
        ))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Findings "),
        );
        frame.render_widget(empty_widget, body_chunks[0]);
    } else if state.finding_view_mode == crate::tui::model::FindingViewMode::Advanced {
        let items: Vec<ListItem> = visible_finding_indices
            .iter()
            .map(|idx| {
                let finding = &state.findings[*idx];
                let is_checked = state.selected_findings.contains(idx);
                let checkbox = if is_checked { "[x]" } else { "[ ]" };
                let risk_color = match finding.risk {
                    RiskLevel::Safe => state.theme.success,
                    RiskLevel::Review => state.theme.warning,
                    RiskLevel::Risky => state.theme.error,
                    RiskLevel::Info => state.theme.info,
                };
                let risk_badge = format!("[{}]", finding.risk);
                let size_str = format!("({})", format_size(finding.size_bytes));
                let display_path = get_relative_display_path(&finding.path, &state.target_path);

                let style = if *idx == state.selected_idx {
                    Style::default()
                        .bg(state.theme.select_bg)
                        .fg(state.theme.select_fg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(state.theme.ink)
                };

                let is_selected = *idx == state.selected_idx;
                let (check_style, risk_style, rule_style, path_style, size_style) = if is_selected {
                    (
                        Style::default().fg(state.theme.select_fg),
                        Style::default()
                            .fg(state.theme.select_fg)
                            .add_modifier(Modifier::BOLD),
                        Style::default().fg(state.theme.select_fg),
                        Style::default().fg(state.theme.select_fg),
                        Style::default().fg(state.theme.select_fg),
                    )
                } else {
                    (
                        if is_checked {
                            Style::default().fg(state.theme.success)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        },
                        Style::default().fg(risk_color).add_modifier(Modifier::BOLD),
                        Style::default().fg(Color::DarkGray),
                        Style::default().fg(state.theme.ink),
                        Style::default().fg(Color::DarkGray),
                    )
                };

                ListItem::new(
                    Line::from(vec![
                        Span::styled(format!("{}  ", checkbox), check_style),
                        Span::styled(format!("{:<8} ", risk_badge), risk_style),
                        Span::styled(format!("{}  ", finding.rule_name), rule_style),
                        Span::styled(display_path, path_style),
                        Span::styled(format!("  {}", size_str), size_style),
                    ])
                    .style(style),
                )
            })
            .collect();

        let list_widget = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(state.theme.primary))
                    .title(" Findings · Advanced "),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        let mut list_state = ListState::default();
        list_state.select(state.selected_visible_position());
        frame.render_stateful_widget(list_widget, body_chunks[0], &mut list_state);
    } else {
        let items: Vec<ListItem> = visible_finding_groups
            .iter()
            .map(|group| {
                let finding = &state.findings[group[0]];
                let selected_count = group
                    .iter()
                    .filter(|idx| state.selected_findings.contains(idx))
                    .count();
                let checkbox = if selected_count == group.len() {
                    "[x]"
                } else if selected_count > 0 {
                    "[-]"
                } else {
                    "[ ]"
                };
                let group_size: u64 = group
                    .iter()
                    .map(|idx| state.findings[*idx].size_bytes)
                    .sum();

                let risk_color = match finding.risk {
                    RiskLevel::Safe => state.theme.success,
                    RiskLevel::Review => state.theme.warning,
                    RiskLevel::Risky => state.theme.error,
                    RiskLevel::Info => state.theme.info,
                };

                let risk_badge = format!("[{}]", finding.risk);
                let size_str = format!("({})", format_size(group_size));
                let display_path = if group.len() == 1 {
                    get_relative_display_path(&finding.path, &state.target_path)
                } else {
                    format!("{} paths", group.len())
                };

                let style = if group.contains(&state.selected_idx) {
                    Style::default()
                        .bg(state.theme.select_bg)
                        .fg(state.theme.select_fg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(state.theme.ink)
                };

                let is_selected = group.contains(&state.selected_idx);
                let (check_style, risk_style, rule_style, path_style, size_style) = if is_selected {
                    (
                        Style::default().fg(state.theme.select_fg),
                        Style::default()
                            .fg(state.theme.select_fg)
                            .add_modifier(Modifier::BOLD),
                        Style::default()
                            .fg(state.theme.select_fg)
                            .add_modifier(Modifier::BOLD),
                        Style::default().fg(state.theme.select_fg),
                        Style::default().fg(state.theme.select_fg),
                    )
                } else {
                    (
                        if selected_count > 0 {
                            Style::default().fg(state.theme.success)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        },
                        Style::default().fg(risk_color).add_modifier(Modifier::BOLD),
                        Style::default().fg(state.theme.ink),
                        Style::default().fg(Color::DarkGray),
                        Style::default().fg(Color::DarkGray),
                    )
                };

                ListItem::new(
                    Line::from(vec![
                        Span::styled(format!("{}  ", checkbox), check_style),
                        Span::styled(format!("{:<8} ", risk_badge), risk_style),
                        Span::styled(format!("{}  ", finding.rule_name), rule_style),
                        Span::styled(display_path, path_style),
                        Span::styled(format!("  {}", size_str), size_style),
                    ])
                    .style(style),
                )
            })
            .collect();

        let list_widget = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(state.theme.primary))
                    .title(" Findings "),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        let mut list_state = ListState::default();
        list_state.select(state.selected_visible_position());
        frame.render_stateful_widget(list_widget, body_chunks[0], &mut list_state);
    }

    // Right Panel: Details or Explanation
    let right_panel_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(state.theme.primary))
        .title(if state.show_explanation {
            " Rule Explanation "
        } else {
            " Finding Details "
        });

    let selected_group = if state.finding_view_mode == crate::tui::model::FindingViewMode::Advanced
    {
        state
            .findings
            .get(state.selected_idx)
            .map(|_| vec![state.selected_idx])
    } else {
        state
            .visible_finding_groups()
            .into_iter()
            .find(|group| group.contains(&state.selected_idx))
    };

    if let Some(group) = selected_group {
        let finding = &state.findings[group[0]];
        let group_size: u64 = group
            .iter()
            .map(|idx| state.findings[*idx].size_bytes)
            .sum();
        let selected_in_group = group
            .iter()
            .filter(|idx| state.selected_findings.contains(idx))
            .count();
        let mut detail_lines = Vec::new();

        if state.show_explanation {
            detail_lines.push(Line::from(vec![
                Span::styled("Rule ID: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    &finding.rule_id,
                    Style::default()
                        .fg(state.theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            detail_lines.push(Line::from(vec![
                Span::styled("Rule Name: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    &finding.rule_name,
                    Style::default()
                        .fg(state.theme.ink)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            detail_lines.push(Line::from(vec![
                Span::styled("Default Action: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    finding.default_action.as_deref().unwrap_or("review"),
                    Style::default().fg(state.theme.warning),
                ),
            ]));
            detail_lines.push(Line::from(vec![
                Span::styled("Group: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(
                        "{} paths, {} selected, {} total",
                        group.len(),
                        selected_in_group,
                        format_size(group_size)
                    ),
                    Style::default().fg(state.theme.success),
                ),
            ]));
            detail_lines.push(Line::from(""));
            detail_lines.push(Line::from(Span::styled(
                "Reason for Cleanup:",
                Style::default()
                    .fg(state.theme.ink)
                    .add_modifier(Modifier::UNDERLINED),
            )));
            detail_lines.push(Line::from(Span::styled(
                &finding.reason,
                Style::default().fg(state.theme.ink),
            )));
            detail_lines.push(Line::from(""));

            if let Some(ref restores) = finding.restore {
                detail_lines.push(Line::from(Span::styled(
                    "How to Rebuild/Restore:",
                    Style::default()
                        .fg(state.theme.ink)
                        .add_modifier(Modifier::UNDERLINED),
                )));
                for cmd in restores {
                    detail_lines.push(Line::from(Span::styled(
                        format!("  $ {}", cmd),
                        Style::default().fg(state.theme.warning),
                    )));
                }
            } else {
                detail_lines.push(Line::from(Span::styled(
                    "No rebuild command specified. Safe to clean.",
                    Style::default().fg(Color::DarkGray),
                )));
            }

            detail_lines.push(Line::from(""));
            detail_lines.push(Line::from(Span::styled(
                "Matched Paths:",
                Style::default()
                    .fg(state.theme.ink)
                    .add_modifier(Modifier::UNDERLINED),
            )));
            for idx in group.iter().take(8) {
                let path =
                    get_relative_display_path(&state.findings[*idx].path, &state.target_path);
                detail_lines.push(Line::from(Span::styled(
                    format!("  {}", path),
                    Style::default().fg(state.theme.ink),
                )));
            }
            if group.len() > 8 {
                detail_lines.push(Line::from(Span::styled(
                    format!("  ...{} more paths", group.len() - 8),
                    Style::default().fg(Color::DarkGray),
                )));
            }
        } else {
            detail_lines.push(Line::from(vec![
                Span::styled("Group: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} ({})", finding.rule_name, finding.rule_id),
                    Style::default().fg(state.theme.accent),
                ),
            ]));
            if let Some(cleaner_name) = &finding.cleaner_name {
                detail_lines.push(Line::from(vec![
                    Span::styled("Cleaner: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(cleaner_name, Style::default().fg(state.theme.ink)),
                ]));
            }
            detail_lines.push(Line::from(vec![
                Span::styled("Category: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    finding.category.to_string(),
                    Style::default().fg(state.theme.ink),
                ),
            ]));
            if let Some(safety_class) = finding.safety_class {
                detail_lines.push(Line::from(vec![
                    Span::styled("Safety Class: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        safety_class.to_string(),
                        Style::default().fg(match safety_class {
                            crate::findings::SafetyClass::Safe => state.theme.success,
                            crate::findings::SafetyClass::UsuallySafe => state.theme.success,
                            crate::findings::SafetyClass::Rebuildable => state.theme.warning,
                            crate::findings::SafetyClass::UserData => state.theme.warning,
                            crate::findings::SafetyClass::SecretOrAuth => state.theme.error,
                        }),
                    ),
                ]));
            }
            detail_lines.push(Line::from(vec![
                Span::styled("Risk Level: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    finding.risk.to_string(),
                    Style::default().fg(match finding.risk {
                        RiskLevel::Safe => state.theme.success,
                        RiskLevel::Review => state.theme.warning,
                        RiskLevel::Risky => state.theme.error,
                        RiskLevel::Info => state.theme.info,
                    }),
                ),
            ]));
            detail_lines.push(Line::from(vec![
                Span::styled("Size: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format_size(group_size),
                    Style::default().fg(state.theme.success),
                ),
            ]));
            detail_lines.push(Line::from(vec![
                Span::styled("Paths: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} total, {} selected", group.len(), selected_in_group),
                    Style::default().fg(state.theme.ink),
                ),
            ]));
            detail_lines.push(Line::from(vec![
                Span::styled("Default Action: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    finding.default_action.as_deref().unwrap_or("review"),
                    Style::default().fg(state.theme.warning),
                ),
            ]));
            if let Some(mtime) = finding.last_modified {
                detail_lines.push(Line::from(vec![
                    Span::styled("Last Modified: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        mtime.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                        Style::default().fg(state.theme.ink),
                    ),
                ]));
            }

            detail_lines.push(Line::from(""));
            detail_lines.push(Line::from(Span::styled(
                "Why:",
                Style::default().fg(Color::DarkGray),
            )));
            detail_lines.push(Line::from(Span::styled(
                &finding.reason,
                Style::default().fg(state.theme.ink),
            )));
            if let Some(ref restores) = finding.restore {
                detail_lines.push(Line::from(""));
                detail_lines.push(Line::from(Span::styled(
                    "Rebuild:",
                    Style::default().fg(Color::DarkGray),
                )));
                for cmd in restores {
                    detail_lines.push(Line::from(vec![
                        Span::styled("  $ ", Style::default().fg(Color::DarkGray)),
                        Span::styled(cmd.clone(), Style::default().fg(state.theme.warning)),
                    ]));
                }
            }
            detail_lines.push(Line::from(""));
            detail_lines.push(Line::from(Span::styled(
                "Matched Paths:",
                Style::default().fg(Color::DarkGray),
            )));
            for idx in group.iter().take(10) {
                let item = &state.findings[*idx];
                let path = get_relative_display_path(&item.path, &state.target_path);
                detail_lines.push(Line::from(vec![
                    Span::styled(
                        format!("{:>9}  ", format_size(item.size_bytes)),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(path, Style::default().fg(state.theme.ink)),
                ]));
            }
            if group.len() > 10 {
                detail_lines.push(Line::from(Span::styled(
                    format!("...{} more paths", group.len() - 10),
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }

        let detail_widget = Paragraph::new(detail_lines)
            .block(right_panel_block)
            .wrap(Wrap { trim: true });
        frame.render_widget(detail_widget, body_chunks[1]);
    } else {
        let detail_widget = Paragraph::new("\n Select a finding to view details.")
            .block(right_panel_block)
            .alignment(Alignment::Center);
        frame.render_widget(detail_widget, body_chunks[1]);
    }

    // --- RENDER FOOTER ---
    let primary = state.theme.primary;
    let footer_text = Line::from(vec![Span::styled(
        format!(" {} ", state.status_message),
        Style::default().fg(state.theme.ink),
    )]);

    let cleanup_label = if state.dry_run {
        "Clean (Simulated)"
    } else if state.delete_directly {
        "Clean (Delete)"
    } else {
        "Clean (Trash)"
    };

    let footer_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Line::from(vec![
            Span::styled(
                " Esc / Q ",
                Style::default()
                    .fg(state.theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if state.is_home_mode {
                    "Back to Menu │ "
                } else {
                    "Quit │ "
                },
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                " Space ",
                Style::default().fg(primary).add_modifier(Modifier::BOLD),
            ),
            Span::styled("Toggle │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                " A ",
                Style::default().fg(primary).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Select Recommended │ ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                " F ",
                Style::default().fg(primary).add_modifier(Modifier::BOLD),
            ),
            Span::styled("Filter │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                " V ",
                Style::default().fg(primary).add_modifier(Modifier::BOLD),
            ),
            Span::styled("View │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                " C ",
                Style::default()
                    .fg(state.theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                cleanup_label,
                Style::default()
                    .fg(state.theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

    let footer_widget = Paragraph::new(footer_text).block(footer_block);
    frame.render_widget(footer_widget, chunks[2]);

    // --- RENDER POPUP OVERLAY ---
    if state.show_confirmation {
        let popup_area = centered_rect(60, 35, area);
        frame.render_widget(Clear, popup_area);

        let selected_count = state.selected_findings.len();
        let selected_size_bytes = state.selected_size_bytes();

        let cleanup_type = if state.dry_run {
            "SIMULATED DRY-RUN"
        } else {
            "SYSTEM TRASH CLEANUP"
        };
        let action_verbiage = if state.dry_run {
            "simulate cleanup"
        } else {
            "move items to trash"
        };

        let popup_lines = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "[!] CONFIRM CLEANUP ACTION",
                Style::default()
                    .fg(state.theme.error)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::raw("Are you sure you want to perform a "),
                Span::styled(
                    cleanup_type,
                    Style::default()
                        .fg(state.theme.warning)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("?"),
            ]),
            Line::from(vec![
                Span::raw("This will "),
                Span::raw(action_verbiage),
                Span::raw(" for "),
                Span::styled(
                    format!("{} selected findings", selected_count),
                    Style::default()
                        .fg(state.theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("reclaiming a total of "),
                Span::styled(
                    format_size(selected_size_bytes),
                    Style::default()
                        .fg(state.theme.success)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("."),
            ]),
            Line::from(vec![
                Span::styled("Fun Fact: ", Style::default().fg(state.theme.warning)),
                Span::styled(
                    format!(
                        "This reclaimed space is {}",
                        get_fun_analogy(selected_size_bytes)
                    ),
                    Style::default().fg(state.theme.ink),
                ),
            ]),
            Line::from(""),
            render_confirm_bar(
                state.theme,
                state.confirm_idx,
                "y",
                "Proceed",
                "n",
                "Cancel",
            ),
        ];

        let popup_widget = Paragraph::new(popup_lines)
            .block(
                Block::default()
                    .title(" Confirmation Required ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(state.theme.error)),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(popup_widget, popup_area);
    }
}

fn render_confirm_bar(
    theme: &NibsTheme,
    idx: usize,
    yes_key: &str,
    yes_label: &str,
    no_key: &str,
    no_label: &str,
) -> Line<'static> {
    let yes_active = idx == 0;
    let no_active = idx == 1;
    Line::from(vec![
        Span::styled(
            if yes_active {
                format!(" [{}] {} ", yes_key.to_uppercase(), yes_label)
            } else {
                format!(" [{}] {} ", yes_key.to_lowercase(), yes_label)
            },
            if yes_active {
                Style::default()
                    .bg(theme.select_bg)
                    .fg(theme.select_fg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            if no_active {
                format!(" [{}] {} ", no_key.to_uppercase(), no_label)
            } else {
                format!(" [{}] {} ", no_key.to_lowercase(), no_label)
            },
            if no_active {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ),
    ])
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

fn centered_rect_fixed(width: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(r.height.saturating_sub(height) / 2),
            Constraint::Length(height.min(r.height)),
            Constraint::Length(r.height.saturating_sub(height) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(r.width.saturating_sub(width) / 2),
            Constraint::Length(width.min(r.width)),
            Constraint::Length(r.width.saturating_sub(width) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn get_relative_display_path(path: &Path, target_path: &Path) -> String {
    if let Ok(rel) = path.strip_prefix(target_path) {
        if rel.as_os_str().is_empty() {
            path.to_string_lossy().to_string()
        } else {
            format!("./{}", rel.to_string_lossy())
        }
    } else {
        path.to_string_lossy().to_string()
    }
}

fn get_system_uptime() -> String {
    if let Ok(content) = std::fs::read_to_string("/proc/uptime")
        && let Some(first_word) = content.split_whitespace().next()
        && let Ok(uptime_secs) = first_word.parse::<f64>()
    {
        let uptime = uptime_secs as u64;
        let days = uptime / 86400;
        let hours = (uptime % 86400) / 3600;
        let minutes = (uptime % 3600) / 60;
        if days > 0 {
            return format!("{}d {}h {}m", days, hours, minutes);
        } else if hours > 0 {
            return format!("{}h {}m", hours, minutes);
        } else {
            return format!("{}m", minutes);
        }
    }
    "Unknown".to_string()
}

fn get_cpu_model() -> String {
    if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
        for line in content.lines() {
            if line.contains("model name")
                && let Some((_, model)) = line.split_once(':')
            {
                let cleaned = model.trim().to_string();
                if cleaned.len() > 60 {
                    return format!("{}...", &cleaned[..57]);
                }
                return cleaned;
            }
        }
    }
    "Generic CPU".to_string()
}

fn draw_cpu_bar_graph(pct: f64, theme: &NibsTheme) -> Line<'static> {
    let mut spans = vec![Span::styled(
        "  CPU Load:   ",
        Style::default().fg(Color::DarkGray),
    )];
    spans.extend(draw_smooth_bar_spans(
        pct,
        20,
        theme.primary,
        Color::DarkGray,
    ));
    spans.push(Span::styled(
        format!(" {:.1}%", pct),
        Style::default().fg(theme.ink).add_modifier(Modifier::BOLD),
    ));
    Line::from(spans)
}

fn draw_bar_graph(
    theme: &NibsTheme,
    label: &str,
    used: u64,
    total: u64,
    color: Color,
) -> Line<'static> {
    let pct = if total > 0 {
        (used as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    let mut spans = vec![Span::styled(
        format!("  {:<12}", label),
        Style::default().fg(Color::DarkGray),
    )];
    spans.extend(draw_smooth_bar_spans(pct, 20, color, Color::DarkGray));
    spans.push(Span::styled(
        format!(" {:.1}% ", pct),
        Style::default().fg(theme.ink).add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::styled(
        format!("({}/{})", format_size(used), format_size(total)),
        Style::default().fg(Color::DarkGray),
    ));
    Line::from(spans)
}

fn draw_net_bar_graph(
    theme: &NibsTheme,
    label: &str,
    rate_kb: f64,
    max_kb: f64,
    color: Color,
) -> Line<'static> {
    let pct = (rate_kb / max_kb * 100.0).clamp(0.0, 100.0);
    let mut spans = vec![Span::styled(
        format!("  {:<12}", label),
        Style::default().fg(Color::DarkGray),
    )];
    spans.extend(draw_smooth_bar_spans(pct, 20, color, Color::DarkGray));
    spans.push(Span::styled(
        format!(" {:.1} KB/s", rate_kb),
        Style::default().fg(theme.ink).add_modifier(Modifier::BOLD),
    ));
    Line::from(spans)
}

fn draw_optimize(state: &TuiState, frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Header
            Constraint::Min(10),   // Body layout (Options on left, results on right)
            Constraint::Length(3), // Footer
        ])
        .split(area);

    // --- HEADER ---
    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));
    let header_inner = header_block.inner(chunks[0]);
    frame.render_widget(header_block, chunks[0]);

    let header_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(30), Constraint::Length(MASCOT_WIDTH + 4)])
        .split(header_inner);

    let header_text = vec![
        Line::from(vec![
            Span::styled(
                " System Optimizer ",
                Style::default()
                    .fg(state.theme.ink)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "— Speed up system performance and reclaim storage",
                Style::default().fg(state.theme.ink),
            ),
        ]),
        Line::from(vec![Span::styled(
            "Review safe maintenance guidance. Nibs avoids sudo-only destructive actions here.",
            Style::default().fg(Color::DarkGray),
        )]),
    ];
    frame.render_widget(Paragraph::new(header_text), header_split[0]);

    let elapsed_ms = state.start_time.elapsed().as_millis() as u64;
    let mascot_state = if state.opt_in_progress {
        "sweeping"
    } else {
        "idle"
    };
    let mascot_lines = get_mascot_lines(elapsed_ms, mascot_state, state.theme);
    render_mascot_with_margins(frame, header_split[1], mascot_lines);

    // --- BODY ---
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Left: Options list
            Constraint::Percentage(50), // Right: Output log console
        ])
        .split(chunks[1]);

    // Left Panel: Options
    let opt_block = Block::default()
        .title(" Optimization Scripts ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(state.theme.primary));

    let options = [
        (
            "Flush DNS Cache",
            "Clear systemd-resolved and local DNS caches to resolve naming issues",
        ),
        (
            "Rebuild Font & MIME Caches",
            "Regenerate local font cache and update MIME type database associations",
        ),
        (
            "Compact SQLite Databases",
            "Compact browsers and VS Code databases using SQLite VACUUM",
        ),
        (
            "Clean Package Manager Cache",
            "Clear packages cache for apt, dnf, or pacman to reclaim storage (requires sudo)",
        ),
        (
            "Remove Orphan Packages",
            "Prunes unused/orphaned system packages from the package manager (requires root/sudo)",
        ),
        (
            "Sync Disk Write Buffers",
            "Forces dirty page cache buffers to write to disk safely (sync)",
        ),
    ];

    let mut list_items = Vec::new();
    for (i, (name, desc)) in options.iter().enumerate() {
        let is_selected = state.opt_selected_indices.contains(&i);
        let checkbox = if is_selected { "[x] " } else { "[ ] " };
        let is_cursor = i == state.opt_cursor_idx;

        let style = if is_cursor {
            Style::default()
                .bg(state.theme.select_bg)
                .fg(state.theme.select_fg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(state.theme.ink)
        };

        let check_style = if is_cursor {
            Style::default()
                .fg(state.theme.select_fg)
                .add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default()
                .fg(state.theme.success)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let desc_style = if is_cursor {
            Style::default()
                .bg(state.theme.select_bg)
                .fg(state.theme.select_fg)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        list_items.push(ListItem::new(vec![
            Line::from(vec![
                Span::styled(checkbox, check_style),
                Span::styled(*name, style),
            ])
            .style(if is_cursor { style } else { Style::default() }),
            Line::from(vec![
                Span::styled("    ", if is_cursor { style } else { Style::default() }),
                Span::styled(*desc, desc_style),
            ])
            .style(if is_cursor { style } else { Style::default() }),
            Line::from(""),
        ]));
    }

    let list_widget = List::new(list_items).block(opt_block);
    frame.render_widget(list_widget, body_chunks[0]);

    // Right Panel: Console Log
    let log_block = Block::default()
        .title(" Execution Console Log ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let mut log_lines = Vec::new();
    if state.opt_in_progress {
        log_lines.push(Line::from(Span::styled(
            "Optimization run in progress...",
            Style::default().fg(state.theme.warning),
        )));
    } else if state.opt_results.is_empty() {
        log_lines.push(Line::from(""));
        log_lines.push(Line::from(Span::styled(
            "  Console Idle.",
            Style::default().fg(Color::DarkGray),
        )));
        log_lines.push(Line::from(""));
        log_lines.push(Line::from(Span::styled(
            "  Use [Space] to select scripts,",
            Style::default().fg(state.theme.ink),
        )));
        log_lines.push(Line::from(Span::styled(
            "  then press [O] to execute selected items.",
            Style::default().fg(state.theme.ink),
        )));
    } else {
        log_lines.push(Line::from(Span::styled(
            "Execution completed:",
            Style::default()
                .fg(state.theme.success)
                .add_modifier(Modifier::BOLD),
        )));
        log_lines.push(Line::from(""));
        for res in &state.opt_results {
            if res.starts_with("  -> Error") {
                log_lines.push(Line::from(Span::styled(
                    res.clone(),
                    Style::default().fg(state.theme.error),
                )));
            } else if res.starts_with("  -> Successfully")
                || res.starts_with("  -> Done")
                || res.starts_with("  -> Executed")
            {
                log_lines.push(Line::from(Span::styled(
                    res.clone(),
                    Style::default().fg(state.theme.success),
                )));
            } else {
                log_lines.push(Line::from(Span::styled(
                    res.clone(),
                    Style::default().fg(state.theme.ink),
                )));
            }
        }
    }

    let log_widget = Paragraph::new(log_lines)
        .block(log_block)
        .wrap(Wrap { trim: true });
    frame.render_widget(log_widget, body_chunks[1]);

    // --- FOOTER ---
    let footer_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));
    let footer_text = Line::from(vec![
        Span::styled(
            " Esc ",
            Style::default()
                .fg(state.theme.error)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Back to Menu │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " Arrows/jk ",
            Style::default()
                .fg(state.theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Navigate │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " Space ",
            Style::default()
                .fg(state.theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Select/Deselect │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " O ",
            Style::default()
                .fg(state.theme.warning)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Run Selected Optimizations",
            Style::default()
                .fg(state.theme.warning)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(Paragraph::new(footer_text).block(footer_block), chunks[2]);
}

fn draw_analyze_integrated(state: &TuiState, frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Header
            Constraint::Min(10),   // List of children
            Constraint::Length(3), // Footer
        ])
        .split(area);

    // --- HEADER ---
    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));
    let header_inner = header_block.inner(chunks[0]);
    frame.render_widget(header_block, chunks[0]);

    let header_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(30), Constraint::Length(MASCOT_WIDTH + 4)])
        .split(header_inner);

    let current_path = if state.analyze_arena.is_empty() {
        std::path::PathBuf::from(".")
    } else {
        state.analyze_arena[state.analyze_current_dir_idx]
            .path
            .clone()
    };

    let total_size = if state.analyze_arena.is_empty() {
        0
    } else {
        state.analyze_arena[state.analyze_current_dir_idx].size_bytes
    };

    let header_text = vec![
        Line::from(vec![
            Span::styled(
                " Interactive Disk Analyzer ",
                Style::default()
                    .fg(state.theme.ink)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("— Total Size: {}", format_size(total_size)),
                Style::default().fg(state.theme.success),
            ),
        ]),
        Line::from(vec![
            Span::styled("Current Path: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                current_path.display().to_string(),
                Style::default().fg(state.theme.accent),
            ),
        ]),
    ];
    frame.render_widget(Paragraph::new(header_text), header_split[0]);

    let elapsed_ms = state.start_time.elapsed().as_millis() as u64;
    let mascot_lines = get_mascot_lines(elapsed_ms, "search", state.theme);
    render_mascot_with_margins(frame, header_split[1], mascot_lines);

    // Get sorted children
    let sorted_children = if state.analyze_arena.is_empty() {
        Vec::new()
    } else {
        let mut children = state.analyze_arena[state.analyze_current_dir_idx]
            .children
            .clone();
        children.sort_by(|&a, &b| {
            state.analyze_arena[b]
                .size_bytes
                .cmp(&state.analyze_arena[a].size_bytes)
        });
        children
    };

    // --- BODY ---
    let body_block = Block::default()
        .title(" Directory Contents ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(state.theme.primary));

    let mut list_items = Vec::new();
    if sorted_children.is_empty() {
        list_items.push(ListItem::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  [This directory is empty]",
                Style::default().fg(Color::DarkGray),
            )),
        ]));
    } else {
        let parent_size = total_size;
        for (i, &child_idx) in sorted_children.iter().enumerate() {
            let child = &state.analyze_arena[child_idx];
            let is_selected = i == state.analyze_selected_idx;

            // Formatted size
            let size_str = format!("{:>10}", format_size(child.size_bytes));

            // Percentage
            let pct = if parent_size > 0 {
                (child.size_bytes as f64 / parent_size as f64) * 100.0
            } else {
                0.0
            };
            let pct_str = format!("{:>5.1}%", pct);

            // Determine bar fill color based on percentage
            let bar_color = if is_selected {
                state.theme.select_fg
            } else if pct >= 50.0 {
                state.theme.error
            } else if pct >= 20.0 {
                state.theme.warning
            } else if pct >= 5.0 {
                state.theme.accent
            } else {
                state.theme.success
            };

            // Display name & icon
            let icon = if child.is_dir { "📁" } else { "📄" };
            let display_name = if child.is_dir {
                format!(" {}/", child.name)
            } else {
                format!(" {}", child.name)
            };

            let line_style = if is_selected {
                Style::default()
                    .bg(state.theme.select_bg)
                    .fg(state.theme.select_fg)
            } else {
                Style::default()
            };

            let (size_style, pct_style, name_style) = if is_selected {
                (
                    Style::default().fg(state.theme.select_fg),
                    Style::default().fg(state.theme.select_fg),
                    Style::default()
                        .fg(state.theme.select_fg)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                (
                    Style::default().fg(state.theme.success),
                    Style::default().fg(Color::DarkGray),
                    if child.is_dir {
                        Style::default()
                            .fg(state.theme.accent)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(state.theme.ink)
                    },
                )
            };

            let mut spans = vec![Span::styled(format!(" {} ", size_str), size_style)];
            spans.extend(draw_smooth_bar_spans(
                pct,
                12,
                bar_color,
                if is_selected {
                    state.theme.select_fg
                } else {
                    Color::DarkGray
                },
            ));
            spans.push(Span::styled(format!(" {}  ", pct_str), pct_style));
            spans.push(Span::styled(
                icon,
                if is_selected {
                    Style::default().fg(state.theme.select_fg)
                } else {
                    Style::default().fg(state.theme.accent)
                },
            ));
            spans.push(Span::styled(display_name, name_style));

            list_items.push(ListItem::new(Line::from(spans).style(line_style)));
        }
    }

    let list_widget = List::new(list_items).block(body_block);
    frame.render_widget(list_widget, chunks[1]);

    // --- FOOTER ---
    let footer_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));

    let footer_text = Line::from(vec![
        Span::styled(
            " Esc ",
            Style::default()
                .fg(state.theme.error)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Back to Menu │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " Arrows/jk ",
            Style::default()
                .fg(state.theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Navigate │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " Enter/l ",
            Style::default()
                .fg(state.theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Open Folder │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " Backspace/u/h ",
            Style::default()
                .fg(state.theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Up │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " d/x ",
            Style::default()
                .fg(state.theme.error)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Move Selected to Trash",
            Style::default()
                .fg(state.theme.error)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(Paragraph::new(footer_text).block(footer_block), chunks[2]);

    // --- POPUP CONFIRMATION ---
    if state.analyze_show_confirmation {
        let popup_area = centered_rect(60, 30, area);
        frame.render_widget(Clear, popup_area);

        if !sorted_children.is_empty() && state.analyze_selected_idx < sorted_children.len() {
            let target_idx = sorted_children[state.analyze_selected_idx];
            let target = &state.analyze_arena[target_idx];

            let popup_lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    " [!] CONFIRM TRASH ACTION ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::raw("Are you sure you want to move this "),
                    Span::styled(
                        if target.is_dir { "folder" } else { "file" },
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" and all its contents to trash?"),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    target.path.display().to_string(),
                    Style::default().fg(Color::Cyan),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled(" Size: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format_size(target.size_bytes),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(""),
                render_confirm_bar(
                    state.theme,
                    state.confirm_idx,
                    "y",
                    "Trash It",
                    "n",
                    "Cancel",
                ),
            ];

            let popup_widget = Paragraph::new(popup_lines)
                .block(
                    Block::default()
                        .title(" Delete Confirmation ")
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

fn draw_status_integrated(state: &TuiState, frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Header
            Constraint::Min(10),   // Dashboard splits
            Constraint::Length(3), // Footer
        ])
        .split(area);

    // --- HEADER ---
    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));
    let header_inner = header_block.inner(chunks[0]);
    frame.render_widget(header_block, chunks[0]);

    let header_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(30), Constraint::Length(MASCOT_WIDTH + 4)])
        .split(header_inner);

    let header_text = vec![
        Line::from(vec![
            Span::styled(
                " Live System Status ",
                Style::default()
                    .fg(state.theme.ink)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "— Real-time system telemetry and process explorer",
                Style::default().fg(state.theme.ink),
            ),
        ]),
        Line::from(vec![Span::styled(
            "Monitor hardware health metrics and manage running application processes.",
            Style::default().fg(Color::DarkGray),
        )]),
    ];
    frame.render_widget(Paragraph::new(header_text), header_split[0]);

    let elapsed_ms = state.start_time.elapsed().as_millis() as u64;
    let mascot_lines = get_mascot_lines(elapsed_ms, "telemetry", state.theme);
    render_mascot_with_margins(frame, header_split[1], mascot_lines);

    // If stats are not loaded yet
    let Some(stats) = &state.sys_stats else {
        let loading_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(state.theme.primary));
        let loading_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Gathering system specifications. Please wait...",
                Style::default().fg(state.theme.warning),
            )),
        ];
        frame.render_widget(
            Paragraph::new(loading_text)
                .block(loading_block)
                .alignment(Alignment::Center),
            chunks[1],
        );
        return;
    };

    // --- BODY DASHBOARD SPLIT ---
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40), // Left: System Info / Telemetry / Disks
            Constraint::Percentage(60), // Right: Network / Top Processes
        ])
        .split(chunks[1]);

    // Left Column splits: System info (Length 8) and Telemetry + Disks (Min 8)
    let left_splits = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(8)])
        .split(body_chunks[0]);

    // System info widget
    let sys_info_block = Block::default()
        .title(" System Hardware Details ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(state.theme.primary));

    let uptime_str = get_system_uptime();
    let cpu_model_str = get_cpu_model();

    let sys_info_lines = vec![
        Line::from(vec![
            Span::styled("  OS:         ", Style::default().fg(Color::DarkGray)),
            Span::styled(&stats.os_name, Style::default().fg(state.theme.ink)),
        ]),
        Line::from(vec![
            Span::styled("  Kernel:     ", Style::default().fg(Color::DarkGray)),
            Span::styled(&stats.kernel, Style::default().fg(state.theme.ink)),
        ]),
        Line::from(vec![
            Span::styled("  Uptime:     ", Style::default().fg(Color::DarkGray)),
            Span::styled(uptime_str, Style::default().fg(state.theme.success)),
        ]),
    ];
    frame.render_widget(
        Paragraph::new(sys_info_lines).block(sys_info_block),
        left_splits[0],
    );

    // Resource Telemetry + Disks
    let telemetry_block = Block::default()
        .title(" Telemetry & Storage ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(state.theme.primary));

    let mut tel_lines = vec![
        Line::from(vec![
            Span::styled("  CPU Model:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                cpu_model_str,
                Style::default()
                    .fg(state.theme.ink)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        draw_cpu_bar_graph(stats.cpu_percent, state.theme),
        Line::from(""),
        draw_bar_graph(
            state.theme,
            "RAM",
            stats.ram_used,
            stats.ram_total,
            state.theme.success,
        ),
        Line::from(""),
    ];

    if stats.swap_total > 0 {
        tel_lines.push(draw_bar_graph(
            state.theme,
            "SWAP",
            stats.swap_used,
            stats.swap_total,
            state.theme.warning,
        ));
        tel_lines.push(Line::from(""));
    }

    tel_lines.push(Line::from(Span::styled(
        "  Storage Disks:",
        Style::default()
            .fg(state.theme.ink)
            .add_modifier(Modifier::BOLD),
    )));
    for disk in &stats.disks {
        tel_lines.push(draw_bar_graph(
            state.theme,
            &disk.target,
            disk.used,
            disk.total,
            state.theme.accent,
        ));
    }

    frame.render_widget(
        Paragraph::new(tel_lines).block(telemetry_block),
        left_splits[1],
    );

    // Right Column splits: Network & Processes
    let right_splits = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Network Speed block height
            Constraint::Min(6),    // Top Processes
        ])
        .split(body_chunks[1]);

    // Network Speeds Block
    let net_block = Block::default()
        .title(" Network Rate Monitor ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(state.theme.primary));

    let net_text = vec![
        draw_net_bar_graph(
            state.theme,
            "Download",
            state.sys_network_in_rate,
            5000.0,
            state.theme.success,
        ),
        Line::from(""),
        draw_net_bar_graph(
            state.theme,
            "Upload",
            state.sys_network_out_rate,
            2000.0,
            state.theme.accent,
        ),
    ];
    frame.render_widget(Paragraph::new(net_text).block(net_block), right_splits[0]);

    // Process List
    let proc_block = Block::default()
        .title(" Top Active Processes ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(state.theme.primary));

    let mut proc_items = Vec::new();
    // Header for table
    proc_items.push(ListItem::new(vec![
        Line::from(vec![
            Span::styled(
                "  PID       ",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Span::styled(
                "CPU%      ",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Span::styled(
                "MEM%      ",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Span::styled(
                "COMMAND",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::UNDERLINED),
            ),
        ]),
        Line::from(""),
    ]));

    for (i, proc) in state.sys_top_processes.iter().enumerate() {
        let is_selected = i == state.sys_process_cursor_idx;

        let pid_str = format!("  {:>6}    ", proc.pid);
        let cpu_str = format!("{:>5.1}%    ", proc.cpu_usage);
        let mem_str = format!("{:>5.1}%    ", proc.mem_usage);
        let cmd_str = proc.command.to_string();

        let line_style = if is_selected {
            Style::default()
                .bg(state.theme.select_bg)
                .fg(state.theme.select_fg)
        } else {
            Style::default()
        };

        let (pid_style, cpu_style, mem_style, cmd_style) = if is_selected {
            (
                Style::default().fg(state.theme.select_fg),
                Style::default().fg(state.theme.select_fg),
                Style::default().fg(state.theme.select_fg),
                Style::default().fg(state.theme.select_fg),
            )
        } else {
            (
                Style::default().fg(state.theme.accent),
                Style::default().fg(state.theme.warning),
                Style::default().fg(state.theme.success),
                Style::default().fg(state.theme.ink),
            )
        };

        proc_items.push(ListItem::new(vec![
            Line::from(vec![
                Span::styled(pid_str, pid_style),
                Span::styled(cpu_str, cpu_style),
                Span::styled(mem_str, mem_style),
                Span::styled(cmd_str, cmd_style),
            ])
            .style(line_style),
        ]));
    }

    let proc_widget = List::new(proc_items).block(proc_block);
    frame.render_widget(proc_widget, right_splits[1]);

    // --- FOOTER ---
    let footer_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));

    let footer_text = Line::from(vec![
        Span::styled(
            " Esc ",
            Style::default()
                .fg(state.theme.error)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Back to Menu │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " Arrows ",
            Style::default()
                .fg(state.theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Scroll Processes │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " k ",
            Style::default()
                .fg(state.theme.error)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Kill Selected Process │ ",
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            " r ",
            Style::default()
                .fg(state.theme.warning)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Force Refresh Stats",
            Style::default()
                .fg(state.theme.warning)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(Paragraph::new(footer_text).block(footer_block), chunks[2]);
}

fn draw_goodbye(state: &TuiState, frame: &mut Frame) {
    let area = frame.area();
    let popup = centered_rect_fixed(46, 15, area);
    frame.render_widget(Clear, popup);

    let elapsed_ms = state.start_time.elapsed().as_millis() as u64;
    let remaining = 5u64.saturating_sub(elapsed_ms / 1000);
    let mascot_lines = get_mascot_lines(elapsed_ms, "celebrate", state.theme);

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Goodbye!",
            Style::default()
                .fg(state.theme.primary)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for line in mascot_lines {
        lines.push(line);
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Nibs is returning you to the terminal.",
        Style::default().fg(Color::Gray),
    )));
    lines.push(Line::from(vec![
        Span::styled("Exiting in ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            remaining.to_string(),
            Style::default()
                .fg(state.theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("...", Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(Span::styled(
        "Press any key to exit now.",
        Style::default().fg(Color::DarkGray),
    )));

    let p = Paragraph::new(lines).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(state.theme.primary)),
    );

    frame.render_widget(p, popup);
}

fn draw_clean_complete(state: &TuiState, frame: &mut Frame) {
    let area = frame.area();
    let popup = centered_rect_fixed(54, 18, area);
    frame.render_widget(Clear, popup);

    let (dest_label, dest_path) = match state.cleaned_mode.as_str() {
        "Simulated" => ("Simulated", "nothing was touched"),
        "Permanently Deleted" => ("Deleted", "files were permanently removed"),
        _ => ("Moved to Trash", "~/.local/share/Trash"),
    };

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            " Clean Complete ",
            Style::default()
                .fg(state.theme.primary)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                format!("{:<22}", dest_label),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format_size(state.cleaned_bytes),
                Style::default()
                    .fg(state.theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                format!("{:<22}", "Files moved"),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("{}", state.cleaned_count),
                Style::default()
                    .fg(state.theme.ink)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                format!("{:<22}", "Restore from"),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(dest_path, Style::default().fg(state.theme.accent)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("Protected  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "tokens, configs, sessions, memories",
                Style::default().fg(state.theme.ink),
            ),
        ]),
        Line::from(Span::styled(
            "             untouched by Nibs",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to return to the menu.",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let p = Paragraph::new(lines).alignment(Alignment::Left).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(state.theme.primary)),
    );
    frame.render_widget(p, popup);
}

fn draw_trash_manager(state: &TuiState, frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Header + Summary
            Constraint::Min(5),    // List
            Constraint::Length(3), // Footer
        ])
        .split(area);

    // Header with summary
    let total_bytes: u64 = state.trash_items.iter().map(|i| i.size_bytes).sum();
    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(state.theme.primary));
    let header_inner = header_block.inner(chunks[0]);
    frame.render_widget(header_block, chunks[0]);

    let summary_lines = if state.trash_items.is_empty() {
        vec![
            Line::from(Span::styled(
                " Trash Manager ",
                Style::default()
                    .fg(state.theme.primary)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Your trash is empty.",
                Style::default().fg(state.theme.ink),
            )),
        ]
    } else {
        vec![
            Line::from(Span::styled(
                " Trash Manager ",
                Style::default()
                    .fg(state.theme.primary)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  Current trash size  ",
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format_size(total_bytes),
                    Style::default()
                        .fg(state.theme.warning)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  Items in trash      ",
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{}", state.trash_items.len()),
                    Style::default()
                        .fg(state.theme.ink)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ]
    };
    frame.render_widget(Paragraph::new(summary_lines), header_inner);

    // List
    let items: Vec<ListItem> = state
        .trash_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = i == state.trash_selected_idx;
            let style = if is_selected {
                Style::default()
                    .bg(state.theme.select_bg)
                    .fg(state.theme.select_fg)
            } else {
                Style::default()
            };

            let name = item
                .trash_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "?".to_string());
            let orig = item
                .original_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "unknown origin".to_string());
            let size = format_size(item.size_bytes);

            let prefix = if is_selected { "● " } else { "  " };

            let (name_style, size_style, orig_style) = if is_selected {
                (
                    Style::default()
                        .fg(state.theme.select_fg)
                        .add_modifier(Modifier::BOLD),
                    Style::default().fg(state.theme.select_fg),
                    Style::default().fg(state.theme.select_fg),
                )
            } else {
                (
                    Style::default()
                        .fg(state.theme.ink)
                        .add_modifier(Modifier::BOLD),
                    Style::default().fg(state.theme.warning),
                    Style::default().fg(Color::DarkGray),
                )
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(prefix, Style::default().fg(state.theme.primary)),
                    Span::styled(name, name_style),
                    Span::styled(format!("  {}  ", size), size_style),
                    Span::styled(orig, orig_style),
                ])
                .style(style),
            ])
        })
        .collect();

    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Trashed Items ");

    if items.is_empty() {
        let empty = Paragraph::new(Line::from(Span::styled(
            "  Your trash is empty. ",
            Style::default().fg(state.theme.ink),
        )))
        .alignment(Alignment::Center)
        .block(list_block);
        frame.render_widget(empty, chunks[1]);
    } else {
        let list_widget = List::new(items).block(list_block);
        frame.render_widget(list_widget, chunks[1]);
    }

    // Footer
    let footer_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));
    let footer_text = Line::from(vec![
        Span::styled(
            " Esc ",
            Style::default()
                .fg(state.theme.error)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Back │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " ↑/↓ ",
            Style::default()
                .fg(state.theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Navigate │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " r ",
            Style::default()
                .fg(state.theme.success)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Restore │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " d ",
            Style::default()
                .fg(state.theme.error)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Delete │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " e ",
            Style::default()
                .fg(state.theme.error)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Empty All", Style::default().fg(Color::DarkGray)),
    ]);
    frame.render_widget(Paragraph::new(footer_text).block(footer_block), chunks[2]);
}

fn draw_smooth_bar_spans(
    pct: f64,
    width: usize,
    filled_color: Color,
    track_color: Color,
) -> Vec<Span<'static>> {
    let pct = pct.clamp(0.0, 100.0);
    let total_ticks = width as f64;
    let filled_ticks = (pct / 100.0) * total_ticks;
    let full_blocks = filled_ticks.floor() as usize;
    let remainder = filled_ticks - (full_blocks as f64);

    let mut spans = Vec::new();

    // Filled portion
    if full_blocks > 0 {
        spans.push(Span::styled(
            "█".repeat(full_blocks),
            Style::default().fg(filled_color),
        ));
    }

    if full_blocks < width {
        // Fractional block
        let frac_char = if remainder < 1.0 / 16.0 {
            '░'
        } else if remainder < 3.0 / 16.0 {
            '▏'
        } else if remainder < 5.0 / 16.0 {
            '▎'
        } else if remainder < 7.0 / 16.0 {
            '▍'
        } else if remainder < 9.0 / 16.0 {
            '▌'
        } else if remainder < 11.0 / 16.0 {
            '▋'
        } else if remainder < 13.0 / 16.0 {
            '▊'
        } else if remainder < 15.0 / 16.0 {
            '▉'
        } else {
            '█'
        };

        if frac_char == '█' {
            spans.push(Span::styled("█", Style::default().fg(filled_color)));
        } else if frac_char == '░' {
            spans.push(Span::styled("░", Style::default().fg(track_color)));
        } else {
            spans.push(Span::styled(
                frac_char.to_string(),
                Style::default().fg(filled_color),
            ));
        }

        // Remaining empty track
        let padded_blocks = width.saturating_sub(full_blocks + 1);
        if padded_blocks > 0 {
            spans.push(Span::styled(
                "░".repeat(padded_blocks),
                Style::default().fg(track_color),
            ));
        }
    }

    spans
}
