use std::collections::HashSet;
use std::fs::Metadata;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use walkdir::WalkDir;

/// Returns the amount of disk space the file currently occupies.
///
/// On Linux this uses allocated filesystem blocks instead of apparent length,
/// so sparse files and hardlinked package stores do not inflate reclaimable
/// space estimates.
pub fn metadata_disk_usage_bytes(meta: &Metadata) -> u64 {
    #[cfg(unix)]
    {
        meta.blocks().saturating_mul(512)
    }

    #[cfg(not(unix))]
    {
        meta.len()
    }
}

#[cfg(unix)]
pub fn metadata_inode_key(meta: &Metadata) -> Option<(u64, u64)> {
    if meta.nlink() > 1 {
        Some((meta.dev(), meta.ino()))
    } else {
        None
    }
}

#[cfg(not(unix))]
pub fn metadata_inode_key(_meta: &Metadata) -> Option<(u64, u64)> {
    None
}

/// Calculates the size of a path (file or directory).
/// Returns the size in bytes, the latest modified timestamp, and a list of warnings encountered during traversal.
pub fn calculate_size(path: &Path) -> (u64, Option<chrono::DateTime<chrono::Utc>>, Vec<String>) {
    let mut seen_hardlinks = HashSet::new();

    if path.is_file() {
        return match std::fs::symlink_metadata(path) {
            Ok(meta) => {
                let mtime = meta.modified().ok().map(chrono::DateTime::from);
                (metadata_disk_usage_bytes(&meta), mtime, Vec::new())
            }
            Err(e) => (
                0,
                None,
                vec![format!("Failed to read metadata for {:?}: {}", path, e)],
            ),
        };
    }

    let mut total_size: u64 = 0;
    let mut latest_mtime: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut warnings = Vec::new();

    for entry in WalkDir::new(path).follow_links(false).into_iter() {
        match entry {
            Ok(e) => match std::fs::symlink_metadata(e.path()) {
                Ok(meta) => {
                    if let Some(key) = metadata_inode_key(&meta)
                        && !seen_hardlinks.insert(key)
                    {
                        continue;
                    }

                    total_size = total_size.saturating_add(metadata_disk_usage_bytes(&meta));
                    if let Ok(sys_time) = meta.modified() {
                        let dt = chrono::DateTime::from(sys_time);
                        if latest_mtime.is_none_or(|current_latest| dt > current_latest) {
                            latest_mtime = Some(dt);
                        }
                    }
                }
                Err(err) => {
                    warnings.push(format!(
                        "Failed to read metadata for {:?}: {}",
                        e.path(),
                        err
                    ));
                }
            },
            Err(err) => {
                warnings.push(format!(
                    "Permission denied or error reading subpath: {}",
                    err
                ));
            }
        }
    }

    (total_size, latest_mtime, warnings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;

    #[test]
    fn calculates_allocated_disk_usage_for_files() {
        let temp_dir = std::env::temp_dir().join("nibble_test_allocated_size");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("tiny.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello").unwrap();

        let meta = std::fs::symlink_metadata(&file_path).unwrap();
        let (size, _, warnings) = calculate_size(&file_path);

        assert!(warnings.is_empty());
        assert_eq!(size, metadata_disk_usage_bytes(&meta));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[cfg(unix)]
    #[test]
    fn does_not_double_count_hardlinked_files_inside_one_tree() {
        let temp_dir = std::env::temp_dir().join("nibble_test_hardlink_size");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let original = temp_dir.join("original.bin");
        let link = temp_dir.join("linked.bin");
        let mut file = File::create(&original).unwrap();
        file.write_all(b"same inode").unwrap();
        fs::hard_link(&original, &link).unwrap();

        let original_meta = std::fs::symlink_metadata(&original).unwrap();
        let dir_meta = std::fs::symlink_metadata(&temp_dir).unwrap();
        let expected =
            metadata_disk_usage_bytes(&dir_meta) + metadata_disk_usage_bytes(&original_meta);
        let (size, _, warnings) = calculate_size(&temp_dir);

        assert!(warnings.is_empty());
        assert_eq!(size, expected);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
