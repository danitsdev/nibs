use std::path::{Path, PathBuf};

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanScope {
    ProjectScan(PathBuf),
    DirectoryScan(PathBuf),
    SystemSafeScan,
}

impl ScanScope {
    /// Resolves the scanning scope based on the provided path.
    pub fn from_path(path: &Path) -> Self {
        // Resolve absolute path or canonical path
        let absolute_path = match std::fs::canonicalize(path) {
            Ok(p) => p,
            Err(_) => {
                // If path doesn't exist yet or canonicalization fails, build it relative to current dir
                if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    std::env::current_dir().unwrap_or_default().join(path)
                }
            }
        };

        if absolute_path.to_string_lossy() == "/" {
            ScanScope::SystemSafeScan
        } else {
            let current_dir = std::env::current_dir().unwrap_or_default();
            let canonical_current = std::fs::canonicalize(&current_dir).unwrap_or(current_dir);

            if absolute_path == canonical_current {
                ScanScope::ProjectScan(absolute_path)
            } else {
                ScanScope::DirectoryScan(absolute_path)
            }
        }
    }

    /// Gets the target path of the scan.
    pub fn target_path(&self) -> PathBuf {
        match self {
            ScanScope::ProjectScan(p) => p.clone(),
            ScanScope::DirectoryScan(p) => p.clone(),
            ScanScope::SystemSafeScan => PathBuf::from("/"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_scope_from_path() {
        let root = Path::new("/");
        let scope = ScanScope::from_path(root);
        assert_eq!(scope, ScanScope::SystemSafeScan);

        let current = env::current_dir().unwrap_or_default();
        let canonical_current = std::fs::canonicalize(&current).unwrap_or(current);
        let scope = ScanScope::from_path(&canonical_current);
        assert_eq!(scope, ScanScope::ProjectScan(canonical_current));

        let temp_dir = env::temp_dir();
        let scope = ScanScope::from_path(&temp_dir);
        assert!(
            matches!(scope, ScanScope::DirectoryScan(_))
                || matches!(scope, ScanScope::ProjectScan(_))
        );
    }
}
