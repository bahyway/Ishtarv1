//! column.rs -- fixed-width columnar data file: surrogate u32 -> row offset.
//!
//! On-disk layout: `AppendWriter::append_committed` writes each row as
//! `W` data bytes followed by one `enkidb_storage::COMMIT_MARKER` byte, so
//! every on-disk record is exactly `W + 1` bytes wide. Because rows are
//! appended strictly in ascending, gapless surrogate order (enforced by
//! `ColumnWriter`), surrogate `s`'s record always starts at byte offset
//! `s * (W + 1)` -- a pure arithmetic lookup, no index structure needed.

use std::path::Path;

use bahyway_core::BahywayError;
use enkidb_storage::{AppendWriter, FsyncPolicy, MmapReader};

#[derive(Debug)]
pub enum EshnunnaError {
    /// `append_row` was called with a surrogate other than the next
    /// expected one -- Eshnunna only ever appends, in surrogate order,
    /// never at arbitrary offsets (Rule: append-only, ADR-006).
    OutOfOrderSurrogate {
        expected: u32,
        got: u32,
    },
    Storage(BahywayError),
    Io(std::io::Error),
}

impl std::fmt::Display for EshnunnaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EshnunnaError::OutOfOrderSurrogate { expected, got } => write!(
                f,
                "Eshnunna is append-only in surrogate order: expected surrogate {expected}, got {got}"
            ),
            EshnunnaError::Storage(e) => write!(f, "storage error: {e}"),
            EshnunnaError::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl std::error::Error for EshnunnaError {}

impl From<BahywayError> for EshnunnaError {
    fn from(e: BahywayError) -> Self {
        EshnunnaError::Storage(e)
    }
}

impl From<std::io::Error> for EshnunnaError {
    fn from(e: std::io::Error) -> Self {
        EshnunnaError::Io(e)
    }
}

/// On-disk byte stride of one committed row: `W` data bytes + 1 commit marker.
const fn stride(w: usize) -> usize {
    w + 1
}

/// Append-only columnar writer. Rows must be appended in ascending,
/// gapless surrogate order -- matching `SurrogateMap::build`'s own
/// contract that `kakis[i]` is assigned surrogate `i`.
pub struct ColumnWriter<const W: usize> {
    writer: AppendWriter,
    next_surrogate: u32,
}

impl<const W: usize> ColumnWriter<W> {
    /// Create (or resume appending to) a column file at `path`.
    pub fn create(path: impl AsRef<Path>, policy: FsyncPolicy) -> Result<Self, EshnunnaError> {
        let writer = AppendWriter::open(path.as_ref(), policy)?;
        // Resume: an existing file's row count tells us the next surrogate.
        let next_surrogate = match std::fs::metadata(path.as_ref()) {
            Ok(meta) => (meta.len() as usize / stride(W)) as u32,
            Err(_) => 0,
        };
        Ok(Self {
            writer,
            next_surrogate,
        })
    }

    /// Append the value for `surrogate`. Must equal the next expected
    /// surrogate (0, 1, 2, ... in order) -- Eshnunna never writes out of
    /// order or overwrites, only ever appends the next row.
    pub fn append_row(&mut self, surrogate: u32, value: [u8; W]) -> Result<(), EshnunnaError> {
        if surrogate != self.next_surrogate {
            return Err(EshnunnaError::OutOfOrderSurrogate {
                expected: self.next_surrogate,
                got: surrogate,
            });
        }
        self.writer.append_committed(&value)?;
        self.next_surrogate += 1;
        Ok(())
    }

    /// Number of rows committed so far.
    pub fn row_count(&self) -> u32 {
        self.next_surrogate
    }
}

/// Read-only columnar reader: fetch a row's value by surrogate in O(1),
/// one mmap read at a computed offset.
pub struct ColumnReader<const W: usize> {
    mmap: MmapReader,
}

impl<const W: usize> ColumnReader<W> {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, EshnunnaError> {
        Ok(Self {
            mmap: MmapReader::open(path)?,
        })
    }

