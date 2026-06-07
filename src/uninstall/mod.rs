pub mod apps;

use crate::cleaner::trash::move_to_trash;
use crate::scanner::size::calculate_size;
use crate::tui::view::format_size;
use anyhow::Result;
use std::path::PathBuf;
use walkdir::WalkDir;

/// Scans standard Linux application folders for directories/files matching the app name query.
pub fn find_app_remnants(app_name: &str) -> Vec<PathBuf> {
    let app_name_lower = app_name.to_lowercase();
    let mut remnants = Vec::new();

    let home = std::env::var("HOME").map(PathBuf::from).ok();
    let mut search_paths = Vec::new();

    if let Some(ref h) = home {
        // User-specific configs, caches, data
        search_paths.push((h.join(".config"), 2));
        search_paths.push((h.join(".cache"), 2));
        search_paths.push((h.join(".local/share"), 2));
        search_paths.push((h.join(".local/state"), 2));
        search_paths.push((h.join(".var/app"), 2));
        search_paths.push((h.join("snap"), 2));

        // User-specific shortcuts/launchers
        search_paths.push((h.join(".local/share/applications"), 1));
        search_paths.push((h.join("Desktop"), 1));
        search_paths.push((h.join(".local/share/flatpak/exports/share/applications"), 1));

        // User local binaries
        search_paths.push((h.join(".local/bin"), 1));

        // User flatpak apps
        search_paths.push((h.join(".local/share/flatpak/app"), 1));
    }

    // System-wide shortcuts/launchers
    search_paths.push((PathBuf::from("/usr/share/applications"), 1));
    search_paths.push((PathBuf::from("/usr/local/share/applications"), 1));
    search_paths.push((
        PathBuf::from("/var/lib/flatpak/exports/share/applications"),
        1,
    ));
    search_paths.push((PathBuf::from("/var/lib/snapd/desktop/applications"), 1));

    // System-wide binaries
    search_paths.push((PathBuf::from("/usr/bin"), 1));
    search_paths.push((PathBuf::from("/usr/local/bin"), 1));
    search_paths.push((PathBuf::from("/snap/bin"), 1));

    // System-wide shared resources & installations
    search_paths.push((PathBuf::from("/opt"), 2));
    search_paths.push((PathBuf::from("/usr/share"), 2));
    search_paths.push((PathBuf::from("/usr/local/share"), 2));
    search_paths.push((PathBuf::from("/var/lib/flatpak/app"), 1));
    search_paths.push((PathBuf::from("/var/lib/snapd/snaps"), 1));
    search_paths.push((PathBuf::from("/snap"), 1));

    for (dir, max_depth) in search_paths {
        if !dir.is_dir() {
            continue;
        }

        let walker = WalkDir::new(&dir)
            .max_depth(max_depth)
            .min_depth(1)
            .follow_links(false);

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let path = entry.path().to_path_buf();
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if name.to_lowercase().contains(&app_name_lower) {
                remnants.push(path);
            }
        }
    }

    // Sort and deduplicate
    remnants.sort();
    remnants.dedup();

    // Filter out paths nested inside other matched paths (e.g. keep ~/.config/Slack, drop ~/.config/Slack/logs)
    let mut filtered_remnants = Vec::new();
    for path in remnants {
        let mut is_sub = false;
        for other in &filtered_remnants {
            if path.starts_with(other) {
                is_sub = true;
                break;
            }
        }
        if !is_sub {
            filtered_remnants.push(path);
        }
    }

    filtered_remnants
}

/// Orchestrates the uninstallation search and cleanup process.
pub fn run_uninstall(app_name: &str, dry_run: bool) -> Result<()> {
    println!(
        "Searching for application remnants matching: '{}'...",
        app_name
    );

    let remnants = find_app_remnants(app_name);

    if remnants.is_empty() {
        println!(
            "No remnants or leftover files found matching '{}'.",
            app_name
        );
        return Ok(());
    }

    println!("\nLocated {} remnant paths:", remnants.len());
    println!("--------------------------------------------------");
    let mut total_bytes = 0;

    for (i, path) in remnants.iter().enumerate() {
        let (size_bytes, _, _) = calculate_size(path);
        total_bytes += size_bytes;

        let type_label = if path.is_dir() { "DIR" } else { "FILE" };
        println!(
            "  [{:2}] [{:4}] {:<50} ({})",
            i + 1,
            type_label,
            path.to_string_lossy(),
            format_size(size_bytes)
        );
    }

    println!("--------------------------------------------------");
    println!("Total reclaimable size: {}", format_size(total_bytes));

    if dry_run {
        println!("\n[Dry-run] Simulated cleanup. No files were removed.");
        return Ok(());
    }

    print!(
        "\n[!] Do you want to move these {} remnants to the system trash? [y/N]: ",
        remnants.len()
    );
    use std::io::Write;
    let _ = std::io::stdout().flush();

    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_ok() {
        if input.trim().eq_ignore_ascii_case("y") {
            println!("\n--- Cleaning to System Trash ---");
            let mut success_count = 0;
            let total_count = remnants.len();
            for path in remnants {
                match move_to_trash(&path) {
                    Ok(_) => {
                        println!("  ✓ Trashed: {}", path.display());
                        success_count += 1;
                    }
                    Err(e) => {
                        println!("  ✗ Failed {}: {}", path.display(), e);
                    }
                }
            }
            println!("--------------------------------------------------");
            println!(
                "Cleanup completed. Successfully trashed {}/{} paths.",
                success_count, total_count
            );
        } else {
            println!("Cleanup aborted by user.");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_find_app_remnants() {
        let temp_dir = std::env::temp_dir().join("nibble_test_uninstall");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let config_dir = temp_dir.join(".config");
        let cache_dir = temp_dir.join(".cache");

        let app_config = config_dir.join("test-app");
        let app_config_sub = app_config.join("nested");
        let app_cache = cache_dir.join("test-app");
        let other_config = config_dir.join("other-app");

        fs::create_dir_all(&app_config_sub).unwrap();
        fs::create_dir_all(&app_cache).unwrap();
        fs::create_dir_all(&other_config).unwrap();

        // Temporarily override HOME env var so the uninstaller scans our temp dir
        let old_home = std::env::var("HOME").ok();
        unsafe {
            std::env::set_var("HOME", &temp_dir);
        }

        let remnants = find_app_remnants("test-app");

        // Restore HOME env var
        unsafe {
            if let Some(ref h) = old_home {
                std::env::set_var("HOME", h);
            } else {
                std::env::remove_var("HOME");
            }
        }

        // Clean up files
        let _ = fs::remove_dir_all(&temp_dir);

        // We expect app_config and app_cache to be matched
        // app_config_sub should be dropped because it is nested under app_config
        // other_config should be ignored
        assert_eq!(remnants.len(), 2);

        let path_strs: Vec<String> = remnants
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        assert!(path_strs.iter().any(|s| s.contains(".config/test-app")));
        assert!(path_strs.iter().any(|s| s.contains(".cache/test-app")));
        assert!(!path_strs.iter().any(|s| s.contains("nested")));
        assert!(!path_strs.iter().any(|s| s.contains("other-app")));
    }
}
