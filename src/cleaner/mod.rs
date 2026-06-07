pub mod trash;

use crate::findings::Finding;
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Helper to clean a path directly or move to trash based on settings.
pub fn clean_path(path: &std::path::Path, delete_directly: bool) -> Result<()> {
    if delete_directly {
        if path.is_dir() {
            std::fs::remove_dir_all(path)
                .with_context(|| format!("Failed to permanently delete directory: {:?}", path))?;
        } else {
            std::fs::remove_file(path)
                .with_context(|| format!("Failed to permanently delete file: {:?}", path))?;
        }
    } else {
        trash::move_to_trash(path)?;
    }
    Ok(())
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CleanupResult {
    pub path: PathBuf,
    pub bytes_freed: u64,
    pub success: bool,
    pub message: String,
}

/// Simulated cleaner execution for findings.
pub fn clean_findings(
    findings: &[Finding],
    dry_run: bool,
    delete_directly: bool,
) -> Result<Vec<CleanupResult>> {
    let mut results = Vec::new();

    for finding in findings {
        if finding
            .default_action
            .as_deref()
            .is_some_and(|action| action.eq_ignore_ascii_case("never"))
        {
            results.push(CleanupResult {
                path: finding.path.clone(),
                bytes_freed: 0,
                success: false,
                message: format!(
                    "Blocked protected item marked never-clean: {:?}",
                    finding.path
                ),
            });
            continue;
        }

        let running_processes = if finding.block_if_running {
            let running_now = crate::cleaners::find_running_processes(&finding.process_names);
            if running_now.is_empty() {
                finding.running_processes.clone()
            } else {
                running_now
            }
        } else {
            Vec::new()
        };

        if finding.block_if_running && !running_processes.is_empty() {
            results.push(CleanupResult {
                path: finding.path.clone(),
                bytes_freed: 0,
                success: false,
                message: format!(
                    "Blocked because {} is running: {:?}",
                    running_processes.join(", "),
                    finding.path
                ),
            });
            continue;
        }

        if dry_run {
            results.push(CleanupResult {
                path: finding.path.clone(),
                bytes_freed: finding.size_bytes,
                success: true,
                message: format!("[Dry-run] Would clean: {:?}", finding.path),
            });
        } else {
            let clean_result = if finding.category == crate::findings::FindingCategory::Trash {
                trash::empty_trash_directory(&finding.path)
            } else {
                clean_path(&finding.path, delete_directly)
            };

            match clean_result {
                Ok(_) => {
                    results.push(CleanupResult {
                        path: finding.path.clone(),
                        bytes_freed: finding.size_bytes,
                        success: true,
                        message: format!("Successfully cleaned: {:?}", finding.path),
                    });
                }
                Err(e) => {
                    results.push(CleanupResult {
                        path: finding.path.clone(),
                        bytes_freed: 0,
                        success: false,
                        message: format!("Failed to clean {:?}: {}", finding.path, e),
                    });
                }
            }
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::findings::{FindingCategory, RiskLevel, SafetyClass};

    #[test]
    fn never_clean_items_are_blocked() {
        let finding = Finding {
            path: PathBuf::from("/tmp/nibble-secret"),
            size_bytes: 1024,
            category: FindingCategory::DevAiAgent,
            risk: RiskLevel::Risky,
            rule_id: "test.secret".to_string(),
            rule_name: "Secret config".to_string(),
            reason: "Protected".to_string(),
            restore: None,
            default_action: Some("never".to_string()),
            cleaner_id: Some("test".to_string()),
            cleaner_name: Some("Test".to_string()),
            safety_class: Some(SafetyClass::SecretOrAuth),
            kept: None,
            block_if_running: false,
            process_names: Vec::new(),
            running_processes: Vec::new(),
            last_modified: None,
        };

        let results = clean_findings(&[finding], true, false).unwrap();
        assert_eq!(results.len(), 1);
        assert!(!results[0].success);
        assert_eq!(results[0].bytes_freed, 0);
        assert!(results[0].message.contains("never-clean"));
    }
}
