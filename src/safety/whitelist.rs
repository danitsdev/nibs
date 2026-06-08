use std::path::{Path, PathBuf};

/// Loads the user whitelist from `~/.config/nibs/whitelist.txt` and `~/.nibsignore` if they exist.
pub fn load_user_whitelist() -> Vec<PathBuf> {
    let mut whitelist = Vec::new();
    let home = match std::env::var("HOME").ok().map(PathBuf::from) {
        Some(h) => h,
        None => return whitelist,
    };

    // Paths of candidate files
    let paths = vec![
        home.join(".config/nibs/whitelist.txt"),
        home.join(".nibsignore"),
    ];

    for path in paths {
        if let Ok(content) = std::fs::read_to_string(&path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }

                // Resolve path starting with ~/
                let resolved = if let Some(stripped) = trimmed.strip_prefix("~/") {
                    home.join(stripped)
                } else {
                    let p = PathBuf::from(trimmed);
                    if p.is_relative() { home.join(p) } else { p }
                };

                whitelist.push(resolved);
            }
        }
    }

    whitelist
}

/// Checks if the given path starts with any path prefix in the whitelist.
/// Paths are resolved to absolute paths before performing the check.
pub fn is_whitelisted_path(path: &Path, whitelist: &[PathBuf]) -> bool {
    let absolute_path = if path.is_relative() {
        if let Ok(cwd) = std::env::current_dir() {
            cwd.join(path)
        } else {
            path.to_path_buf()
        }
    } else {
        path.to_path_buf()
    };

    whitelist.iter().any(|prefix| {
        // Resolve prefix to absolute just in case
        let abs_prefix = if prefix.is_relative() {
            home_dir()
                .map(|h| h.join(prefix))
                .unwrap_or_else(|| prefix.clone())
        } else {
            prefix.clone()
        };
        absolute_path.starts_with(&abs_prefix)
    })
}

fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_is_whitelisted_path() {
        let whitelist = vec![
            PathBuf::from("/home/user/safe_dir"),
            PathBuf::from("/home/user/another/file.txt"),
        ];

        assert!(is_whitelisted_path(
            Path::new("/home/user/safe_dir"),
            &whitelist
        ));
        assert!(is_whitelisted_path(
            Path::new("/home/user/safe_dir/subdir/file"),
            &whitelist
        ));
        assert!(is_whitelisted_path(
            Path::new("/home/user/another/file.txt"),
            &whitelist
        ));
        assert!(!is_whitelisted_path(
            Path::new("/home/user/safe_dir_other"),
            &whitelist
        ));
        assert!(!is_whitelisted_path(
            Path::new("/home/user/other"),
            &whitelist
        ));
    }

    #[test]
    fn test_load_user_whitelist() {
        // Since load_user_whitelist relies on HOME, we can temporarily set it or verify it works.
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_home = std::env::temp_dir().join(format!("nibs_test_home_{}", unique_id));
        std::fs::create_dir_all(&temp_home).unwrap();

        let old_home = std::env::var("HOME").ok();
        unsafe {
            std::env::set_var("HOME", &temp_home);
        }

        // Create ~/.config/nibs directory
        let config_dir = temp_home.join(".config/nibs");
        std::fs::create_dir_all(&config_dir).unwrap();

        // Write whitelist.txt
        let whitelist_path = config_dir.join("whitelist.txt");
        let mut file = File::create(&whitelist_path).unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "/absolute/path").unwrap();
        writeln!(file, "~/relative_to_home").unwrap();
        writeln!(file).unwrap(); // empty line

        // Write .nibsignore
        let nibsignore_path = temp_home.join(".nibsignore");
        let mut file2 = File::create(&nibsignore_path).unwrap();
        writeln!(file2, "another_relative").unwrap();

        let loaded = load_user_whitelist();
        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded[0], PathBuf::from("/absolute/path"));
        assert_eq!(loaded[1], temp_home.join("relative_to_home"));
        assert_eq!(loaded[2], temp_home.join("another_relative"));

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
}
