use crate::scanner::size::{metadata_disk_usage_bytes, metadata_inode_key};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct ArenaNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
}

/// Walks a directory to construct a parent-child indexed arena and aggregate folder sizes bottom-up.
pub fn build_disk_tree(root: &Path) -> (Vec<ArenaNode>, Vec<String>) {
    let mut arena = vec![ArenaNode {
        name: root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "/".to_string()),
        path: root.to_path_buf(),
        is_dir: true,
        size_bytes: 0,
        parent: None,
        children: Vec::new(),
    }];

    let mut path_to_idx = HashMap::new();
    path_to_idx.insert(root.to_path_buf(), 0);

    let mut warnings = Vec::new();
    let mut seen_hardlinks = HashSet::new();

    let walker = WalkDir::new(root).min_depth(1);
    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                warnings.push(err.to_string());
                continue;
            }
        };

        let path = entry.path().to_path_buf();
        let parent_path = match path.parent() {
            Some(p) => p.to_path_buf(),
            None => continue,
        };

        let parent_idx = match path_to_idx.get(&parent_path) {
            Some(&idx) => idx,
            None => {
                // Fallback to root if the parent wasn't indexed due to some walking skip
                0
            }
        };

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let is_dir = entry.file_type().is_dir();

        let size_bytes = match std::fs::symlink_metadata(&path) {
            Ok(meta) => {
                if let Some(key) = metadata_inode_key(&meta) {
                    if !seen_hardlinks.insert(key) {
                        0
                    } else {
                        metadata_disk_usage_bytes(&meta)
                    }
                } else {
                    metadata_disk_usage_bytes(&meta)
                }
            }
            Err(err) => {
                warnings.push(format!(
                    "Failed to read metadata for {}: {}",
                    path.display(),
                    err
                ));
                0
            }
        };

        let idx = arena.len();
        arena.push(ArenaNode {
            name,
            path: path.clone(),
            is_dir,
            size_bytes,
            parent: Some(parent_idx),
            children: Vec::new(),
        });

        arena[parent_idx].children.push(idx);
        path_to_idx.insert(path, idx);
    }

    // Propagate sizes from leaves upwards (iterating in reverse order)
    for idx in (1..arena.len()).rev() {
        let size = arena[idx].size_bytes;
        if let Some(parent_idx) = arena[idx].parent {
            arena[parent_idx].size_bytes += size;
        }
    }

    (arena, warnings)
}

/// Subtracts the deleted size from all ancestors, and removes the node from its parent's child list.
pub fn delete_node_from_tree(arena: &mut [ArenaNode], node_idx: usize) {
    let size_to_remove = arena[node_idx].size_bytes;
    let mut current_parent = arena[node_idx].parent;

    while let Some(parent_idx) = current_parent {
        arena[parent_idx].size_bytes = arena[parent_idx].size_bytes.saturating_sub(size_to_remove);
        current_parent = arena[parent_idx].parent;
    }

    if let Some(parent_idx) = arena[node_idx].parent {
        arena[parent_idx].children.retain(|&idx| idx != node_idx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;

    #[test]
    fn test_build_disk_tree_and_delete() {
        let temp_dir = std::env::temp_dir().join("nibs_test_analyze_tree");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let sub_dir = temp_dir.join("subdir");
        fs::create_dir_all(&sub_dir).unwrap();

        let file1 = temp_dir.join("file1.txt");
        let mut f1 = File::create(&file1).unwrap();
        f1.write_all(b"12345").unwrap(); // 5 bytes

        let file2 = sub_dir.join("file2.txt");
        let mut f2 = File::create(&file2).unwrap();
        f2.write_all(b"1234567890").unwrap(); // 10 bytes

        let (mut arena, warnings) = build_disk_tree(&temp_dir);
        let file1_size = metadata_disk_usage_bytes(&std::fs::symlink_metadata(&file1).unwrap());
        let file2_size = metadata_disk_usage_bytes(&std::fs::symlink_metadata(&file2).unwrap());
        let subdir_own_size =
            metadata_disk_usage_bytes(&std::fs::symlink_metadata(&sub_dir).unwrap());

        assert!(warnings.is_empty());
        assert_eq!(
            arena[0].size_bytes,
            file1_size + file2_size + subdir_own_size
        );

        // Find file2 in the arena
        let file2_idx = arena
            .iter()
            .position(|node| node.name == "file2.txt")
            .unwrap();
        assert_eq!(arena[file2_idx].size_bytes, file2_size);

        // Find subdir in the arena
        let subdir_idx = arena.iter().position(|node| node.name == "subdir").unwrap();
        assert_eq!(arena[subdir_idx].size_bytes, file2_size + subdir_own_size);

        // Delete file2 from the tree
        delete_node_from_tree(&mut arena, file2_idx);

        assert_eq!(arena[subdir_idx].size_bytes, subdir_own_size);
        assert_eq!(arena[0].size_bytes, file1_size + subdir_own_size);

        // Subdir's children should no longer contain file2_idx
        assert!(!arena[subdir_idx].children.contains(&file2_idx));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