    /// Fetch the value for `surrogate` by direct offset. `None` if the
    /// surrogate is beyond the committed row count (never a partial read).
    pub fn get(&self, surrogate: u32) -> Option<[u8; W]> {
        let offset = surrogate as usize * stride(W);
        self.mmap.read_at::<W>(offset)
    }

    /// Number of complete rows stored in the file.
    pub fn row_count(&self) -> usize {
        self.mmap.len() / stride(W)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("eshnunna_test_{name}_{}.bin", std::process::id()))
    }

    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    #[test]
    fn write_then_read_back_by_surrogate() {
        let path = temp_path("roundtrip");
        let _cleanup = Cleanup(path.clone());

        let mut w = ColumnWriter::<8>::create(&path, FsyncPolicy::Never).unwrap();
        for i in 0u32..100 {
            let value = (i as u64 * 7).to_le_bytes();
            w.append_row(i, value).unwrap();
        }
        assert_eq!(w.row_count(), 100);
        drop(w);

        let r = ColumnReader::<8>::open(&path).unwrap();
        assert_eq!(r.row_count(), 100);
        for i in 0u32..100 {
            let got = r.get(i).expect("row must exist");
            assert_eq!(
                u64::from_le_bytes(got),
                i as u64 * 7,
                "surrogate {i} mismatch"
            );
        }
    }

    #[test]
    fn surrogate_beyond_row_count_returns_none() {
        let path = temp_path("beyond");
        let _cleanup = Cleanup(path.clone());

        let mut w = ColumnWriter::<4>::create(&path, FsyncPolicy::Never).unwrap();
        for i in 0u32..10 {
            w.append_row(i, i.to_le_bytes()).unwrap();
        }
        drop(w);

        let r = ColumnReader::<4>::open(&path).unwrap();
        assert!(r.get(10).is_none());
        assert!(r.get(9999).is_none());
    }

    #[test]
    fn out_of_order_surrogate_is_rejected() {
        let path = temp_path("outoforder");
        let _cleanup = Cleanup(path.clone());

        let mut w = ColumnWriter::<4>::create(&path, FsyncPolicy::Never).unwrap();
        w.append_row(0, 0u32.to_le_bytes()).unwrap();
        let err = w.append_row(5, 5u32.to_le_bytes()).unwrap_err();
        assert!(matches!(
            err,
            EshnunnaError::OutOfOrderSurrogate {
                expected: 1,
                got: 5
            }
        ));
    }

    #[test]
    fn writer_resumes_from_existing_file() {
        let path = temp_path("resume");
        let _cleanup = Cleanup(path.clone());

        {
            let mut w = ColumnWriter::<4>::create(&path, FsyncPolicy::Never).unwrap();
            for i in 0u32..5 {
                w.append_row(i, i.to_le_bytes()).unwrap();
            }
        }
        // Reopen: must resume at surrogate 5, not restart at 0.
        let mut w2 = ColumnWriter::<4>::create(&path, FsyncPolicy::Never).unwrap();
        assert_eq!(w2.row_count(), 5);
        w2.append_row(5, 5u32.to_le_bytes()).unwrap();
        drop(w2);

        let r = ColumnReader::<4>::open(&path).unwrap();
        assert_eq!(r.row_count(), 6);
        for i in 0u32..6 {
            assert_eq!(u32::from_le_bytes(r.get(i).unwrap()), i);
        }
    }

    #[test]
    fn empty_file_has_zero_rows() {
        let path = temp_path("empty");
        let _cleanup = Cleanup(path.clone());

        let w = ColumnWriter::<8>::create(&path, FsyncPolicy::Never).unwrap();
        assert_eq!(w.row_count(), 0);
        drop(w);

        let r = ColumnReader::<8>::open(&path).unwrap();
        assert_eq!(r.row_count(), 0);
        assert!(r.get(0).is_none());
    }
}
