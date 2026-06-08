pub mod trash;

use crate::findings::Finding;
use anyhow::{Context, Result};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

fn shred_file(path: &Path) -> Result<()> {
    let metadata = path
        .symlink_metadata()
        .with_context(|| format!("Failed to read metadata for shredding: {:?}", path))?;
    if metadata.is_file() {
        let size = metadata.len();
        if size > 0 {
            let mut file = OpenOptions::new()
                .write(true)
                .open(path)
                .with_context(|| format!("Failed to open file for shredding: {:?}", path))?;

            let chunk_size = 64 * 1024;
            let zeros = vec![0u8; chunk_size];
            let mut written = 0;
            while written < size {
                let to_write = std::cmp::min(size - written, chunk_size as u64) as usize;
                file.write_all(&zeros[..to_write]).with_context(|| {
                    format!(
                        "Failed to write zero bytes to file during shredding: {:?}",
                        path
                    )
                })?;
                written += to_write as u64;
            }
            file.sync_all()
                .with_context(|| format!("Failed to sync file during shredding: {:?}", path))?;
        }
    }
    std::fs::remove_file(path)
        .with_context(|| format!("Failed to remove file after shredding: {:?}", path))?;
    Ok(())
}

pub fn shred_path(path: &Path) -> Result<()> {
    let meta = path
        .symlink_metadata()
        .with_context(|| format!("Failed to read metadata for shredding: {:?}", path))?;
    if meta.is_symlink() {
        std::fs::remove_file(path)
            .with_context(|| format!("Failed to remove symlink during shredding: {:?}", path))?;
    } else if meta.is_dir() {
        let walker = walkdir::WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok());

        for entry in walker {
            let entry_path = entry.path();
            let entry_meta = entry_path.symlink_metadata();
            if let Ok(m) = entry_meta
                && m.is_file()
                && !m.is_symlink()
            {
                let _ = shred_file(entry_path);
            }
        }
        std::fs::remove_dir_all(path)
            .with_context(|| format!("Failed to remove directory after shredding: {:?}", path))?;
    } else {
        shred_file(path)?;
    }
    Ok(())
}

/// Helper to clean a path directly or move to trash based on settings.
pub fn clean_path(path: &std::path::Path, delete_directly: bool, shred: bool) -> Result<()> {
    let whitelist = crate::safety::load_user_whitelist();
    if crate::safety::is_whitelisted_path(path, &whitelist) {
        return Err(anyhow::anyhow!(
            "Aborted cleaning: path {:?} is whitelisted by user configuration",
            path
        ));
    }

    if shred {
        shred_path(path)?;
    } else if delete_directly {
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

#[derive(Debug, Clone)]
pub struct CleanupResult {
    pub bytes_freed: u64,
    pub message: String,
}

/// Simulated cleaner execution for findings.
pub fn clean_findings(
    findings: &[Finding],
    dry_run: bool,
    delete_directly: bool,
    shred: bool,
) -> Result<Vec<CleanupResult>> {
    let mut results = Vec::new();

    for finding in findings {
        if finding
            .default_action
            .as_deref()
            .is_some_and(|action| action.eq_ignore_ascii_case("never"))
        {
            results.push(CleanupResult {
                bytes_freed: 0,
                message: format!(
                    "Blocked protected item marked never-clean: {:?}",
                    finding.path
                ),
            });
            continue;
        }

        if dry_run {
            results.push(CleanupResult {
                bytes_freed: finding.size_bytes,
                message: format!("[Dry-run] Would clean: {:?}", finding.path),
            });
        } else {
            let clean_result = if finding.category == crate::findings::FindingCategory::Trash {
                trash::empty_trash_directory(&finding.path)
            } else {
                clean_path(&finding.path, delete_directly, shred)
            };

            match clean_result {
                Ok(_) => {
                    results.push(CleanupResult {
                        bytes_freed: finding.size_bytes,
                        message: format!("Successfully cleaned: {:?}", finding.path),
                    });
                }
                Err(e) => {
                    results.push(CleanupResult {
                        bytes_freed: 0,
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
    use std::path::PathBuf;

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
            last_modified: None,
        };

        let results = clean_findings(&[finding], true, false, false).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].bytes_freed, 0);
        assert!(results[0].message.contains("never-clean"));
    }

    #[test]
    fn clean_path_blocks_whitelisted() {
        let temp_home = std::env::temp_dir().join("nibble_test_clean_home");
        let _ = std::fs::remove_dir_all(&temp_home);
        std::fs::create_dir_all(&temp_home).unwrap();

        let old_home = std::env::var("HOME").ok();
        unsafe {
            std::env::set_var("HOME", &temp_home);
        }

        // Write .nibbleignore in temp_home
        let ignore_path = temp_home.join(".nibbleignore");
        std::fs::write(&ignore_path, "/tmp/blocked_path\n").unwrap();

        let target = std::path::Path::new("/tmp/blocked_path/some_cache");
        let res = clean_path(target, false, false);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("is whitelisted"));

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_home);
        if let Some(old) = old_home {
            unsafe {
                std::env::set_var("HOME", old);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
    }

    #[test]
    fn test_shred_path() {
        let temp_dir = std::env::temp_dir().join("nibble_test_shred");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test_file.txt");
        std::fs::write(&file_path, "highly secret data contents").unwrap();

        shred_path(&file_path).unwrap();
        assert!(!file_path.exists());

        let subdir = temp_dir.join("subdir");
        std::fs::create_dir_all(&subdir).unwrap();
        let subfile = subdir.join("subfile.txt");
        std::fs::write(&subfile, "some other data").unwrap();

        shred_path(&temp_dir).unwrap();
        assert!(!temp_dir.exists());
    }
}
