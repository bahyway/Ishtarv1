#![forbid(unsafe_code)]
//! ProcessingZone — manages the Processing/ and Moved_To/ staging folders (BeeMDM §12.3).
//!
//! Layout inside the shard directory:
//!   shard/
//!   ├── Processing/           ← ZIP entries being worked on by ETL stations
//!   │   └── {batch}_{ts}/     ← one sub-dir per active batch
//!   └── Moved_To/             ← completed batches (all stations passed)
//!       └── {batch}_{ts}/

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// One file entry to be written to the Processing staging directory.
pub struct StagedEntry {
    pub name: String,
    pub data: Vec<u8>,
}

/// Manages the Processing/ and Moved_To/ sub-directories inside a shard root.
pub struct ProcessingZone {
    processing_dir: PathBuf,
    moved_to_dir:   PathBuf,
}

impl ProcessingZone {
    /// Create (or open) the zone inside `shard_dir`.
    /// Creates `shard_dir/Processing/` and `shard_dir/Moved_To/` if they don't exist.
    pub fn new(shard_dir: &Path) -> io::Result<Self> {
        let processing = shard_dir.join("Processing");
        let moved_to   = shard_dir.join("Moved_To");
        fs::create_dir_all(&processing)?;
        fs::create_dir_all(&moved_to)?;
        Ok(ProcessingZone { processing_dir: processing, moved_to_dir: moved_to })
    }

    /// Stage a list of entries into `Processing/{batch_dir_name}/`.
    /// Returns the path to the new batch directory.
    pub fn stage(&self, batch_dir_name: &str, entries: &[StagedEntry]) -> io::Result<PathBuf> {
        let batch_dir = self.processing_dir.join(batch_dir_name);
        fs::create_dir_all(&batch_dir)?;
        for entry in entries {
            // Flatten paths — only use the filename, not subdirectories from the ZIP
            let file_name = Path::new(&entry.name)
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new(&entry.name));
            fs::write(batch_dir.join(file_name), &entry.data)?;
        }
        Ok(batch_dir)
    }

    /// Write a text file (e.g. `.schema` descriptor) directly into a staged batch dir.
    pub fn write_in_batch(&self, batch_dir_name: &str, file_name: &str, content: &str) -> io::Result<()> {
        let path = self.processing_dir.join(batch_dir_name).join(file_name);
        fs::write(path, content.as_bytes())
    }

    /// Move `Processing/{batch_dir_name}/` → `Moved_To/{batch_dir_name}/`.
    /// The batch directory must exist in Processing/.
    pub fn complete(&self, batch_dir_name: &str) -> io::Result<PathBuf> {
        let src  = self.processing_dir.join(batch_dir_name);
        let dest = self.moved_to_dir.join(batch_dir_name);
        fs::rename(&src, &dest)?;
        Ok(dest)
    }

    /// True if the given batch directory currently exists under Processing/.
    pub fn is_staged(&self, batch_dir_name: &str) -> bool {
        self.processing_dir.join(batch_dir_name).is_dir()
    }

    pub fn processing_dir(&self) -> &Path { &self.processing_dir }
    pub fn moved_to_dir(&self) -> &Path   { &self.moved_to_dir }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("pzone_{tag}"));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn creates_subdirs() {
        let shard = tmp("create");
        let zone  = ProcessingZone::new(&shard).unwrap();
        assert!(zone.processing_dir().exists());
        assert!(zone.moved_to_dir().exists());
        let _ = fs::remove_dir_all(&shard);
    }

    #[test]
    fn stage_and_complete() {
        let shard = tmp("stage");
        let zone  = ProcessingZone::new(&shard).unwrap();

        let entries = vec![
            StagedEntry { name: "data.csv".into(), data: b"id,name\n1,Ali\n".to_vec() },
        ];
        let batch_dir = zone.stage("batch_001", &entries).unwrap();
        assert!(batch_dir.join("data.csv").exists());
        assert!(zone.is_staged("batch_001"));

        let dest = zone.complete("batch_001").unwrap();
        assert!(dest.exists());
        assert!(!zone.is_staged("batch_001"));
        let _ = fs::remove_dir_all(&shard);
    }

    #[test]
    fn write_schema_descriptor() {
        let shard = tmp("schema");
        let zone  = ProcessingZone::new(&shard).unwrap();
        zone.stage("b", &[]).unwrap();
        zone.write_in_batch("b", "batch.schema", "table_name = tb_test\n").unwrap();
        let content = fs::read_to_string(
            zone.processing_dir().join("b").join("batch.schema")
        ).unwrap();
        assert!(content.contains("tb_test"));
        let _ = fs::remove_dir_all(&shard);
    }
}
