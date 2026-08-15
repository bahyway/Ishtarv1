//! DataFileWriter — appends particle records to a Data File and stages
//! their index entries. Staging is unsorted (cheap, O(1) per append);
//! `compact_index` merges staging into the sorted `.idx` the Reader
//! binary-searches, matching the real database bulk-load pattern
//! (append fast, sort in a separate compaction step, atomic swap).

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

use enkidb_particles::Particle;

use crate::index::{IndexRecord, INDEX_RECORD_SIZE};
use crate::record::encode_particle;

/// Buffer size for the data/staging `BufWriter`s. Unbuffered per-record
/// `write_all` calls cost two syscalls per append; at index-build scale
/// (materializing millions to billions of records) that overhead alone
/// dwarfs the actual work. 256 KiB batches syscalls without holding much
/// memory.
const WRITE_BUF_SIZE: usize = 256 * 1024;

pub struct DataFileWriter {
    data_file:    BufWriter<File>,
    staging_file: BufWriter<File>,
    data_path:    PathBuf,
    staging_path: PathBuf,
    idx_path:     PathBuf,
    next_offset:  u64,
}

impl DataFileWriter {
    /// Open (or create) a Data File + staging index for append.
    ///
    /// `base` determines the three sibling files: `{base}.data`,
    /// `{base}.idx.staging`, `{base}.idx`.
    pub fn open(base: impl AsRef<Path>) -> io::Result<Self> {
        let base = base.as_ref();
        let data_path = base.with_extension("data");
        let staging_path = base.with_extension("idx.staging");
        let idx_path = base.with_extension("idx");

        let data_file = OpenOptions::new().create(true).append(true).read(true).open(&data_path)?;
        let next_offset = data_file.metadata()?.len();
        let staging_file = OpenOptions::new().create(true).append(true).open(&staging_path)?;

        Ok(DataFileWriter {
            data_file: BufWriter::with_capacity(WRITE_BUF_SIZE, data_file),
            staging_file: BufWriter::with_capacity(WRITE_BUF_SIZE, staging_file),
            data_path,
            staging_path,
            idx_path,
            next_offset,
        })
    }

    /// Append one particle. Returns the offset it was written at.
    pub fn append(&mut self, particle: &Particle) -> io::Result<u64> {
        self.append_raw(*particle.entity.bytes(), &encode_particle(particle))
    }

    /// Append an arbitrary keyed record. Same append-only-data +
    /// staged-index mechanics as [`Self::append`], but the caller supplies
    /// both the 16-byte key and the record bytes directly instead of going
    /// through the Particle codec — lets other record shapes (an entity's
    /// full raw EAV history, an EAV posting list, ...) reuse this crate's
    /// on-disk binary-search machinery without inventing their own I/O.
    pub fn append_raw(&mut self, key: [u8; 16], bytes: &[u8]) -> io::Result<u64> {
        let offset = self.next_offset;
        let len = bytes.len() as u32;

        self.data_file.write_all(bytes)?;
        self.next_offset += bytes.len() as u64;

        let record = IndexRecord { kaki: key, offset, len };
        self.staging_file.write_all(&record.to_bytes())?;

        Ok(offset)
    }

    /// Flush + fsync both files. Call before `compact_index` for
    /// crash-safety (durability to physical disk, not just to the OS).
    pub fn sync(&mut self) -> io::Result<()> {
        self.data_file.flush()?;
        self.data_file.get_ref().sync_all()?;
        self.staging_file.flush()?;
        self.staging_file.get_ref().sync_all()
    }

