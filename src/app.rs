use crate::analyze::run_analyze_tui;
use crate::cleaner::clean_findings;
use crate::cli::{Cli, CliAction, ScanConfig};
use crate::report::generate_json_report;
use crate::rules::load_all_rules;
use crate::safety::ScanScope;
use crate::scanner::{ScanOptions, scan_directory};
use crate::status::run_status_tui;
use crate::tui::view::format_size;
use crate::tui::{run_tui_home, run_tui_scanning};
use crate::uninstall::run_uninstall;
use anyhow::{Context, Result};

/// Entry point coordinator for running the Nibble application.
pub fn run_app() -> Result<()> {
    // 1. Resolve arguments and actions
    let action = Cli::resolve_action();

    match action {
        CliAction::Scan(config) => run_scan(config),
        CliAction::Analyze { path } => {
            // Check if directory exists and resolve absolute path
            let canonical_path = std::fs::canonicalize(&path)
                .context(format!("Failed to resolve path: {:?}", path))?;

            run_analyze_tui(&canonical_path)
        }
        CliAction::Status => run_status_tui(),
        CliAction::Uninstall { app_name, dry_run } => run_uninstall(&app_name, dry_run),
        CliAction::Home => run_tui_home(),
        CliAction::Doctor => run_doctor_cli(),
    }
}

fn run_doctor_cli() -> Result<()> {
    println!("Nibble Environment Doctor — Diagnostics");
    println!("--------------------------------------------------");
    let results = crate::doctor::run_diagnostics();
    for result in results {
        let status_color = match result.status {
            crate::doctor::CheckStatus::Ok => "✓ [PASS]",
            crate::doctor::CheckStatus::Warning => "⚠ [WARN]",
            crate::doctor::CheckStatus::Error => "✗ [FAIL]",
        };
        println!(
            "{:<8} {:<24} : {}",
            status_color, result.name, result.detail
        );
    }
    println!("--------------------------------------------------");
    Ok(())
}

fn run_scan(config: ScanConfig) -> Result<()> {
    // Determine scan scope and canonicalize paths
    let scope = ScanScope::from_path(&config.path);
    let target_path = scope.target_path();

    // Configure scanning options
    let options = ScanOptions {
        detect_duplicates: config.detect_duplicates,
        min_age_days: config.min_age,
        min_size_bytes: config.min_size.saturating_mul(1024 * 1024),
        brute: config.brute,
        ..Default::default()
    };

    // Trace status for debugging
    tracing::info!("Resolved Scope: {:?}", scope);
    tracing::info!("Scanning directory: {:?}", target_path);

    // Handle Plain Text / JSON output with a synchronous scan.
    if config.json || config.no_tui {
        let rules = load_all_rules();
        let (findings, warnings) = scan_directory(&target_path, &rules, &options);

        if config.json {
            let json_report = generate_json_report(target_path, &findings, &warnings)?;
            println!("{}", json_report);
            return Ok(());
        }

        println!("Nibble — Terminal Cleaner for Developers");
        println!("Target path : {:?}", target_path);
        println!("Scan scope  : {:?}", scope);
        println!("--------------------------------------------------");

        if findings.is_empty() {
            println!("No cleanable findings located in this directory.");
        } else {
            let mut total_bytes = 0;
            let mut recommended_bytes = 0;
            let mut recommended_count = 0;
            for (i, finding) in findings.iter().enumerate() {
                println!(
                    "[{:2}] [{:6}] [{:6}] {:<40} ({})",
                    i + 1,
                    finding.risk.to_string(),
                    finding.default_action.as_deref().unwrap_or("review"),
                    finding.path.to_string_lossy(),
                    format_size(finding.size_bytes)
                );
                total_bytes += finding.size_bytes;
                if finding.is_recommended_clean() {
                    recommended_bytes += finding.size_bytes;
                    recommended_count += 1;
                }
            }
            println!("--------------------------------------------------");
            println!("Detected size          : {}", format_size(total_bytes));
            println!(
                "Recommended cleanup    : {} across {} safe items",
                format_size(recommended_bytes),
                recommended_count
            );
        }

        if !warnings.is_empty() {
            println!("\nWarnings:");
            for warning in &warnings {
                if let Some(ref path) = warning.path {
                    println!("  [Warning] (path={:?}): {}", path, warning.message);
                } else {
                    println!("  [Warning]: {}", warning.message);
                }
            }
        }

        // Handle dry-run simulated cleaner execution
        if config.dry_run {
            let recommended: Vec<_> = findings
                .iter()
                .filter(|finding| finding.is_recommended_clean())
                .cloned()
                .collect();
            if recommended.is_empty() {
                println!("\nNo recommended safe cleanup actions selected by rule defaults.");
                return Ok(());
            }
            println!("\n--- Dry-Run Cleanup (Simulation) ---");
            let clean_results = clean_findings(&recommended, true, false)?;
            for result in &clean_results {
                println!("  {}", result.message);
            }
            let total_freed: u64 = clean_results.iter().map(|r| r.bytes_freed).sum();
            println!("--------------------------------------------------");
            println!(
                "Simulated cleanup completed. Reclaimed {}.",
                format_size(total_freed)
            );
        } else if !findings.is_empty() {
            let recommended: Vec<_> = findings
                .iter()
                .filter(|finding| finding.is_recommended_clean())
                .cloned()
                .collect();
            if recommended.is_empty() {
                println!("\nNo recommended safe cleanup actions selected by rule defaults.");
                return Ok(());
            }
            let total_bytes: u64 = recommended.iter().map(|f| f.size_bytes).sum();
            print!(
                "\nThis will move {} recommended safe items ({}) to the system trash. Proceed? [y/N]: ",
                recommended.len(),
                format_size(total_bytes)
            );
            use std::io::Write;
            let _ = std::io::stdout().flush();
            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_ok() {
                if input.trim().eq_ignore_ascii_case("y") {
                    println!("\n--- Cleaning to System Trash ---");
                    let clean_results = clean_findings(&recommended, false, false)?;
                    for result in &clean_results {
                        println!("  {}", result.message);
                    }
                    let total_freed: u64 = clean_results.iter().map(|r| r.bytes_freed).sum();
                    println!("--------------------------------------------------");
                    println!(
                        "Cleanup completed. Moved {} to trash.",
                        format_size(total_freed)
                    );
                } else {
                    println!("Cleanup aborted by user.");
                }
            }
        }

        return Ok(());
    }

    run_tui_scanning(target_path, scope, options, config.dry_run)?;

    Ok(())
}
