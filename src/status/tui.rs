use crate::tui::view::{format_size, get_mascot_lines};
use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};
use std::time::{Duration, Instant};

pub struct SystemStats {
    pub os_name: String,
    pub kernel: String,
    pub cpu_percent: f64,
    pub ram_total: u64,
    pub ram_used: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub disks: Vec<DiskInfo>,
    pub docker: DockerInfo,
}

pub struct DiskInfo {
    pub target: String,
    pub total: u64,
    pub used: u64,
    pub avail: u64,
}

pub struct DockerInfo {
    pub running: bool,
    pub details: Vec<String>,
}

fn get_cpu_ticks() -> Result<(u64, u64)> {
    let stat = std::fs::read_to_string("/proc/stat")?;
    let first_line = stat.lines().next().context("Empty /proc/stat")?;
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 5 {
        return Err(anyhow::anyhow!("Invalid /proc/stat format"));
    }

    let user: u64 = parts[1].parse()?;
    let nice: u64 = parts[2].parse()?;
    let system: u64 = parts[3].parse()?;
    let idle: u64 = parts[4].parse()?;
    let iowait: u64 = parts[5].parse().unwrap_or(0);
    let irq: u64 = parts[6].parse().unwrap_or(0);
    let softirq: u64 = parts[7].parse().unwrap_or(0);
    let steal: u64 = parts[8].parse().unwrap_or(0);

    let idle_total = idle + iowait;
    let active_total = user + nice + system + irq + softirq + steal;
    Ok((idle_total, active_total))
}

fn get_os_name() -> String {
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if let Some(stripped) = line.strip_prefix("PRETTY_NAME=") {
                return stripped.trim_matches('"').to_string();
            }
        }
    }
    "Linux OS".to_string()
}

fn get_kernel_version() -> String {
    std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "Unknown Kernel".to_string())
}

fn get_mem_info() -> Result<(u64, u64, u64, u64)> {
    let meminfo = std::fs::read_to_string("/proc/meminfo")?;
    let mut total = 0;
    let mut available = 0;
    let mut swap_total = 0;
    let mut swap_free = 0;

    for line in meminfo.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        let key = parts[0].trim_end_matches(':');
        let val: u64 = parts[1].parse().unwrap_or(0) * 1024; // convert kB to bytes

        match key {
            "MemTotal" => total = val,
            "MemAvailable" => available = val,
            "SwapTotal" => swap_total = val,
            "SwapFree" => swap_free = val,
            _ => {}
        }
    }

    let mem_used = total.saturating_sub(available);
    let swap_used = swap_total.saturating_sub(swap_free);
    Ok((total, mem_used, swap_total, swap_used))
}

fn get_disk_info() -> Vec<DiskInfo> {
    let mut disks = Vec::new();
    if let Ok(output) = std::process::Command::new("df")
        .arg("-B1")
        .arg("--output=target,size,used,avail")
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 4 {
                continue;
            }

            let target = parts[0].to_string();
            let total: u64 = parts[1].parse().unwrap_or(0);
            let used: u64 = parts[2].parse().unwrap_or(0);
            let avail: u64 = parts[3].parse().unwrap_or(0);

            if target.starts_with("/sys")
                || target.starts_with("/proc")
                || target.starts_with("/run")
                || target.starts_with("/dev")
                || target.starts_with("/boot/efi")
            {
                continue;
            }
            if total == 0 {
                continue;
            }

            disks.push(DiskInfo {
                target,
                total,
                used,
                avail,
            });
        }
    }
    disks
}

fn get_docker_info() -> DockerInfo {
    if std::process::Command::new("docker")
        .arg("--version")
        .output()
        .is_err()
    {
        return DockerInfo {
            running: false,
            details: vec!["Docker CLI is not installed".to_string()],
        };
    }

    let output = std::process::Command::new("docker")
        .arg("system")
        .arg("df")
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let details = stdout.lines().map(|s| s.to_string()).collect();
                DockerInfo {
                    running: true,
                    details,
                }
            } else {
                DockerInfo {
                    running: false,
                    details: vec!["Docker daemon is not running".to_string()],
                }
            }
        }
        Err(_) => DockerInfo {
            running: false,
            details: vec!["Failed to query Docker daemon".to_string()],
        },
    }
}

