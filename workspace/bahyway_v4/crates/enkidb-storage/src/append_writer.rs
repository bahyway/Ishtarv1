//! AppendWriter — sequential append-only file writer with atomic commit markers.
//!
//! Every Event-Kaki write is followed by an atomic commit marker (last byte)
//! so crash recovery can detect mid-write interruptions (§11.5).
//!
//! The commit marker byte value is 0xCA (COMMITTED/ACCEPTED).

use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::Path;

use bahyway_core::Result;
use crate::durability::FsyncPolicy;

pub const COMMIT_MARKER: u8 = 0xCA;

/// Append-only sequential writer with fsync-per-commit or batched durability.
pub struct AppendWriter {
    inner:  BufWriter<File>,
    policy: FsyncPolicy,
    path:   std::path::PathBuf,
}

impl AppendWriter {
    /// Open or create a file for append-only writing.
    pub fn open(path: impl AsRef<Path>, policy: FsyncPolicy) -> io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path.as_ref())?;
        Ok(AppendWriter {
            inner:  BufWriter::new(file),
            policy,
            path:   path.as_ref().to_path_buf(),
        })
    }

    /// Append a byte slice + the atomic commit marker, then apply the fsync policy.
    pub fn append_committed(&mut self, data: &[u8]) -> Result<()> {
        self.inner.write_all(data)?;
        self.inner.write_all(&[COMMIT_MARKER])?;
        self.inner.flush()?;
        self.apply_fsync()?;
        Ok(())
    }

    fn apply_fsync(&mut self) -> io::Result<()> {
        match &self.policy {
            FsyncPolicy::PerCommit => {
                self.inner.get_ref().sync_all()?;
            }
            FsyncPolicy::Batched { .. } | FsyncPolicy::Never => {
                // Caller is responsible for periodic fsync under Batched policy.
            }
        }
        Ok(())
    }

    /// Force fsync right now regardless of policy — e.g., at checkpoint time.
    pub fn sync_now(&self) -> io::Result<()> {
        self.inner.get_ref().sync_all()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use crate::durability::FsyncPolicy;

    #[test]
    fn append_and_commit_marker() {
        let dir  = std::env::temp_dir();
        let path = dir.join("enkidb_append_test.bin");
        let _cleanup = Cleanup(path.clone());

        let mut w = AppendWriter::open(&path, FsyncPolicy::Never).unwrap();
        w.append_committed(b"hello").unwrap();
        w.append_committed(b"world").unwrap();
        drop(w);

        let raw = fs::read(&path).unwrap();
        // "hello" + COMMIT_MARKER + "world" + COMMIT_MARKER
        assert_eq!(raw, b"hello\xCAworld\xCA");
    }

    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) { let _ = fs::remove_file(&self.0); }
    }
}
