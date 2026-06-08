use crate::scanner::size::calculate_size;
use crate::tui::view::format_size;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub enum CheckStatus {
    Ok,
    Warning,
    Error,
}

impl std::fmt::Display for CheckStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckStatus::Ok => write!(f, "PASS"),
            CheckStatus::Warning => write!(f, "WARN"),
            CheckStatus::Error => write!(f, "FAIL"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub name: String,
    pub status: CheckStatus,
    pub detail: String,
}

pub fn run_diagnostics() -> Vec<CheckResult> {
    let mut results = vec![
        check_write_permissions(),
        check_trash_size(),
        check_journal_size(),
        check_docker_status(),
    ];

    // 5. Developer Cache Sizes Check
    results.extend(check_developer_caches());

    // 6. Failed Systemd Units Check
    results.push(check_failed_services());

    results
}

fn check_write_permissions() -> CheckResult {
    let home = std::env::var("HOME").ok().map(PathBuf::from);
    let tmp = PathBuf::from("/tmp");

    let mut details = Vec::new();
    let mut errors = 0;

    if let Some(h) = home {
        let test_file = h.join(".nibs_perm_test");
        if fs::write(&test_file, "perm_test").is_ok() {
            let _ = fs::remove_file(&test_file);
            details.push("Home directory write permissions: OK".to_string());
        } else {
            errors += 1;
            details.push("Home directory write permissions: FAILED".to_string());
        }
    } else {
        details.push("Home directory: NOT FOUND".to_string());
    }

    let tmp_file = tmp.join(".nibs_perm_test");
    if fs::write(&tmp_file, "perm_test").is_ok() {
        let _ = fs::remove_file(&tmp_file);
        details.push("/tmp directory write permissions: OK".to_string());
    } else {
        errors += 1;
        details.push("/tmp directory write permissions: FAILED".to_string());
    }

    CheckResult {
        name: "Disk Write Permissions".to_string(),
        status: if errors == 0 {
            CheckStatus::Ok
        } else {
            CheckStatus::Error
        },
        detail: details.join(" │ "),
    }
}

fn check_trash_size() -> CheckResult {
    let home = std::env::var("HOME").ok().map(PathBuf::from);
    if let Some(h) = home {
        let trash_path = h.join(".local/share/Trash");
        if trash_path.exists() {
            let (bytes, _, _) = calculate_size(&trash_path);
            let status = if bytes > 5_000_000_000 {
                CheckStatus::Warning // Over 5GB in Trash
            } else {
                CheckStatus::Ok
            };
            CheckResult {
                name: "Trash Bin Size".to_string(),
                status,
                detail: format!(
                    "Total size: {} (Path: ~/.local/share/Trash)",
                    format_size(bytes)
                ),
            }
        } else {
            CheckResult {
                name: "Trash Bin Size".to_string(),
                status: CheckStatus::Ok,
                detail: "Trash directory ~/.local/share/Trash does not exist (empty).".to_string(),
            }
        }
    } else {
        CheckResult {
            name: "Trash Bin Size".to_string(),
            status: CheckStatus::Warning,
            detail: "Could not resolve HOME directory to inspect Trash.".to_string(),
        }
    }
}

fn check_journal_size() -> CheckResult {
    // Run: journalctl --disk-usage
    let output = Command::new("journalctl").arg("--disk-usage").output();

    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
            // Expected output: "Archived and active journals take up 48.0M in the file system."
            let is_large = text.contains("G")
                || (text.contains("M") && {
                    // Check if it's > 500M
                    if let Some(pos) = text.find("take up ") {
                        let num_part = &text[pos + 8..];
                        if let Some(space_pos) = num_part.find('M') {
                            num_part[..space_pos]
                                .parse::<f64>()
                                .map(|m| m > 500.0)
                                .unwrap_or(false)
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                });

            CheckResult {
                name: "Systemd Journal Logs".to_string(),
                status: if is_large {
                    CheckStatus::Warning
                } else {
                    CheckStatus::Ok
                },
                detail: format!(
                    "{} (Tip: use 'journalctl --vacuum-time=7d' if too large)",
                    text
                ),
            }
        }
        _ => CheckResult {
            name: "Systemd Journal Logs".to_string(),
            status: CheckStatus::Ok,
            detail: "journalctl disk usage unavailable or not systemd-based.".to_string(),
        },
    }
}

