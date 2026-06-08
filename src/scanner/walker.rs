use crate::findings::{Finding, RiskLevel};
use crate::rules::{Rule, matches_pattern};
use crate::safety::{is_protected_path, is_restricted_path};
use crate::scanner::size::calculate_size;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanWarning {
    pub path: Option<PathBuf>,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub follow_symlinks: bool,
    pub cross_filesystems: bool,
    pub detect_duplicates: bool,
    pub min_age_days: u64,
    pub min_size_bytes: u64,
    pub brute: bool,
    pub include_deep_rules: bool,
    pub whitelist: Vec<PathBuf>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            follow_symlinks: false,
            cross_filesystems: false,
            detect_duplicates: false,
            min_age_days: 0,
            min_size_bytes: 1024 * 1024,
            brute: false,
            include_deep_rules: false,
            whitelist: crate::safety::load_user_whitelist(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ScanProgress {
    FilesScanned(u64),
    CurrentPath(PathBuf),
    FindingAdded {
        path: PathBuf,
        size: u64,
    },
    Finished {
        findings: Vec<Finding>,
        warnings: Vec<ScanWarning>,
    },
}

/// Recursively scans the target directory matching paths against rules.
pub fn scan_directory(
    target_path: &Path,
    rules: &[Rule],
    options: &ScanOptions,
) -> (Vec<Finding>, Vec<ScanWarning>) {
    scan_directory_with_progress(target_path, rules, options, None)
}

/// Recursively scans the target directory matching paths against rules, sending progress events.
pub fn scan_directory_with_progress(
    target_path: &Path,
    rules: &[Rule],
    options: &ScanOptions,
    progress_tx: Option<&std::sync::mpsc::Sender<ScanProgress>>,
) -> (Vec<Finding>, Vec<ScanWarning>) {
    let mut findings = Vec::new();
    let mut warnings = Vec::new();
    let mut candidate_files = std::collections::HashMap::new();

    // Check if the target root is protected
    if is_protected_path(target_path) {
        warnings.push(ScanWarning {
            path: Some(target_path.to_path_buf()),
            message: "Target path is a system protected directory. Scan aborted.".to_string(),
        });
        if let Some(tx) = progress_tx {
            let _ = tx.send(ScanProgress::Finished {
                findings: findings.clone(),
                warnings: warnings.clone(),
            });
        }
        return (findings, warnings);
    }

    let mut walker = WalkDir::new(target_path).follow_links(options.follow_symlinks);

    // If scanning / (root) and not crossing filesystems
    let is_root = target_path.to_string_lossy() == "/";
    if is_root && !options.cross_filesystems {
        // Enforce same filesystem boundary
        walker = walker.same_file_system(true);
    }

    let mut it = walker.into_iter();
    let mut files_scanned = 0;

    while let Some(entry) = it.next() {
        files_scanned += 1;

        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                warnings.push(ScanWarning {
                    path: err.path().map(Path::to_path_buf),
                    message: err.to_string(),
                });
                continue;
            }
        };

        let path = entry.path();

        // Periodically report files scanned
        if files_scanned % 100 == 0
            && let Some(tx) = progress_tx
        {
            if tx.send(ScanProgress::FilesScanned(files_scanned)).is_err() {
                break;
            }
            if tx
                .send(ScanProgress::CurrentPath(path.to_path_buf()))
                .is_err()
            {
                break;
            }
        }

        // Skip the target path itself for rule matching
        if path == target_path {
            continue;
        }

        // Custom User Whitelist Filter: Skip user-configured whitelist paths
        if crate::safety::is_whitelisted_path(path, &options.whitelist) {
            if entry.file_type().is_dir() {
                it.skip_current_dir();
            }
            continue;
        }

        // Safety Filter: Check for protected system paths (always skip)
        if is_protected_path(path) {
            if entry.file_type().is_dir() {
                it.skip_current_dir();
            }
            continue;
        }

        // Restricted system paths: skip if not running in brute mode
        if is_restricted_path(path) && !options.brute {
            if entry.file_type().is_dir() {
                it.skip_current_dir();
            }
            continue;
        }

        // Rule matching
        let mut matched_rule = None;
        for rule in rules {
            for pattern in &rule.patterns {
                if matches_pattern(path, pattern) {
                    matched_rule = Some(rule);
                    break;
                }
            }
            if matched_rule.is_some() {
                break;
            }
        }

        if let Some(rule) = matched_rule {
            let (size_bytes, last_modified, size_warnings) = calculate_size(path);
            for sw in size_warnings {
                warnings.push(ScanWarning {
                    path: Some(path.to_path_buf()),
                    message: sw,
                });
            }

            if size_bytes < options.min_size_bytes {
                if entry.file_type().is_dir() {
                    it.skip_current_dir();
                }
                continue;
            }

            // Verify minimum age threshold (inactivity)
            if options.min_age_days > 0 {
                if let Some(mtime) = last_modified {
                    let now = chrono::Utc::now();
                    let age_days = now.signed_duration_since(mtime).num_days();
                    if age_days < options.min_age_days as i64 {
                        // Directory is active (too recently modified). Skip cleaning.
                        if entry.file_type().is_dir() {
                            it.skip_current_dir();
                        }
                        continue;
                    }
                } else {
                    // Skip if mtime is unreadable and a minimum age is enforced
                    if entry.file_type().is_dir() {
                        it.skip_current_dir();
                    }
                    continue;
                }
            }

            // Upgrade risk level if inside a restricted zone
            let mut risk = rule.risk;
            if is_restricted_path(path) && risk != RiskLevel::Risky {
                risk = RiskLevel::Review; // Elevate to review
            }

            findings.push(Finding {
                path: path.to_path_buf(),
                size_bytes,
                category: rule.category,
                risk,
                rule_id: rule.id.clone(),
                rule_name: rule.name.clone(),
                reason: rule.reason.clone(),
                restore: rule.restore.clone(),
                default_action: rule.default_action.clone(),
                cleaner_id: rule.cleaner_id.clone(),
                cleaner_name: rule.cleaner_name.clone(),
                safety_class: rule.safety_class,
                last_modified,
            });

            if let Some(tx) = progress_tx
                && tx
                    .send(ScanProgress::FindingAdded {
                        path: path.to_path_buf(),
                        size: size_bytes,
                    })
                    .is_err()
            {
                break;
            }

            // Skip descending into this directory since the directory itself matches
            if entry.file_type().is_dir() {
                it.skip_current_dir();
            }
        } else if options.detect_duplicates && entry.file_type().is_file() {
            // Collect candidate files of same size for hashing afterwards
            if let Ok(meta) = entry.metadata() {
                let size = meta.len();
                if size >= options.min_size_bytes {
                    candidate_files
                        .entry(size)
                        .or_insert_with(Vec::new)
                        .push(path.to_path_buf());
                }
            }
        }
    }

    // Process exact duplicates using BLAKE3 hashing
    if options.detect_duplicates {
        for (size, paths) in candidate_files {
            if size > 0 && paths.len() > 1 {
                let mut hashes = std::collections::HashMap::new();
                for path in paths {
                    if let Ok(hash) = crate::scanner::duplicate::calculate_blake3_hash(&path) {
                        hashes.entry(hash).or_insert_with(Vec::new).push(path);
                    }
                }

                for (_hash, dup_paths) in hashes {
                    if dup_paths.len() > 1 {
                        // Keep the first file as original, treat other matches as exact duplicates
                        let original = &dup_paths[0];
                        for dup_path in &dup_paths[1..] {
                            let last_modified = dup_path
                                .metadata()
                                .ok()
                                .and_then(|m| m.modified().ok())
                                .map(chrono::DateTime::<chrono::Utc>::from);

                            findings.push(Finding {
                                path: dup_path.clone(),
                                size_bytes: size,
                                category: crate::findings::FindingCategory::ExactDuplicate,
                                risk: RiskLevel::Review,
                                rule_id: "exact_duplicate".to_string(),
                                rule_name: "Exact duplicate file".to_string(),
                                reason: format!("Exact duplicate of: {}", original.display()),
                                restore: None,
                                default_action: Some("review".to_string()),
                                cleaner_id: None,
                                cleaner_name: None,
                                safety_class: None,
                                last_modified,
                            });
                        }
                    }
                }
            }
        }
    }

    if let Some(tx) = progress_tx {
        let _ = tx.send(ScanProgress::Finished {
            findings: findings.clone(),
            warnings: warnings.clone(),
        });
    }

    (findings, warnings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::findings::FindingCategory;
    use crate::rules::Rule;
    use std::fs::{self, File};
    use std::io::Write;

    #[test]
    fn test_duplicate_detection() {
        let temp_dir = std::env::temp_dir().join("nibs_test_duplicates");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let file1 = temp_dir.join("file1.txt");
        let file2 = temp_dir.join("file2.txt");

        let mut f1 = File::create(&file1).unwrap();
        f1.write_all(b"identical content").unwrap();

        let mut f2 = File::create(&file2).unwrap();
        f2.write_all(b"identical content").unwrap();

        let rules: Vec<Rule> = Vec::new();
        let options = ScanOptions {
            detect_duplicates: true,
            min_size_bytes: 0,
            ..Default::default()
        };

        let (findings, _warnings) = scan_directory(&temp_dir, &rules, &options);

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].category, FindingCategory::ExactDuplicate);
        assert_eq!(findings[0].risk, RiskLevel::Review);
        assert!(findings[0].reason.contains("Exact duplicate of"));
    }

    #[test]
    fn test_min_age_filtering() {
        use crate::findings::FindingCategory;

        let temp_dir = std::env::temp_dir().join("nibs_test_age");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create a fake node_modules folder
        let node_modules_dir = temp_dir.join("node_modules");
        fs::create_dir_all(&node_modules_dir).unwrap();

        let file = node_modules_dir.join("package.json");
        let mut f = File::create(&file).unwrap();
        f.write_all(b"{}").unwrap();

        let rules = vec![Rule {
            id: "node_modules".to_string(),
            name: "Node modules".to_string(),
            category: FindingCategory::RebuildableDependency,
            risk: RiskLevel::Safe,
            patterns: vec!["**/node_modules".to_string()],
            reason: "Node modules cache".to_string(),
            restore: None,
            default_action: None,
            cleaner_id: None,
            cleaner_name: None,
            safety_class: None,
        }];

        // 1. Scan with min_age_days = 7 -> since it was just created, it should be SKIPPED!
        let mut options = ScanOptions {
            min_age_days: 7,
            min_size_bytes: 0,
            ..Default::default()
        };
        let (findings, _warnings) = scan_directory(&temp_dir, &rules, &options);
        assert_eq!(findings.len(), 0);

        // 2. Scan with min_age_days = 0 -> should be FOUND!
        options.min_age_days = 0;
        let (findings, _warnings) = scan_directory(&temp_dir, &rules, &options);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "node_modules");

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_min_size_filtering() {
        use crate::findings::FindingCategory;

        let temp_dir = std::env::temp_dir().join("nibs_test_min_size");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let cache_dir = temp_dir.join(".vite");
        fs::create_dir_all(&cache_dir).unwrap();
        let file = cache_dir.join("tiny-cache.bin");
        let mut f = File::create(&file).unwrap();
        f.write_all(b"tiny").unwrap();

        let rules = vec![Rule {
            id: "vite_cache".to_string(),
            name: "Vite cache".to_string(),
            category: FindingCategory::FrameworkCache,
            risk: RiskLevel::Safe,
            patterns: vec!["**/.vite".to_string()],
            reason: "Vite cache".to_string(),
            restore: None,
            default_action: Some("clean".to_string()),
            cleaner_id: None,
            cleaner_name: None,
            safety_class: None,
        }];

        let mut options = ScanOptions {
            min_size_bytes: 1024 * 1024,
            ..Default::default()
        };
        let (findings, _warnings) = scan_directory(&temp_dir, &rules, &options);
        assert!(findings.is_empty());

        options.min_size_bytes = 0;
        let (findings, _warnings) = scan_directory(&temp_dir, &rules, &options);
        assert_eq!(findings.len(), 1);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_whitelist_filtering() {
        use crate::findings::FindingCategory;

        let temp_dir = std::env::temp_dir().join("nibs_test_whitelist");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let node_modules_dir = temp_dir.join("node_modules");
        fs::create_dir_all(&node_modules_dir).unwrap();

        let file = node_modules_dir.join("package.json");
        let mut f = File::create(&file).unwrap();
        f.write_all(b"{}").unwrap();

        let rules = vec![Rule {
            id: "node_modules".to_string(),
            name: "Node modules".to_string(),
            category: FindingCategory::RebuildableDependency,
            risk: RiskLevel::Safe,
            patterns: vec!["**/node_modules".to_string()],
            reason: "Node modules cache".to_string(),
            restore: None,
            default_action: None,
            cleaner_id: None,
            cleaner_name: None,
            safety_class: None,
        }];

        // 1. Scan with empty whitelist -> node_modules should be found.
        let mut options = ScanOptions {
            min_size_bytes: 0,
            whitelist: vec![],
            ..Default::default()
        };
        let (findings, _warnings) = scan_directory(&temp_dir, &rules, &options);
        assert_eq!(findings.len(), 1);

        // 2. Scan with node_modules dir whitelisted -> should be skipped!
        options.whitelist = vec![node_modules_dir.clone()];
        let (findings, _warnings) = scan_directory(&temp_dir, &rules, &options);
        assert_eq!(findings.len(), 0);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
