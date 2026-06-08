use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct TrashItem {
    pub trash_path: PathBuf,
    pub original_path: Option<PathBuf>,
    pub size_bytes: u64,
    pub deletion_time: Option<SystemTime>,
}

/// Lists items in the user's system trash (~/.local/share/Trash/files/).
pub fn list_trash_items() -> Vec<TrashItem> {
    let Some(home) = std::env::var("HOME").ok().map(PathBuf::from) else {
        return Vec::new();
    };
    let files_dir = home.join(".local/share/Trash/files");
    let info_dir = home.join(".local/share/Trash/info");
    if !files_dir.exists() {
        return Vec::new();
    }

    let mut items = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&files_dir) {
        for entry in entries.filter_map(Result::ok) {
            let trash_path = entry.path();
            let file_name = trash_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            // Read corresponding .trashinfo file for original path and deletion time
            let info_path = info_dir.join(format!("{}.trashinfo", file_name));
            let (original_path, deletion_time) = read_trashinfo(&info_path);

            let size_bytes = if trash_path.is_dir() {
                dir_size(&trash_path)
            } else if let Ok(meta) = trash_path.metadata() {
                meta.len()
            } else {
                0
            };

            items.push(TrashItem {
                trash_path,
                original_path,
                size_bytes,
                deletion_time,
            });
        }
    }

    // Sort by deletion time (newest first)
    items.sort_by_key(|b| std::cmp::Reverse(b.deletion_time));
    items
}

fn read_trashinfo(path: &Path) -> (Option<PathBuf>, Option<SystemTime>) {
    if !path.exists() {
        return (None, None);
    }
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return (None, None),
    };

    let mut original_path = None;
    let mut deletion_time = None;

    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("Path=") {
            // The path is URL-encoded in trashinfo
            let decoded = urlencoding_decode(rest);
            original_path = Some(PathBuf::from(decoded));
        } else if let Some(rest) = line.strip_prefix("DeletionDate=") {
            // Format: 2024-01-15T10:30:00
            if let Ok(t) = parse_trash_date(rest) {
                deletion_time = Some(t);
            }
        }
    }

    (original_path, deletion_time)
}

fn urlencoding_decode(s: &str) -> String {
    // Simple percent-decoding for trashinfo paths
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else {
            result.push(c);
        }
    }
    result
}

fn parse_trash_date(s: &str) -> Result<SystemTime> {
    // Format: 2024-01-15T10:30:00
    use std::time::Duration;
    let parts: Vec<&str> = s.split(&['T', ':', '-', '+'][..]).collect();
    if parts.len() < 6 {
        return Err(anyhow::anyhow!("invalid date format"));
    }
    let year: u64 = parts[0].parse()?;
    let month: u64 = parts[1].parse()?;
    let day: u64 = parts[2].parse()?;
    let hour: u64 = parts[3].parse()?;
    let min: u64 = parts[4].parse()?;
    let sec: u64 = parts[5].parse()?;

    // Approximate: count seconds from Unix epoch (not perfect for dates before 1970, but fine for trash)
    let days_since_epoch = (year - 1970) * 365 + (year - 1970) / 4 + day_of_year(year, month, day);
    let total_secs = days_since_epoch * 86400 + hour * 3600 + min * 60 + sec;
    Ok(SystemTime::UNIX_EPOCH + Duration::from_secs(total_secs))
}

fn day_of_year(year: u64, month: u64, day: u64) -> u64 {
    let leap = (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400);
    let days_in_months = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    days_in_months[..(month as usize - 1)].iter().sum::<u64>() + day - 1
}

fn dir_size(path: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                total = total.saturating_add(dir_size(&path));
            } else if let Ok(meta) = entry.metadata() {
                total = total.saturating_add(meta.len());
            }
        }
    }
    total
}

