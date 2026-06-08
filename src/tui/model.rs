use crate::findings::Finding;
use crate::safety::ScanScope;
use crate::scanner::ScanWarning;
use crate::theme::NibbleTheme;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingFilter {
    All,
    Recommended,
    Safe,
    Review,
    Selected,
}

impl FindingFilter {
    pub fn label(self) -> &'static str {
        match self {
            FindingFilter::All => "All",
            FindingFilter::Recommended => "Recommended",
            FindingFilter::Safe => "Safe",
            FindingFilter::Review => "Review",
            FindingFilter::Selected => "Selected",
        }
    }

    pub fn next(self) -> Self {
        match self {
            FindingFilter::All => FindingFilter::Recommended,
            FindingFilter::Recommended => FindingFilter::Safe,
            FindingFilter::Safe => FindingFilter::Review,
            FindingFilter::Review => FindingFilter::Selected,
            FindingFilter::Selected => FindingFilter::All,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingViewMode {
    Grouped,
    Advanced,
}

impl FindingViewMode {
    pub fn label(self) -> &'static str {
        match self {
            FindingViewMode::Grouped => "Grouped",
            FindingViewMode::Advanced => "Advanced",
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            FindingViewMode::Grouped => FindingViewMode::Advanced,
            FindingViewMode::Advanced => FindingViewMode::Grouped,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub cpu_usage: f32,
    pub mem_usage: f32,
    pub command: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiScreen {
    Home,
    HomeConfirmTrash,
    Wizard,
    Dashboard,
    SmartClean,
    AppUninstallSelector,
    AppUninstallList,
    Scanning,
    Optimize,
    Analyze,
    Status,
    Settings,
    Goodbye,
    CleanComplete,
    TrashManager,
}

pub struct TuiState {
    pub target_path: PathBuf,
    pub scope: ScanScope,
    pub findings: Vec<Finding>,
    pub warnings: Vec<ScanWarning>,
    pub selected_idx: usize,
    pub show_explanation: bool,
    pub show_confirmation: bool,
    pub dry_run: bool,
    pub selected_findings: HashSet<usize>, // Checkboxes for cleanup selection
    pub finding_filter: FindingFilter,
    pub finding_view_mode: FindingViewMode,
    pub search_active: bool,
    pub search_query: String,
    pub should_quit: bool,
    pub status_message: String,
    pub screen: TuiScreen,
    pub wizard_idx: usize, // 0 for Safe, 1 for Deep, 2 for Custom
    pub tick: usize,
    pub start_time: std::time::Instant,
    pub is_home_mode: bool,
    pub is_smart_clean: bool,

    // Last scan tracking (for Today panel)
    pub last_scan_time: Option<std::time::Instant>,
    pub last_scan_findings: usize,
    pub last_scan_recommended: usize,
    pub last_scan_size: u64,

    // Home screen & Sub-modules TUI state
    pub home_selected_idx: usize, // Selected index in the Home menu (0: Smart Clean, 1: Deep Clean, 2: Analyze Disk, 3: Apps & Leftovers, 4: Optimize System, 5: Trash, 6: Settings, 7: Exit)
    pub app_name: String,
    pub app_remnants: Vec<(PathBuf, u64)>, // Path and size of remnants
    pub selected_remnants: HashSet<usize>, // Selected indices for app leftovers cleaning

    // Async scanning state
    pub scan_rx: Option<std::sync::mpsc::Receiver<crate::scanner::walker::ScanProgress>>,
    pub scan_files_count: u64,
    pub scan_findings_count: usize,
    pub scan_total_size: u64,
    pub scan_current_path: PathBuf,

    // App selector state
    pub installed_apps: Vec<crate::uninstall::apps::InstalledApp>,
    pub selected_app_idx: usize,

    // Optimize screen state
    pub opt_selected_indices: HashSet<usize>,
    pub opt_cursor_idx: usize,
    pub opt_in_progress: bool,
    pub opt_results: Vec<String>,

    // Integrated Analyze (Disk Tree) state
    pub analyze_arena: Vec<crate::analyze::tree::ArenaNode>,
    pub analyze_current_dir_idx: usize,
    pub analyze_selected_idx: usize,
    pub analyze_history: Vec<(usize, usize)>,
    pub analyze_show_confirmation: bool,
    pub analyze_warnings: Vec<String>,

    // Integrated Status screen state
    pub sys_stats: Option<crate::status::tui::SystemStats>,
    pub sys_last_update: std::time::Instant,
    pub sys_top_processes: Vec<ProcessInfo>,
    pub sys_process_cursor_idx: usize,
    pub sys_network_in_rate: f64,
    pub sys_network_out_rate: f64,
    pub sys_net_last_rx_bytes: u64,
    pub sys_net_last_tx_bytes: u64,
    pub sys_net_last_update: std::time::Instant,

    // Settings state
    pub delete_directly: bool,
    pub shred: bool,
    pub theme: &'static NibbleTheme,
    pub settings_cursor_idx: usize,
    pub confirm_idx: usize,

    // Clean complete state
    pub cleaned_bytes: u64,
    pub cleaned_count: usize,
    pub cleaned_mode: String,

    // Trash manager state
    pub trash_items: Vec<crate::cleaner::trash::TrashItem>,
    pub trash_selected_idx: usize,
}

impl TuiState {
    pub fn new(
        target_path: PathBuf,
        scope: ScanScope,
        findings: Vec<Finding>,
        warnings: Vec<ScanWarning>,
        dry_run: bool,
    ) -> Self {
        let has_findings = !findings.is_empty();
        Self {
            target_path,
            scope,
            findings,
            warnings,
            selected_idx: 0,
            show_explanation: false,
            show_confirmation: false,
            dry_run,
            selected_findings: HashSet::new(),
            finding_filter: FindingFilter::All,
            finding_view_mode: FindingViewMode::Grouped,
            search_active: false,
            search_query: String::new(),
            should_quit: false,
            status_message: "".to_string(),
            screen: if has_findings {
                TuiScreen::Wizard
            } else {
                TuiScreen::Dashboard
            },
            wizard_idx: 0,
            tick: 0,
            start_time: std::time::Instant::now(),
            is_home_mode: false,
            is_smart_clean: false,

            last_scan_time: None,
            last_scan_findings: 0,
            last_scan_recommended: 0,
            last_scan_size: 0,

            home_selected_idx: 0,
            app_name: String::new(),
            app_remnants: Vec::new(),
            selected_remnants: HashSet::new(),

            delete_directly: false,
            shred: false,
            theme: &crate::theme::SYSTEM,
            settings_cursor_idx: 0,
            confirm_idx: 0,

            cleaned_bytes: 0,
            cleaned_count: 0,
            cleaned_mode: String::new(),

            trash_items: Vec::new(),
            trash_selected_idx: 0,

            scan_rx: None,
            scan_files_count: 0,
            scan_findings_count: 0,
            scan_total_size: 0,
            scan_current_path: PathBuf::new(),

            installed_apps: Vec::new(),
            selected_app_idx: 0,

            opt_selected_indices: HashSet::new(),
            opt_cursor_idx: 0,
            opt_in_progress: false,
            opt_results: Vec::new(),

            analyze_arena: Vec::new(),
            analyze_current_dir_idx: 0,
            analyze_selected_idx: 0,
            analyze_history: Vec::new(),
            analyze_show_confirmation: false,
            analyze_warnings: Vec::new(),

            sys_stats: None,
            sys_last_update: std::time::Instant::now() - std::time::Duration::from_secs(10),
            sys_top_processes: Vec::new(),
            sys_process_cursor_idx: 0,
            sys_network_in_rate: 0.0,
            sys_network_out_rate: 0.0,
            sys_net_last_rx_bytes: 0,
            sys_net_last_tx_bytes: 0,
            sys_net_last_update: std::time::Instant::now(),
        }
    }

    pub fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    pub fn visible_finding_indices(&self) -> Vec<usize> {
        self.findings
            .iter()
            .enumerate()
            .filter_map(|(idx, finding)| {
                let visible = match self.finding_filter {
                    FindingFilter::All => true,
                    FindingFilter::Recommended => finding.is_recommended_clean(),
                    FindingFilter::Safe => finding.is_safe_clean_candidate(),
                    FindingFilter::Review => finding.risk == crate::findings::RiskLevel::Review,
                    FindingFilter::Selected => self.selected_findings.contains(&idx),
                };
                if !visible {
                    return None;
                }

                let query = self.search_query.trim().to_lowercase();
                if query.is_empty() {
                    return Some(idx);
                }

                let haystack = format!(
                    "{} {} {} {} {} {} {}",
                    finding.path.display(),
                    finding.rule_id,
                    finding.rule_name,
                    finding.category,
                    finding.risk,
                    finding.cleaner_name.as_deref().unwrap_or(""),
                    finding
                        .safety_class
                        .map(|class| class.to_string())
                        .unwrap_or_default()
                )
                .to_lowercase();
                haystack.contains(&query).then_some(idx)
            })
            .collect()
    }

    pub fn selected_visible_position(&self) -> Option<usize> {
        match self.finding_view_mode {
            FindingViewMode::Grouped => self
                .visible_finding_groups()
                .iter()
                .position(|group| group.contains(&self.selected_idx)),
            FindingViewMode::Advanced => self
                .visible_finding_indices()
                .iter()
                .position(|idx| *idx == self.selected_idx),
        }
    }

    pub fn filter_count(&self) -> usize {
        match self.finding_view_mode {
            FindingViewMode::Grouped => self.visible_finding_groups().len(),
            FindingViewMode::Advanced => self.visible_finding_indices().len(),
        }
    }

    pub fn visible_finding_groups(&self) -> Vec<Vec<usize>> {
        let mut groups: Vec<Vec<usize>> = Vec::new();

        for idx in self.visible_finding_indices() {
            let finding = &self.findings[idx];
            if let Some(group) = groups.iter_mut().find(|group| {
                group
                    .first()
                    .and_then(|first_idx| self.findings.get(*first_idx))
                    .is_some_and(|first| first.rule_id == finding.rule_id)
            }) {
                group.push(idx);
            } else {
                groups.push(vec![idx]);
            }
        }

        groups.sort_by(|a, b| {
            let size_a: u64 = a.iter().map(|idx| self.findings[*idx].size_bytes).sum();
            let size_b: u64 = b.iter().map(|idx| self.findings[*idx].size_bytes).sum();
            size_b.cmp(&size_a).then_with(|| {
                self.findings[a[0]]
                    .rule_name
                    .cmp(&self.findings[b[0]].rule_name)
            })
        });
        groups
    }

    pub fn selected_size_bytes(&self) -> u64 {
        self.selected_findings
            .iter()
            .filter_map(|idx| self.findings.get(*idx))
            .map(|finding| finding.size_bytes)
            .sum()
    }

    pub fn recommended_summary(&self) -> (usize, u64) {
        self.findings
            .iter()
            .filter(|finding| finding.is_recommended_clean())
            .fold((0, 0), |(count, size), finding| {
                (count + 1, size + finding.size_bytes)
            })
    }

    pub fn clamp_selected_to_filter(&mut self) {
        match self.finding_view_mode {
            FindingViewMode::Grouped => {
                let groups = self.visible_finding_groups();
                if groups.is_empty() {
                    self.selected_idx = 0;
                } else if !groups
                    .iter()
                    .any(|group| group.contains(&self.selected_idx))
                {
                    self.selected_idx = groups[0][0];
                }
            }
            FindingViewMode::Advanced => {
                let visible = self.visible_finding_indices();
                if visible.is_empty() {
                    self.selected_idx = 0;
                } else if !visible.contains(&self.selected_idx) {
                    self.selected_idx = visible[0];
                }
            }
        }
    }

    pub fn move_finding_selection(&mut self, delta: isize) {
        match self.finding_view_mode {
            FindingViewMode::Grouped => {
                let groups = self.visible_finding_groups();
                if groups.is_empty() {
                    self.selected_idx = 0;
                    return;
                }

                let current_pos = groups
                    .iter()
                    .position(|group| group.contains(&self.selected_idx))
                    .unwrap_or(0);
                let next_group_pos = if delta < 0 {
                    current_pos.saturating_sub(delta.unsigned_abs())
                } else {
                    (current_pos + delta as usize).min(groups.len() - 1)
                };
                self.selected_idx = groups[next_group_pos][0];
            }
            FindingViewMode::Advanced => {
                let visible = self.visible_finding_indices();
                if visible.is_empty() {
                    self.selected_idx = 0;
                    return;
                }

                let current_pos = visible
                    .iter()
                    .position(|idx| *idx == self.selected_idx)
                    .unwrap_or(0);
                let next_pos = if delta < 0 {
                    current_pos.saturating_sub(delta.unsigned_abs())
                } else {
                    (current_pos + delta as usize).min(visible.len() - 1)
                };
                self.selected_idx = visible[next_pos];
            }
        }
    }

    pub fn cycle_finding_filter(&mut self) {
        self.finding_filter = self.finding_filter.next();
        self.clamp_selected_to_filter();
        self.status_message = format!(
            "Filter: {} ({} visible)",
            self.finding_filter.label(),
            self.filter_count()
        );
    }

    pub fn clear_search(&mut self) {
        self.search_active = false;
        self.search_query.clear();
        self.clamp_selected_to_filter();
        self.status_message = "Search cleared.".to_string();
    }

    pub fn toggle_view_mode(&mut self) {
        self.finding_view_mode = self.finding_view_mode.toggle();
        self.clamp_selected_to_filter();
        self.status_message = format!("View mode: {}", self.finding_view_mode.label());
    }

    /// Selects findings the rules explicitly mark as safe default cleanup.
    pub fn select_all_safe(&mut self) {
        for (idx, finding) in self.findings.iter().enumerate() {
            if finding.is_recommended_clean() {
                self.selected_findings.insert(idx);
            }
        }
        self.status_message = format!(
            "Selected {} recommended safe items.",
            self.selected_findings.len()
        );
    }

    /// Apply the chosen wizard profile
    pub fn apply_profile(&mut self) {
        self.selected_findings.clear();
        match self.wizard_idx {
            0 => {
                for (idx, finding) in self.findings.iter().enumerate() {
                    if finding.is_recommended_clean() {
                        self.selected_findings.insert(idx);
                    }
                }
                self.status_message = format!(
                    "Recommended profile applied: {} low-friction items selected.",
                    self.selected_findings.len()
                );
            }
            1 => {
                for (idx, finding) in self.findings.iter().enumerate() {
                    if finding.is_safe_clean_candidate() {
                        self.selected_findings.insert(idx);
                    }
                }
                self.status_message = format!(
                    "Safe review profile applied: {} safe items selected.",
                    self.selected_findings.len()
                );
            }
            2 => {
                self.status_message =
                    "Manual review opened. Toggle exactly what you want with Space.".to_string();
            }
            _ => {}
        }
        self.screen = TuiScreen::Dashboard;
        self.clamp_selected_to_filter();
    }

    /// Toggles the selected state of the current finding.
    pub fn toggle_select_current(&mut self) {
        match self.finding_view_mode {
            FindingViewMode::Grouped => {
                let Some(group) = self
                    .visible_finding_groups()
                    .into_iter()
                    .find(|group| group.contains(&self.selected_idx))
                else {
                    return;
                };

                if group.iter().all(|idx| self.selected_findings.contains(idx)) {
                    for idx in group {
                        self.selected_findings.remove(&idx);
                    }
                } else {
                    for idx in group {
                        self.selected_findings.insert(idx);
                    }
                }
            }
            FindingViewMode::Advanced => {
                if self.findings.is_empty()
                    || !self.visible_finding_indices().contains(&self.selected_idx)
                {
                    return;
                }

                if self.selected_findings.contains(&self.selected_idx) {
                    self.selected_findings.remove(&self.selected_idx);
                } else {
                    self.selected_findings.insert(self.selected_idx);
                }
            }
        }
    }

    pub fn update_system_stats(&mut self) {
        if let Ok(stats) = crate::status::tui::collect_stats() {
            // Calculate network speed delta
            let (rx, tx) = get_network_bytes();
            let elapsed = self.sys_net_last_update.elapsed().as_secs_f64();
            if elapsed > 0.0 && self.sys_net_last_rx_bytes > 0 {
                let rx_diff = rx.saturating_sub(self.sys_net_last_rx_bytes);
                let tx_diff = tx.saturating_sub(self.sys_net_last_tx_bytes);
                self.sys_network_in_rate = (rx_diff as f64 / 1024.0) / elapsed; // KB/s
                self.sys_network_out_rate = (tx_diff as f64 / 1024.0) / elapsed;
                // KB/s
            }
            self.sys_net_last_rx_bytes = rx;
            self.sys_net_last_tx_bytes = tx;
            self.sys_net_last_update = std::time::Instant::now();

            self.sys_stats = Some(stats);
            self.sys_top_processes = get_top_processes();
        }
    }
}

fn get_network_bytes() -> (u64, u64) {
    let mut total_rx = 0;
    let mut total_tx = 0;
    if let Ok(content) = std::fs::read_to_string("/proc/net/dev") {
        for line in content.lines().skip(2) {
            let Some((iface, data)) = line.split_once(':') else {
                continue;
            };
            if iface.trim() == "lo" {
                continue;
            }
            let parts: Vec<&str> = data.split_whitespace().collect();
            if parts.len() >= 9 {
                let rx: u64 = parts[0].parse().unwrap_or(0);
                let tx: u64 = parts[8].parse().unwrap_or(0);
                total_rx += rx;
                total_tx += tx;
            }
        }
    }
    (total_rx, total_tx)
}

fn get_top_processes() -> Vec<ProcessInfo> {
    let mut list = Vec::new();
    if let Ok(output) = std::process::Command::new("ps")
        .arg("-eo")
        .arg("pid,pcpu,pmem,comm")
        .arg("--sort=-pcpu")
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let pid: u32 = parts[0].parse().unwrap_or(0);
                let cpu_usage: f32 = parts[1].parse().unwrap_or(0.0);
                let mem_usage: f32 = parts[2].parse().unwrap_or(0.0);
                let command = parts[3..].join(" ");
                if pid > 0 {
                    list.push(ProcessInfo {
                        pid,
                        cpu_usage,
                        mem_usage,
                        command,
                    });
                }
            }
        }
    }
    list
}
