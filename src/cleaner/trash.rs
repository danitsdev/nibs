use anyhow::{Context, Result};
use std::path::Path;

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