/// Restores a trash item to its original location.
pub fn restore_trash_item(item: &TrashItem) -> Result<()> {
    let original = item
        .original_path
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No original path recorded for this trash item"))?;

    // Ensure parent directory exists
    if let Some(parent) = original.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory: {:?}", parent))?;
    }

    std::fs::rename(&item.trash_path, original)
        .with_context(|| format!("Failed to restore {:?} to {:?}", item.trash_path, original))?;

    // Also remove the .trashinfo file
    let file_name = item.trash_path.file_name().unwrap_or_default();
    let info_dir = item
        .trash_path
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("info"));
    if let Some(info_dir) = info_dir {
        let info_path = info_dir.join(format!("{}.trashinfo", file_name.to_string_lossy()));
        let _ = std::fs::remove_file(&info_path);
    }

    Ok(())
}

/// Permanently deletes a single trash item from the system trash.
pub fn delete_trash_item(item: &TrashItem) -> Result<()> {
    // Delete the file/directory in ~/.local/share/Trash/files/
    if item.trash_path.exists() {
        if item.trash_path.is_dir() {
            std::fs::remove_dir_all(&item.trash_path).with_context(|| {
                format!(
                    "Failed to delete directory from trash: {:?}",
                    item.trash_path
                )
            })?;
        } else {
            std::fs::remove_file(&item.trash_path).with_context(|| {
                format!("Failed to delete file from trash: {:?}", item.trash_path)
            })?;
        }
    }

    // Also remove the corresponding .trashinfo file in ~/.local/share/Trash/info/
    let file_name = item.trash_path.file_name().unwrap_or_default();
    let info_dir = item
        .trash_path
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("info"));
    if let Some(info_dir) = info_dir {
        let info_path = info_dir.join(format!("{}.trashinfo", file_name.to_string_lossy()));
        if info_path.exists() {
            let _ = std::fs::remove_file(&info_path);
        }
    }

    Ok(())
}

/// Safely moves a file or directory to the system trash.
pub fn move_to_trash(path: &Path) -> Result<()> {
    tracing::info!("Moving path to trash: {:?}", path);

    // We use the trash crate's delete method which complies with FreeDesktop trash specifications on Linux
    trash::delete(path).with_context(|| format!("Failed to move {:?} to system trash", path))?;

    Ok(())
}

/// Permanently deletes the contents of the trash directory without deleting the directory itself.
pub fn empty_trash_directory(trash_path: &Path) -> Result<()> {
    tracing::info!("Emptying trash directory: {:?}", trash_path);

    let files_dir = trash_path.join("files");
    let info_dir = trash_path.join("info");

    if files_dir.exists() {
        std::fs::remove_dir_all(&files_dir)
            .with_context(|| format!("Failed to empty files directory: {:?}", files_dir))?;
        std::fs::create_dir_all(&files_dir)
            .with_context(|| format!("Failed to recreate files directory: {:?}", files_dir))?;
    }
    if info_dir.exists() {
        std::fs::remove_dir_all(&info_dir)
            .with_context(|| format!("Failed to empty info directory: {:?}", info_dir))?;
        std::fs::create_dir_all(&info_dir)
            .with_context(|| format!("Failed to recreate info directory: {:?}", info_dir))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};

    #[test]
    fn test_empty_trash_directory() {
        let temp_dir = std::env::temp_dir().join("nibble_test_trash_empty");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let files_dir = temp_dir.join("files");
        let info_dir = temp_dir.join("info");
        fs::create_dir_all(&files_dir).unwrap();
        fs::create_dir_all(&info_dir).unwrap();

        // Put some files inside files and info
        let dummy_file = files_dir.join("deleted_item.txt");
        let dummy_info = info_dir.join("deleted_item.trashinfo");
        File::create(&dummy_file).unwrap();
        File::create(&dummy_info).unwrap();

        assert!(dummy_file.exists());
        assert!(dummy_info.exists());

        empty_trash_directory(&temp_dir).unwrap();

        assert!(files_dir.exists());
        assert!(info_dir.exists());
        assert!(!dummy_file.exists());
        assert!(!dummy_info.exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
