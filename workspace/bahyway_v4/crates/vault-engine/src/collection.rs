//! # Collection — Sovereign File Collection Builder
//!
//! Groups real `FileRecord` entries into sovereign collections based on
//! top-level parent folder. Each collection becomes a node in the
//! knowledge graph (dubsar-canvas Template 4).

use super::file_walker::FileRecord;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// A named group of files with shared semantics.
/// Maps to ONE node in the knowledge graph (Template 4 — NodeGraph).
#[derive(Debug, Clone)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub files: Vec<FileRecord>,
    pub total_bytes: u64,
    pub file_count: u64,
    pub zero_byte_cnt: u64,
    pub avg_age_days: f32,
    /// Dominant file extension (e.g. "pdf", "jpg")
    pub dominant_ext: String,
}

impl Collection {
    pub fn from_files(id: String, name: String, path: PathBuf, files: Vec<FileRecord>) -> Self {
        let file_count = files.len() as u64;
        let total_bytes = files.iter().map(|f| f.size).sum();
        let zero_byte_cnt = files.iter().filter(|f| f.is_zero).count() as u64;
        let avg_age_days = if file_count > 0 {
            files.iter().map(|f| f.age_days()).sum::<f32>() / file_count as f32
        } else {
            0.0
        };

        let mut ext_counts: BTreeMap<String, u64> = BTreeMap::new();
        for f in &files {
            *ext_counts.entry(f.extension.clone()).or_insert(0) += 1;
        }
        let dominant_ext = ext_counts
            .into_iter()
            .max_by_key(|(_, c)| *c)
            .map(|(e, _)| e)
            .unwrap_or_else(|| "(mixed)".to_string());

        Self {
            id,
            name,
            path,
            files,
            total_bytes,
            file_count,
            zero_byte_cnt,
            avg_age_days,
            dominant_ext,
        }
    }
}

/// A set of collections derived from a single scan root.
#[derive(Debug, Default, Clone)]
pub struct CollectionSet {
    pub collections: Vec<Collection>,
    pub root: PathBuf,
}

impl CollectionSet {
    pub fn total_files(&self) -> u64 {
        self.collections.iter().map(|c| c.file_count).sum()
    }
    pub fn total_bytes(&self) -> u64 {
        self.collections.iter().map(|c| c.total_bytes).sum()
    }
    pub fn len(&self) -> usize {
        self.collections.len()
    }
    pub fn is_empty(&self) -> bool {
        self.collections.is_empty()
    }
}

/// Build collections by grouping files by their immediate parent folder
/// under the scan root. Each top-level subfolder becomes one collection.
/// Files directly in root become the "(root)" collection.
pub fn build_collections(root: &std::path::Path, files: Vec<FileRecord>) -> CollectionSet {
    let mut groups: BTreeMap<PathBuf, Vec<FileRecord>> = BTreeMap::new();

    for file in files {
        let relative = file.path.strip_prefix(root).unwrap_or(&file.path);
        let top_level = relative
            .components()
            .next()
            .map(|c| std::path::PathBuf::from(c.as_os_str()))
            .unwrap_or_else(|| PathBuf::from("(root)"));

        let key = if file.path.parent() == Some(root) {
            PathBuf::from("(root)")
        } else {
            root.join(&top_level)
        };

        groups.entry(key).or_default().push(file);
    }

    let mut collections: Vec<Collection> = groups
        .into_iter()
        .enumerate()
        .map(|(i, (path, files))| {
            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("(root)")
                .to_string();
            let id = format!("col_{:04}", i);
            Collection::from_files(id, name, path, files)
        })
        .collect();

    collections.sort_by_key(|b| std::cmp::Reverse(b.file_count));

    CollectionSet {
        collections,
        root: root.to_path_buf(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_walker::walk_folder;

    fn make_temp_dir() -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "vault_col_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
        ));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn empty_root_produces_empty_collection_set() {
        let dir = make_temp_dir();
        let (files, _) = walk_folder(&dir);
        let set = build_collections(&dir, files);
        assert!(set.is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn files_in_subfolders_become_collections() {
        let dir = make_temp_dir();
        let sub = dir.join("contracts");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(sub.join("a.pdf"), b"pdf1").unwrap();
        std::fs::write(sub.join("b.pdf"), b"pdf2").unwrap();

        let (files, _) = walk_folder(&dir);
        let set = build_collections(&dir, files);
        assert_eq!(set.len(), 1);
        assert_eq!(set.collections[0].file_count, 2);
        assert_eq!(set.collections[0].dominant_ext, "pdf");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn root_files_become_root_collection() {
        let dir = make_temp_dir();
        std::fs::write(dir.join("readme.txt"), b"hello").unwrap();

        let (files, _) = walk_folder(&dir);
        let set = build_collections(&dir, files);
        assert_eq!(set.len(), 1);
        assert_eq!(set.collections[0].name, "(root)");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn collections_sorted_by_file_count_descending() {
        let dir = make_temp_dir();
        let big = dir.join("big");
        let small = dir.join("small");
        std::fs::create_dir(&big).unwrap();
        std::fs::create_dir(&small).unwrap();
        for i in 0..5 {
            std::fs::write(big.join(format!("f{}.txt", i)), b"x").unwrap();
        }
        std::fs::write(small.join("only.txt"), b"y").unwrap();

        let (files, _) = walk_folder(&dir);
        let set = build_collections(&dir, files);
        assert_eq!(set.len(), 2);
        assert!(
            set.collections[0].file_count >= set.collections[1].file_count,
            "collections must be sorted largest first"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn collection_total_bytes_sums_all_files() {
        let dir = make_temp_dir();
        let sub = dir.join("data");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(sub.join("a.bin"), b"hello").unwrap(); // 5 bytes
        std::fs::write(sub.join("b.bin"), b"world").unwrap(); // 5 bytes

        let (files, _) = walk_folder(&dir);
        let set = build_collections(&dir, files);
        assert_eq!(set.total_bytes(), 10);
        std::fs::remove_dir_all(&dir).ok();
    }
}
