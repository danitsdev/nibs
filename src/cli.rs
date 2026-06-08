use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "nibs",
    author = "Nibble contributors",
    version,
    about = "A safe, Rust-powered terminal cleaner for Linux developers."
)]
pub struct Cli {
    /// The path to scan. Defaults to the current directory.
    pub path: Option<PathBuf>,

    /// Run in non-TUI mode, printing findings directly to standard output.
    #[arg(long, default_value_t = false)]
    pub no_tui: bool,

    /// Execute a simulated dry-run cleanup.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    /// Output findings in JSON format. Implies --no-tui.
    #[arg(long, default_value_t = false)]
    pub json: bool,

    /// Scan for and report exact duplicate files using BLAKE3 hashing.
    #[arg(long, default_value_t = false)]
    pub detect_duplicates: bool,

    /// Minimum age of files/folders in days since last modified to consider them cleanable.
    #[arg(long, default_value = "0")]
    pub min_age: u64,

    /// Minimum finding size in MiB. Defaults to 1 MiB to avoid noisy tiny findings.
    #[arg(long, default_value = "1")]
    pub min_size: u64,

    /// Enable brute mode: scans restricted zones (like apt/pacman caches and logs) under root.
    #[arg(long, default_value_t = false)]
    pub brute: bool,

    /// Securely shred files by overwriting with zero bytes before deletion.
    #[arg(long, default_value_t = false)]
    pub shred: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Open the interactive toolbox menu.
    Home,
    /// Open the interactive cleaner flow.
    Clean {
        /// The path to clean. Defaults to the current directory.
        path: Option<PathBuf>,

        /// Run in non-TUI mode, printing findings directly to standard output.
        #[arg(long, default_value_t = false)]
        no_tui: bool,

        /// Execute a simulated dry-run cleanup.
        #[arg(long, default_value_t = false)]
        dry_run: bool,

        /// Output findings in JSON format.
        #[arg(long, default_value_t = false)]
        json: bool,

        /// Scan for and report exact duplicate files using BLAKE3 hashing.
        #[arg(long, default_value_t = false)]
        detect_duplicates: bool,

        /// Minimum age of files/folders in days since last modified to consider them cleanable.
        #[arg(long, default_value = "0")]
        min_age: u64,

        /// Minimum finding size in MiB. Defaults to 1 MiB to avoid noisy tiny findings.
        #[arg(long, default_value = "1")]
        min_size: u64,

        /// Enable brute mode: scans restricted zones (like apt/pacman caches and logs) under root.
        #[arg(long, default_value_t = false)]
        brute: bool,

        /// Securely shred files by overwriting with zero bytes before deletion.
        #[arg(long, default_value_t = false)]
        shred: bool,
    },
    /// Deep review scan for large, old, or expensive cleanup targets.
    Deep {
        /// The path to deep scan. Defaults to the user's home directory.
        path: Option<PathBuf>,

        /// Run in non-TUI mode, printing findings directly to standard output.
        #[arg(long, default_value_t = false)]
        no_tui: bool,

        /// Execute a simulated dry-run cleanup. Deep mode does not auto-select review items.
        #[arg(long, default_value_t = false)]
        dry_run: bool,

        /// Output findings in JSON format.
        #[arg(long, default_value_t = false)]
        json: bool,

        /// Disable exact duplicate detection for a faster deep scan.
        #[arg(long, default_value_t = false)]
        no_duplicates: bool,

        /// Minimum age of files/folders in days since last modified to consider them cleanable.
        #[arg(long, default_value = "7")]
        min_age: u64,

        /// Minimum finding size in MiB. Deep mode defaults to 0 MiB to surface small-but-useful caches too.
        #[arg(long, default_value = "0")]
        min_size: u64,
    },
    /// Scan a directory for cleanable files and folders.
    Scan {
        /// The path to scan. Defaults to the current directory.
        path: Option<PathBuf>,

        /// Output findings in JSON format.
        #[arg(long, default_value_t = false)]
        json: bool,

        /// Execute a simulated dry-run cleanup.
        #[arg(long, default_value_t = false)]
        dry_run: bool,

        /// Scan for and report exact duplicate files using BLAKE3 hashing.
        #[arg(long, default_value_t = false)]
        detect_duplicates: bool,

        /// Minimum age of files/folders in days since last modified to consider them cleanable.
        #[arg(long, default_value = "0")]
        min_age: u64,

        /// Minimum finding size in MiB. Defaults to 1 MiB to avoid noisy tiny findings.
        #[arg(long, default_value = "1")]
        min_size: u64,

        /// Enable brute mode: scans restricted zones (like apt/pacman caches and logs) under root.
        #[arg(long, default_value_t = false)]
        brute: bool,

        /// Securely shred files by overwriting with zero bytes before deletion.
        #[arg(long, default_value_t = false)]
        shred: bool,
    },
    /// Interactive disk analyzer (ncdu/gdu style explorer).
    Analyze {
        /// The path to analyze. Defaults to the current directory.
        path: Option<PathBuf>,
    },
    /// Live system status telemetry and dashboard.
    Status,
    /// Uninstall applications and find their leftover/remnant files.
    Uninstall {
        /// Name of the application to search and uninstall.
        app_name: String,

        /// Run in dry-run mode (only lists paths without deleting).
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
    /// Check system health, disk spaces, and cache sizes.
    Doctor,
    /// Open the Trash Manager to review, restore or empty trashed files.
    Trash,
}

#[derive(Debug, Clone)]
pub enum CliAction {
    Scan(ScanConfig),
    Analyze { path: PathBuf },
    Status,
    Uninstall { app_name: String, dry_run: bool },
    Home,
    Doctor,
    Trash,
}

#[derive(Debug, Clone)]
pub struct ScanConfig {
    pub path: PathBuf,
    pub no_tui: bool,
    pub json: bool,
    pub dry_run: bool,
    pub detect_duplicates: bool,
    pub min_age: u64,
    pub min_size: u64,
    pub brute: bool,
    pub profile: ScanProfile,
    pub shred: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanProfile {
    Smart,
    Deep,
}

impl ScanProfile {
    pub fn label(self) -> &'static str {
        match self {
            ScanProfile::Smart => "Smart Clean",
            ScanProfile::Deep => "Deep Clean",
        }
    }

