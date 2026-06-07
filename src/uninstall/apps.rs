use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct InstalledApp {
    pub name: String,
    pub exec: String,
    pub desktop_file: PathBuf,
}

/// Discovers installed applications by walking .desktop directories and parsing launcher files.
pub fn discover_installed_apps() -> Vec<InstalledApp> {
    let mut apps = Vec::new();
    let mut search_paths = Vec::new();

    // Standard desktop entries directories in Linux
    search_paths.push(PathBuf::from("/usr/share/applications"));
    search_paths.push(PathBuf::from("/usr/local/share/applications"));
    if let Some(home) = std::env::var("HOME").ok().map(PathBuf::from) {
        search_paths.push(home.join(".local/share/applications"));
    }

    for dir in search_paths {
        if !dir.is_dir() {
            continue;
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "desktop")
                    && let Some(app) = parse_desktop_file(&path)
                {
                    apps.push(app);
                }
            }
        }
    }

    // Deduplicate by name case-insensitively, keeping the user-specific one (~/.local/share) or first found
    apps.sort_by_key(|a| a.name.to_lowercase());
    let mut deduped: Vec<InstalledApp> = Vec::new();
    for app in apps {
        if !deduped
            .iter()
            .any(|existing| existing.name.eq_ignore_ascii_case(&app.name))
        {
            deduped.push(app);
        }
    }

    deduped
}

fn parse_desktop_file(path: &Path) -> Option<InstalledApp> {
    let content = fs::read_to_string(path).ok()?;
    let mut name = None;
    let mut exec = None;

    // We only care about entries under [Desktop Entry] section
    let mut in_desktop_entry = false;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_desktop_entry = line == "[Desktop Entry]";
            continue;
        }

        if in_desktop_entry {
            if line.starts_with("Name=") && name.is_none() {
                name = Some(line["Name=".len()..].trim().to_string());
            } else if line.starts_with("Exec=") && exec.is_none() {
                let full_exec = line["Exec=".len()..].trim();
                // Exec lines can have arguments like "discord %U" or "/usr/bin/slack --no-sandbox"
                // Extract only the executable binary filename or base path
                let binary = full_exec.split_whitespace().next().unwrap_or("");
                // Strip quotes if present
                let binary_clean = binary.trim_matches('"').trim_matches('\'');
                let exec_name = Path::new(binary_clean)
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| binary_clean.to_string());

                exec = Some(exec_name);
            }
        }
    }

    if let (Some(n), Some(e)) = (name, exec) {
        // Exclude system tools or non-user facing helpers that are boring
        if n.is_empty()
            || e.is_empty()
            || n.contains("Settings")
            || n.contains("Installer")
            || e == "true"
            || e == "false"
        {
            return None;
        }
        Some(InstalledApp {
            name: n,
            exec: e,
            desktop_file: path.to_path_buf(),
        })
    } else {
        None
    }
}