pub fn collect_stats() -> Result<SystemStats> {
    let os_name = get_os_name();
    let kernel = get_kernel_version();

    // CPU calculation (needs delta)
    let (idle1, active1) = get_cpu_ticks().unwrap_or((0, 0));
    std::thread::sleep(Duration::from_millis(40));
    let (idle2, active2) = get_cpu_ticks().unwrap_or((0, 0));

    let idle_delta = idle2.saturating_sub(idle1);
    let active_delta = active2.saturating_sub(active1);
    let total_delta = idle_delta + active_delta;
    let cpu_percent = if total_delta > 0 {
        (active_delta as f64 / total_delta as f64) * 100.0
    } else {
        0.0
    };

    let (ram_total, ram_used, swap_total, swap_used) = get_mem_info().unwrap_or((0, 0, 0, 0));
    let disks = get_disk_info();
    let docker = get_docker_info();

    Ok(SystemStats {
        os_name,
        kernel,
        cpu_percent,
        ram_total,
        ram_used,
        swap_total,
        swap_used,
        disks,
        docker,
    })
}

pub fn run_status_tui() -> Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;

    let mut should_quit = false;
    let mut last_update = Instant::now() - Duration::from_secs(2); // force immediate load
    let mut stats = collect_stats()?;
    let start_time = Instant::now();

    while !should_quit {
        if last_update.elapsed() >= Duration::from_millis(1500) {
            if let Ok(new_stats) = collect_stats() {
                stats = new_stats;
            }
            last_update = Instant::now();
        }

        terminal.draw(|f| draw_status(&stats, start_time.elapsed().as_millis() as u64, f))?;

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                break;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    should_quit = true;
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    if let Ok(new_stats) = collect_stats() {
                        stats = new_stats;
                    }
                    last_update = Instant::now();
                }
                _ => {}
            }
        }
    }

    ratatui::restore();
    Ok(())
}