    pub fn auto_selects_safe_items(self) -> bool {
        matches!(self, ScanProfile::Smart)
    }
}

impl Cli {
    pub fn resolve_action() -> CliAction {
        let cli = Cli::parse();

        if let Some(cmd) = cli.command {
            match cmd {
                Commands::Home => CliAction::Home,
                Commands::Clean {
                    path,
                    no_tui,
                    json,
                    dry_run,
                    detect_duplicates,
                    min_age,
                    min_size,
                    brute,
                    shred,
                } => CliAction::Scan(ScanConfig {
                    path: path.unwrap_or_else(|| PathBuf::from(".")),
                    no_tui: no_tui || json,
                    json,
                    dry_run,
                    detect_duplicates,
                    min_age,
                    min_size,
                    brute,
                    profile: ScanProfile::Smart,
                    shred,
                }),
                Commands::Deep {
                    path,
                    no_tui,
                    dry_run,
                    json,
                    no_duplicates,
                    min_age,
                    min_size,
                } => CliAction::Scan(ScanConfig {
                    path: path.unwrap_or_else(default_deep_path),
                    no_tui: no_tui || json,
                    json,
                    dry_run,
                    detect_duplicates: !no_duplicates,
                    min_age,
                    min_size,
                    brute: true,
                    profile: ScanProfile::Deep,
                    shred: false,
                }),
                Commands::Scan {
                    path,
                    json,
                    dry_run,
                    detect_duplicates,
                    min_age,
                    min_size,
                    brute,
                    shred,
                } => {
                    CliAction::Scan(ScanConfig {
                        path: path.unwrap_or_else(|| PathBuf::from(".")),
                        no_tui: true, // Subcommand scan is implicitly non-TUI
                        json,
                        dry_run,
                        detect_duplicates,
                        min_age,
                        min_size,
                        brute,
                        profile: ScanProfile::Smart,
                        shred,
                    })
                }
                Commands::Analyze { path } => CliAction::Analyze {
                    path: path.unwrap_or_else(|| PathBuf::from(".")),
                },
                Commands::Status => CliAction::Status,
                Commands::Uninstall { app_name, dry_run } => {
                    CliAction::Uninstall { app_name, dry_run }
                }
                Commands::Doctor => CliAction::Doctor,
                Commands::Trash => CliAction::Trash,
            }
        } else if cli.path.is_some()
            || cli.no_tui
            || cli.json
            || cli.dry_run
            || cli.detect_duplicates
            || cli.min_age > 0
            || cli.min_size != 1
            || cli.brute
            || cli.shred
        {
            let json = cli.json;
            let mut no_tui = cli.no_tui;
            if json {
                no_tui = true;
            }

            CliAction::Scan(ScanConfig {
                path: cli.path.unwrap_or_else(|| PathBuf::from(".")),
                no_tui,
                json,
                dry_run: cli.dry_run,
                detect_duplicates: cli.detect_duplicates,
                min_age: cli.min_age,
                min_size: cli.min_size,
                brute: cli.brute,
                profile: ScanProfile::Smart,
                shred: cli.shred,
            })
        } else {
            CliAction::Home
        }
    }
}

fn default_deep_path() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}