fn check_docker_status() -> CheckResult {
    // Run: docker info
    let output = Command::new("docker").arg("system").arg("df").output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut summary = "Running".to_string();

            // Search for RECLAIMABLE sizes
            let lines: Vec<&str> = stdout.lines().collect();
            if lines.len() > 1 {
                for line in &lines {
                    if line.contains("Images")
                        || line.contains("Containers")
                        || line.contains("Local Volumes")
                        || line.contains("Build Cache")
                    {
                        // Check if we can parse the last column or size info
                        // E.g. "Images          5         4      1.2GB     500MB (41%)"
                        if let Some(percent_idx) = line.find('%')
                            && let Some(open_paren) = line[..percent_idx].rfind('(')
                        {
                            let pct_str = &line[open_paren + 1..percent_idx];
                            if let Ok(pct) = pct_str.parse::<u32>()
                                && pct > 0
                            {
                                summary = format!(
                                    "Running (has reclaimable caches, e.g. {})",
                                    line.trim()
                                );
                            }
                        }
                    }
                }
            }

            CheckResult {
                name: "Docker Engine Status".to_string(),
                status: CheckStatus::Ok,
                detail: summary,
            }
        }
        _ => CheckResult {
            name: "Docker Engine Status".to_string(),
            status: CheckStatus::Ok,
            detail: "Docker CLI not found or daemon not running.".to_string(),
        },
    }
}

fn check_developer_caches() -> Vec<CheckResult> {
    let home = std::env::var("HOME").ok().map(PathBuf::from);
    let mut caches = Vec::new();

    if let Some(h) = home {
        // Rust Cargo
        let cargo = h.join(".cargo");
        if cargo.exists() {
            let (bytes, _, _) = calculate_size(&cargo);
            caches.push(CheckResult {
                name: "Rust Cargo Cache".to_string(),
                status: if bytes > 10_000_000_000 {
                    CheckStatus::Warning
                } else {
                    CheckStatus::Ok
                },
                detail: format!("Size: {} (Path: ~/.cargo)", format_size(bytes)),
            });
        }

        // Python Pip
        let pip = h.join(".cache/pip");
        if pip.exists() {
            let (bytes, _, _) = calculate_size(&pip);
            caches.push(CheckResult {
                name: "Python Pip Cache".to_string(),
                status: if bytes > 3_000_000_000 {
                    CheckStatus::Warning
                } else {
                    CheckStatus::Ok
                },
                detail: format!("Size: {} (Path: ~/.cache/pip)", format_size(bytes)),
            });
        }

        // Go Build
        let go = h.join(".cache/go-build");
        if go.exists() {
            let (bytes, _, _) = calculate_size(&go);
            caches.push(CheckResult {
                name: "Go Build Cache".to_string(),
                status: if bytes > 5_000_000_000 {
                    CheckStatus::Warning
                } else {
                    CheckStatus::Ok
                },
                detail: format!("Size: {} (Path: ~/.cache/go-build)", format_size(bytes)),
            });
        }

        // NPM Cache
        let npm = h.join(".npm");
        if npm.exists() {
            let (bytes, _, _) = calculate_size(&npm);
            caches.push(CheckResult {
                name: "NPM Package Cache".to_string(),
                status: if bytes > 3_000_000_000 {
                    CheckStatus::Warning
                } else {
                    CheckStatus::Ok
                },
                detail: format!("Size: {} (Path: ~/.npm)", format_size(bytes)),
            });
        }
    }

    caches
}

fn check_failed_services() -> CheckResult {
    let system_out = Command::new("systemctl")
        .arg("--failed")
        .arg("--state=failed")
        .arg("--legend=no")
        .output();

    let user_out = Command::new("systemctl")
        .arg("--user")
        .arg("--failed")
        .arg("--state=failed")
        .arg("--legend=no")
        .output();

    let mut failed_units = Vec::new();

    if let Ok(out) = system_out
        && out.status.success()
    {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                failed_units.push(format!("system:{}", parts[0]));
            }
        }
    }

    if let Ok(out) = user_out
        && out.status.success()
    {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                failed_units.push(format!("user:{}", parts[0]));
            }
        }
    }

    let failed_count = failed_units.len();
    CheckResult {
        name: "Failed Systemd Services".to_string(),
        status: if failed_count > 0 {
            CheckStatus::Warning
        } else {
            CheckStatus::Ok
        },
        detail: if failed_count > 0 {
            format!(
                "{} failed units found: {}",
                failed_count,
                failed_units.join(", ")
            )
        } else {
            "All systemd services running normally.".to_string()
        },
    }
}
