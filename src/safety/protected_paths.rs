use std::path::Path;

/// Checks if a path is a protected Linux path that must never be scanned or cleaned.
pub fn is_protected_path(path: &Path) -> bool {
    // Path::starts_with matches components. e.g. /proc/cpuinfo starts with /proc,
    // but /home/user/proc does not.
    let protected_prefixes = [
        Path::new("/proc"),
        Path::new("/sys"),
        Path::new("/dev"),
        Path::new("/run"),
        Path::new("/boot"),
        Path::new("/lost+found"),
    ];

    for prefix in &protected_prefixes {
        if path.starts_with(prefix) {
            return true;
        }
    }
    false
}

/// Checks if a path belongs to a restricted system zone that requires extra caution.
pub fn is_restricted_path(path: &Path) -> bool {
    let restricted_prefixes = [
        Path::new("/etc"),
        Path::new("/bin"),
        Path::new("/sbin"),
        Path::new("/lib"),
        Path::new("/lib64"),
        Path::new("/usr/bin"),
        Path::new("/usr/sbin"),
        Path::new("/usr/lib"),
        Path::new("/var/lib"),
        Path::new("/snap"),
        Path::new("/flatpak"),
    ];

    for prefix in &restricted_prefixes {
        if path.starts_with(prefix) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_protected_path() {
        assert!(is_protected_path(Path::new("/proc")));
        assert!(is_protected_path(Path::new("/proc/123/status")));
        assert!(is_protected_path(Path::new("/sys/class/net")));
        assert!(!is_protected_path(Path::new("/home/user/proc")));
        assert!(!is_protected_path(Path::new("/var/log/sys")));
    }

    #[test]
    fn test_is_restricted_path() {
        assert!(is_restricted_path(Path::new("/etc")));
        assert!(is_restricted_path(Path::new("/etc/hosts")));
        assert!(is_restricted_path(Path::new("/usr/bin/cargo")));
        assert!(!is_restricted_path(Path::new("/home/user/etc")));
    }
}
