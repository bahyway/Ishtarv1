//! LandingZone — polls a shared directory for newly arrived files.
//!
//! Pure std: no inotify/kqueue; uses `read_dir` polling.
//! Call `poll()` repeatedly; each call returns files not yet returned.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// File categories understood by the ETL pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LandingFileKind {
    Zip, // .zip — sovereign archive for bulk records
    Way, // .way — BahyWay security declaration
    Csv, // .csv — comma-separated records
    Tsv, // .tsv — tab-separated records
    Other,
}

impl LandingFileKind {
    pub fn detect(name: &str) -> Self {
        let lower = name.to_lowercase();
        if lower.ends_with(".zip") {
            LandingFileKind::Zip
        } else if lower.ends_with(".way") {
            LandingFileKind::Way
        } else if lower.ends_with(".csv") {
            LandingFileKind::Csv
        } else if lower.ends_with(".tsv") {
            LandingFileKind::Tsv
        } else {
            LandingFileKind::Other
        }
    }
}

/// A file found in the landing zone.
#[derive(Debug, Clone)]
pub struct LandingFile {
    pub path: PathBuf,
    pub kind: LandingFileKind,
    pub size: u64,
}

/// Monitors a directory for newly arrived files.
///
/// Tracks filenames that have already been returned by `poll()` so they
/// are not reported a second time.  State is in-memory only; on restart
/// every existing file is treated as new.
pub struct LandingZone {
    path: PathBuf,
    seen: BTreeSet<String>,
}

impl LandingZone {
    /// Open (or create) the landing-zone directory.
    pub fn new(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        fs::create_dir_all(&path)?;
        Ok(LandingZone {
            path,
            seen: BTreeSet::new(),
        })
    }

    /// Returns all files in the directory not yet returned by a prior `poll()`.
    pub fn poll(&mut self) -> Vec<LandingFile> {
        let mut new_files = Vec::new();
        let entries = match fs::read_dir(&self.path) {
            Ok(e) => e,
            Err(_) => return new_files,
        };
        let mut candidates: Vec<LandingFile> = entries
            .flatten()
            .filter(|e| e.path().is_file())
            .filter_map(|e| {
                let p = e.path();
                let name = p.file_name()?.to_string_lossy().to_string();
                let size = e.metadata().map(|m| m.len()).unwrap_or(0);
                let kind = LandingFileKind::detect(&name);
                Some(LandingFile {
                    path: p,
                    kind,
                    size,
                })
            })
            .collect();

        // Sort by filename for deterministic ordering
        candidates.sort_by(|a, b| a.path.file_name().cmp(&b.path.file_name()));

        for f in candidates {
            let name = f
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            if !self.seen.contains(&name) {
                self.seen.insert(name);
                new_files.push(f);
            }
        }
        new_files
    }

    /// Mark a file as already seen (so it will not be returned by the next `poll()`).
    pub fn mark_seen(&mut self, name: &str) {
        self.seen.insert(name.to_string());
    }

    /// Move a landed file out of the watched root into `{path}/Archived/`,
    /// preserving it (never deleted) but removing it from `poll()`'s search
    /// space. `seen` is in-memory only — without this, a process restart
    /// treats every file still sitting in the watched root as new, and
    /// reprocesses already-completed batches, minting duplicate particles.
    /// Callers should archive a file before or immediately after dispatching
    /// it, not defer it until the whole batch has finished processing.
    pub fn archive(&self, lf: &LandingFile) -> std::io::Result<PathBuf> {
        let archived_dir = self.path.join("Archived");
        fs::create_dir_all(&archived_dir)?;
        let file_name = lf
            .path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("source"));
        let dest = archived_dir.join(file_name);
        fs::rename(&lf.path, &dest)?;
        Ok(dest)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn seen_count(&self) -> usize {
        self.seen.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("enkidw_lz_{}", tag));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn touch(dir: &Path, name: &str) {
        fs::write(dir.join(name), b"test").unwrap();
    }

    #[test]
    fn poll_returns_new_files() {
        let dir = tmp_dir("poll");
        let mut lz = LandingZone::new(&dir).unwrap();
        touch(&dir, "a.zip");
        touch(&dir, "b.way");
        let first = lz.poll();
        assert_eq!(first.len(), 2);
        let second = lz.poll();
        assert!(second.is_empty(), "should not re-return seen files");
        touch(&dir, "c.csv");
        let third = lz.poll();
        assert_eq!(third.len(), 1);
        assert_eq!(third[0].kind, LandingFileKind::Csv);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn archive_moves_file_out_of_watched_root() {
        let dir = tmp_dir("archive");
        let mut lz = LandingZone::new(&dir).unwrap();
        touch(&dir, "a.zip");
        let found = lz.poll();
        assert_eq!(found.len(), 1);

        let dest = lz.archive(&found[0]).unwrap();
        assert!(dest.exists());
        assert!(dest.starts_with(dir.join("Archived")));
        assert!(
            !dir.join("a.zip").exists(),
            "original must be moved, not copied"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn archived_file_is_not_rediscovered_after_restart() {
        // Simulates a process restart: a fresh LandingZone (no in-memory
        // `seen` state) pointed at the same directory must not rediscover a
        // file that was archived by a prior instance — this is what prevents
        // duplicate reprocessing of an already-completed batch.
        let dir = tmp_dir("restart");
        {
            let mut lz = LandingZone::new(&dir).unwrap();
            touch(&dir, "a.zip");
            let found = lz.poll();
            lz.archive(&found[0]).unwrap();
        }
        let mut lz2 = LandingZone::new(&dir).unwrap();
        let found2 = lz2.poll();
        assert!(
            found2.is_empty(),
            "archived file must not resurface as a new file after restart"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn kind_detection() {
        assert_eq!(LandingFileKind::detect("data.ZIP"), LandingFileKind::Zip);
        assert_eq!(LandingFileKind::detect("najaf.way"), LandingFileKind::Way);
        assert_eq!(LandingFileKind::detect("records.tsv"), LandingFileKind::Tsv);
        assert_eq!(
            LandingFileKind::detect("unknown.bin"),
            LandingFileKind::Other
        );
    }
}
