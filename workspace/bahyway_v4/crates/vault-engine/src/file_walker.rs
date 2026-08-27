//! # file_walker — Sovereign File System Walker
//!
//! Pure-std recursive file system scanner. Produces a complete inventory
//! of files with their real sizes, modification times, and extensions.
//!
//! SOVEREIGN RULE: every byte of metadata comes from the real OS — no
//! invented or demo data in the walker. Zero external dependencies.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// A single file discovered during the scan.
/// Contains ONLY real properties read from the file system.
#[derive(Debug, Clone)]
pub struct FileRecord {
    pub path: PathBuf,
    pub size: u64,
    pub extension: String,
    pub modified: Option<SystemTime>,
    pub is_zero: bool,
    pub parent_dir: PathBuf,
}

impl FileRecord {
    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("(unnamed)")
            .to_string()
    }

    pub fn parent_name(&self) -> String {
        self.parent_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("(root)")
            .to_string()
    }

    /// Age in days since last modification
    pub fn age_days(&self) -> f32 {
        let Some(modified) = self.modified else {
            return 0.0;
        };
        let Ok(elapsed) = modified.elapsed() else {
            return 0.0;
        };
        elapsed.as_secs() as f32 / 86_400.0
    }
}

/// Aggregate stats from a scan pass.
#[derive(Debug, Clone, Default)]
pub struct ScanSummary {
    pub root: PathBuf,
    pub total_files: u64,
    pub total_bytes: u64,
    pub zero_byte_count: u64,
    pub extension_counts: std::collections::BTreeMap<String, u64>,
    pub deepest_path: u32,
    pub scan_duration_ms: u64,
}

/// Walk a folder and return `(records, summary)`.
/// Pure-std recursive scan — no external dependencies.
pub fn walk_folder(root: &Path) -> (Vec<FileRecord>, ScanSummary) {
    let start = std::time::Instant::now();
    let mut records = Vec::new();
    let mut summary = ScanSummary {
        root: root.to_path_buf(),
        ..Default::default()
    };

    walk_recursive(root, root, 0, 12, &mut records, &mut summary);

    summary.scan_duration_ms = start.elapsed().as_millis() as u64;
    (records, summary)
}

fn walk_recursive(
    dir: &Path,
    root: &Path,
    depth: u32,
    max_depth: u32,
    records: &mut Vec<FileRecord>,
    summary: &mut ScanSummary,
) {
    if depth > max_depth {
        return;
    }

    let Ok(read) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in read.filter_map(|e| e.ok()) {
        let path = entry.path();
        let Ok(metadata) = entry.metadata() else {
            continue;
        };

        if metadata.is_dir() {
            walk_recursive(&path, root, depth + 1, max_depth, records, summary);
            continue;
        }
        if !metadata.is_file() {
            continue;
        }

        let size = metadata.len();
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_else(|| "(none)".to_string());

        let parent_dir = path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| root.to_path_buf());

        let modified = metadata.modified().ok();
        let is_zero = size == 0;

        if is_zero {
            summary.zero_byte_count += 1;
        }
        summary.total_files += 1;
        summary.total_bytes += size;
        *summary
            .extension_counts
            .entry(extension.clone())
            .or_insert(0) += 1;
        if depth > summary.deepest_path {
            summary.deepest_path = depth;
        }

        records.push(FileRecord {
            path,
            size,
            extension,
            modified,
            is_zero,
            parent_dir,
        });
    }
}

/// Non-blocking scan via a background thread.
/// Returns a receiver; poll once in an async or update loop.
pub fn walk_folder_async(
    root: PathBuf,
) -> std::sync::mpsc::Receiver<(Vec<FileRecord>, ScanSummary)> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let result = walk_folder(&root);
        let _ = tx.send(result);
    });
    rx
}

#[cfg(test)]
mod tests {
    use super::*;
    fn make_temp_dir() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "vault_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
        ));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn walk_empty_dir_returns_no_files() {
        let dir = make_temp_dir();
        let (records, summary) = walk_folder(&dir);
        assert!(records.is_empty());
        assert_eq!(summary.total_files, 0);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn walk_finds_created_file() {
        let dir = make_temp_dir();
        let file_path = dir.join("test.txt");
        std::fs::write(&file_path, b"hello vault").unwrap();

        let (records, summary) = walk_folder(&dir);
        assert_eq!(records.len(), 1);
        assert_eq!(summary.total_files, 1);
        assert_eq!(summary.total_bytes, 11);
        assert_eq!(records[0].extension, "txt");
        assert!(!records[0].is_zero);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn walk_detects_zero_byte_file() {
        let dir = make_temp_dir();
        std::fs::write(dir.join("empty.txt"), b"").unwrap();

        let (records, summary) = walk_folder(&dir);
        assert_eq!(summary.zero_byte_count, 1);
        assert!(records[0].is_zero);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn walk_recurses_into_subdirectories() {
        let dir = make_temp_dir();
        let sub = dir.join("sub");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(sub.join("deep.rs"), b"fn main(){}").unwrap();

        let (records, summary) = walk_folder(&dir);
        assert_eq!(records.len(), 1);
        assert_eq!(summary.total_files, 1);
        assert!(summary.deepest_path >= 1);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn extension_counts_are_tracked() {
        let dir = make_temp_dir();
        std::fs::write(dir.join("a.rs"), b"rs").unwrap();
        std::fs::write(dir.join("b.rs"), b"rs").unwrap();
        std::fs::write(dir.join("c.txt"), b"txt").unwrap();

        let (_, summary) = walk_folder(&dir);
        assert_eq!(summary.extension_counts.get("rs").copied(), Some(2));
        assert_eq!(summary.extension_counts.get("txt").copied(), Some(1));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn file_record_age_days_is_non_negative() {
        let dir = make_temp_dir();
        std::fs::write(dir.join("x.bin"), b"data").unwrap();
        let (records, _) = walk_folder(&dir);
        assert!(records[0].age_days() >= 0.0);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn async_walker_returns_same_result_as_sync() {
        let dir = make_temp_dir();
        std::fs::write(dir.join("a.txt"), b"async test").unwrap();

        let rx = walk_folder_async(dir.clone());
        let (async_records, async_summary) = rx.recv().unwrap();
        let (sync_records, sync_summary) = walk_folder(&dir);

        assert_eq!(async_records.len(), sync_records.len());
        assert_eq!(async_summary.total_files, sync_summary.total_files);
        assert_eq!(async_summary.total_bytes, sync_summary.total_bytes);

        std::fs::remove_dir_all(&dir).ok();
    }
}