fn draw_status(stats: &SystemStats, elapsed_ms: u64, frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Header
            Constraint::Min(10),   // Body
            Constraint::Length(3), // Footer
        ])
        .split(area);

    // --- RENDER HEADER ---
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
                " Nibs System Telemetry Dashboard ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("— Live Telemetry", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled("OS: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&stats.os_name, Style::default().fg(Color::White)),
            Span::styled(" │ Kernel: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&stats.kernel, Style::default().fg(Color::White)),
        ]),
    ];

    let header_widget = Paragraph::new(header_lines);
    frame.render_widget(header_widget, header_split[0]);

    // Render animated telemetry mascot
    let mascot_lines = get_mascot_lines(elapsed_ms, "telemetry", &crate::theme::NORD);
    let mascot_widget = Paragraph::new(mascot_lines).alignment(Alignment::Right);
    frame.render_widget(mascot_widget, header_split[1]);

    // --- RENDER BODY ---
    let body_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),      // CPU & RAM
            Constraint::Percentage(40), // Disks
            Constraint::Percentage(40), // Docker Info
        ])
        .split(chunks[1]);

    // Pane 1: CPU & Memory
    let cpu_mem_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" CPU & Memory ");
    let mut cpu_mem_lines = Vec::new();

    // CPU Bar
    let cpu_bar_len = ((stats.cpu_percent / 5.0).round() as usize).min(20);
    let cpu_bar = format!(
        "[{}{}] {:>5.1}%",
        "■".repeat(cpu_bar_len),
        " ".repeat(20 - cpu_bar_len),
        stats.cpu_percent
    );
    cpu_mem_lines.push(Line::from(vec![
        Span::styled("  CPU Usage  : ", Style::default().fg(Color::DarkGray)),
        Span::styled(cpu_bar, Style::default().fg(Color::Cyan)),
    ]));

    // RAM Bar
    let ram_percent = if stats.ram_total > 0 {
        (stats.ram_used as f64 / stats.ram_total as f64) * 100.0
    } else {
        0.0
    };
    let ram_bar_len = ((ram_percent / 5.0).round() as usize).min(20);
    let ram_bar = format!(
        "[{}{}] {:>5.1}% ({} / {})",
        "■".repeat(ram_bar_len),
        " ".repeat(20 - ram_bar_len),
        ram_percent,
        format_size(stats.ram_used),
        format_size(stats.ram_total)
    );
    cpu_mem_lines.push(Line::from(vec![
        Span::styled("  RAM Usage  : ", Style::default().fg(Color::DarkGray)),
        Span::styled(ram_bar, Style::default().fg(Color::Green)),
    ]));

    // Swap Bar
    let swap_percent = if stats.swap_total > 0 {
        (stats.swap_used as f64 / stats.swap_total as f64) * 100.0
    } else {
        0.0
    };
    let swap_bar_len = ((swap_percent / 5.0).round() as usize).min(20);
    let swap_bar = format!(
        "[{}{}] {:>5.1}% ({} / {})",
        "■".repeat(swap_bar_len),
        " ".repeat(20 - swap_bar_len),
        swap_percent,
        format_size(stats.swap_used),
        format_size(stats.swap_total)
    );
    cpu_mem_lines.push(Line::from(vec![
        Span::styled("  Swap Usage : ", Style::default().fg(Color::DarkGray)),
        Span::styled(swap_bar, Style::default().fg(Color::Yellow)),
    ]));

    let cpu_mem_widget = Paragraph::new(cpu_mem_lines).block(cpu_mem_block);
    frame.render_widget(cpu_mem_widget, body_chunks[0]);

    // Pane 2: Disk Mounts
    let disks_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Disk Space & Mounts ");
    let mut disk_lines = Vec::new();
    if stats.disks.is_empty() {
        disk_lines.push(Line::from("  No disks found."));
    } else {
        disk_lines.push(Line::from(vec![Span::styled(
            format!(
                "  {:<25} {:<12} {:<12} {:<12} Usage",
                "Mount Point", "Total", "Used", "Available"
            ),
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::UNDERLINED),
        )]));
        for disk in &stats.disks {
            let pct = (disk.used as f64 / disk.total.max(1) as f64) * 100.0;
            let bar_len = ((pct / 10.0).round() as usize).min(10);
            let bar = format!(
                "[{}{}] {:.1}%",
                "■".repeat(bar_len),
                " ".repeat(10 - bar_len),
                pct
            );

            disk_lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:<25} ", disk.target),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{:<12} ", format_size(disk.total)),
                    Style::default().fg(Color::Gray),
                ),
                Span::styled(
                    format!("{:<12} ", format_size(disk.used)),
                    Style::default().fg(Color::Red),
                ),
                Span::styled(
                    format!("{:<12} ", format_size(disk.avail)),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(
                    bar,
                    Style::default().fg(if pct > 85.0 { Color::Red } else { Color::Cyan }),
                ),
            ]));
        }
    }
    let disks_widget = Paragraph::new(disk_lines).block(disks_block);
    frame.render_widget(disks_widget, body_chunks[1]);

    // Pane 3: Docker Info
    let docker_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Docker Containers & Storage ");
    let mut docker_lines = Vec::new();
    if stats.docker.running {
        for line in &stats.docker.details {
            docker_lines.push(Line::from(format!("  {}", line)));
        }
    } else {
        docker_lines.push(Line::from(vec![Span::styled(
            format!(
                "  [Offline] {}",
                stats
                    .docker
                    .details
                    .first()
                    .unwrap_or(&"Docker not running".to_string())
            ),
            Style::default().fg(Color::Red),
        )]));
        docker_lines.push(Line::from(""));
        docker_lines.push(Line::from("  Ensure Docker service is running if you want to scan Docker container image/volume waste."));
    }
    let docker_widget = Paragraph::new(docker_lines)
        .block(docker_block)
        .wrap(Wrap { trim: true });
    frame.render_widget(docker_widget, body_chunks[2]);

    // --- RENDER FOOTER ---
    let footer_text = Line::from(vec![Span::styled(
        " Live values updated every 1.5 seconds. ",
        Style::default().fg(Color::White),
    )]);

    let footer_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Line::from(vec![
            Span::styled(
                " Q / Esc ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Back to CLI │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                " R ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Force Refresh System Telemetry",
                Style::default().fg(Color::DarkGray),
            ),
        ]));

    let footer_widget = Paragraph::new(footer_text).block(footer_block);
    frame.render_widget(footer_widget, chunks[2]);
}
