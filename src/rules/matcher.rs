use std::path::Path;

/// A lightweight, high-performance wildcard (glob) matcher.
/// Supports '*' representing zero or more characters.
pub fn matches_glob(pattern: &str, text: &str) -> bool {
    let pattern_bytes = pattern.as_bytes();
    let text_bytes = text.as_bytes();
    let mut p = 0;
    let mut t = 0;
    let mut star_p = None;
    let mut match_t = 0;

    while t < text_bytes.len() {
        if p < pattern_bytes.len()
            && (pattern_bytes[p] == b'*' || pattern_bytes[p] == text_bytes[t])
        {
            if pattern_bytes[p] == b'*' {
                star_p = Some(p);
                match_t = t;
                p += 1;
            } else {
                p += 1;
                t += 1;
            }
        } else if let Some(sp) = star_p {
            p = sp + 1;
            match_t += 1;
            t = match_t;
        } else {
            return false;
        }
    }

    while p < pattern_bytes.len() && pattern_bytes[p] == b'*' {
        p += 1;
    }

    p == pattern_bytes.len()
}

/// Checks if a given filesystem path matches a rule pattern.
pub fn matches_pattern(path: &Path, pattern: &str) -> bool {
    let path_str = path.to_string_lossy();
    let normalized_path = path_str.replace('\\', "/");
    let normalized_pattern = pattern.replace('\\', "/");

    // Check if the pattern starts with "**/", which means we match a suffix
    if let Some(suffix_pattern) = normalized_pattern.strip_prefix("**/") {
        // We match if there is any suffix starting at a component boundary that matches suffix_pattern
        let path_clean = normalized_path
            .strip_prefix('/')
            .unwrap_or(&normalized_path);

        let mut current = path_clean;
        loop {
            if matches_glob(suffix_pattern, current) {
                return true;
            }
            if let Some(idx) = current.find('/') {
                current = &current[idx + 1..];
            } else {
                break;
            }
        }
        false
    } else {
        // Absolute or direct match (the entire path must match the pattern)
        let path_clean = normalized_path
            .strip_prefix('/')
            .unwrap_or(&normalized_path);
        let pattern_clean = normalized_pattern
            .strip_prefix('/')
            .unwrap_or(&normalized_pattern);
        matches_glob(pattern_clean, path_clean)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_matches_pattern() {
        assert!(matches_pattern(
            Path::new("/home/user/project/node_modules"),
            "**/node_modules"
        ));
        assert!(matches_pattern(
            Path::new("/home/user/.cargo/registry"),
            "**/.cargo/registry"
        ));
        assert!(matches_pattern(
            Path::new("/home/user/file.pyc"),
            "**/*.pyc"
        ));
        assert!(!matches_pattern(
            Path::new("/home/user/file.py"),
            "**/*.pyc"
        ));
        assert!(!matches_pattern(
            Path::new("/home/user/node_modules_fake"),
            "**/node_modules"
        ));
        assert!(matches_pattern(
            Path::new("/var/log/syslog.1.gz"),
            "**/var/log/*.gz"
        ));
        assert!(matches_pattern(
            Path::new("/var/log/nginx/access.1.gz"),
            "**/var/log/*.gz"
        ));
        assert!(!matches_pattern(
            Path::new("/home/user/var/log/file.txt"),
            "**/var/log/*.gz"
        ));
    }
}