    /// Merge the unsorted staging index into the sorted `.idx` file that
    /// `DataFileReader` binary-searches. Existing sorted entries are
    /// merged in (not discarded), then the staging file is truncated.
    /// Atomic: writes to a `.idx.new` file and renames over `.idx`.
    pub fn compact_index(&mut self) -> io::Result<usize> {
        // read_all_records reads `staging_path` through an independent file
        // handle -- must flush the buffered writer first or recently
        // appended-but-not-yet-flushed records would be silently dropped
        // from this compaction.
        self.staging_file.flush()?;
        let mut records = read_all_records(&self.staging_path)?;
        let existing = read_all_records(&self.idx_path).unwrap_or_default();
        records.extend(existing);
        records.sort_by(|a, b| a.kaki.cmp(&b.kaki));

        let new_path = self.idx_path.with_extension("idx.new");
        {
            let mut f = BufWriter::with_capacity(WRITE_BUF_SIZE, File::create(&new_path)?);
            for r in &records {
                f.write_all(&r.to_bytes())?;
            }
            f.flush()?;
            f.get_ref().sync_all()?;
        }
        fs::rename(&new_path, &self.idx_path)?;

        // Truncate staging now that it's merged in.
        let staging_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.staging_path)?;
        drop(staging_file);
        let staging_file = OpenOptions::new().create(true).append(true).open(&self.staging_path)?;
        self.staging_file = BufWriter::with_capacity(WRITE_BUF_SIZE, staging_file);

        Ok(records.len())
    }

    pub fn data_path(&self) -> &Path {
        &self.data_path
    }
}

fn read_all_records(path: &Path) -> io::Result<Vec<IndexRecord>> {
    let bytes = fs::read(path)?;
    let count = bytes.len() / INDEX_RECORD_SIZE;
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let start = i * INDEX_RECORD_SIZE;
        out.push(IndexRecord::from_bytes(&bytes[start..start + INDEX_RECORD_SIZE]));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use akkvalue::AkkValue;
    use bahyway_core::TribeId;
    use enkidb_kaki::{IdentityKaki, KakiMinter, KakiRole};

    fn tmp_base(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("enkidb_datafile_writer_{name}"));
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        dir.join("particles")
    }

    #[test]
    fn append_advances_offset_and_writes_bytes() {
        let base = tmp_base("append");
        let mut w = DataFileWriter::open(&base).unwrap();
        let m = KakiMinter::new(TribeId::from_u16(0x0010));
        let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let p = Particle::base(e, "a", AkkValue::Int(1), 0);

        let off1 = w.append(&p).unwrap();
        let off2 = w.append(&p).unwrap();
        assert_eq!(off1, 0);
        assert!(off2 > off1);
    }

    #[test]
    fn compact_index_merges_staging_and_sorts() {
        let base = tmp_base("compact");
        let mut w = DataFileWriter::open(&base).unwrap();
        let m = KakiMinter::new(TribeId::from_u16(0x0011));

        for i in 0..20 {
            let e = IdentityKaki::try_from_kaki(m.mint_identity(i, KakiRole::Zikru)).unwrap();
            let p = Particle::base(e, "a", AkkValue::Int(i as i64), 0);
            w.append(&p).unwrap();
        }
        w.sync().unwrap();
        let n = w.compact_index().unwrap();
        assert_eq!(n, 20);

        let idx_path = base.with_extension("idx");
        let records = read_all_records(&idx_path).unwrap();
        assert_eq!(records.len(), 20);
        for pair in records.windows(2) {
            assert!(pair[0].kaki <= pair[1].kaki, "index must be sorted ascending by kaki bytes");
        }
    }

    #[test]
    fn second_compaction_after_more_appends_keeps_all_entries_sorted() {
        let base = tmp_base("compact_twice");
        let mut w = DataFileWriter::open(&base).unwrap();
        let m = KakiMinter::new(TribeId::from_u16(0x0012));

        for i in 0..10 {
            let e = IdentityKaki::try_from_kaki(m.mint_identity(i, KakiRole::Zikru)).unwrap();
            w.append(&Particle::base(e, "a", AkkValue::Int(i as i64), 0)).unwrap();
        }
        w.compact_index().unwrap();

        for i in 10..20 {
            let e = IdentityKaki::try_from_kaki(m.mint_identity(i, KakiRole::Zikru)).unwrap();
            w.append(&Particle::base(e, "a", AkkValue::Int(i as i64), 0)).unwrap();
        }
        let n = w.compact_index().unwrap();
        assert_eq!(n, 20);

        let idx_path = base.with_extension("idx");
        let records = read_all_records(&idx_path).unwrap();
        assert_eq!(records.len(), 20);
        for pair in records.windows(2) {
            assert!(pair[0].kaki <= pair[1].kaki);
        }
    }
}
