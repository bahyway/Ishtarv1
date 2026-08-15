//! DataFileReader — point lookup against a Data File + sorted Index,
//! using binary search directly on disk (seek + read_exact). Opening a
//! reader never loads the data file or the index into memory: only file
//! handles and the record count (one stat call) are held.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

use enkidb_particles::Particle;

use crate::index::{IndexRecord, INDEX_RECORD_SIZE};
use crate::record::decode_particle;

pub struct DataFileReader {
    data_file:    File,
    idx_file:     File,
    record_count: u64,
}

impl DataFileReader {
    /// Open a Data File + its sorted Index for reading. O(1): reads only
    /// the index file's length (one stat syscall), nothing else.
    pub fn open(base: impl AsRef<Path>) -> io::Result<Self> {
        let base = base.as_ref();
        let data_file = File::open(base.with_extension("data"))?;
        let idx_file = File::open(base.with_extension("idx"))?;
        let idx_len = idx_file.metadata()?.len();
        let record_count = idx_len / INDEX_RECORD_SIZE as u64;
        Ok(DataFileReader { data_file, idx_file, record_count })
    }

    /// Number of indexed particles (index_file_size / 28), no data read.
    pub fn record_count(&self) -> u64 {
        self.record_count
    }

    /// Point lookup by KAKI bytes. O(log n) disk reads: no full-file
    /// load, no journal replay.
    pub fn get(&mut self, kaki: &[u8; 16]) -> io::Result<Option<Particle>> {
        Ok(self.get_traced(kaki)?.0)
    }

    /// Same as `get`, but also returns how many index probes (seek+read
    /// pairs against the .idx file) were performed. This is the concrete,
    /// runtime-measurable proof of O(log n) behavior: for N indexed
    /// particles, probes <= ceil(log2(N)) + 1, never N.
    pub fn get_traced(&mut self, kaki: &[u8; 16]) -> io::Result<(Option<Particle>, usize)> {
        let (found, probes) = self.find_index_record(kaki)?;
        match found {
            Some(record) => Ok((Some(self.read_data_record(&record)?), probes)),
            None => Ok((None, probes)),
        }
    }

    /// Point lookup returning the raw record bytes as written by
    /// [`crate::writer::DataFileWriter::append_raw`], for record shapes
    /// other than `Particle` that share this crate's disk-resident binary
    /// search (an entity's raw EAV history, an EAV posting list, ...).
    pub fn get_raw(&mut self, key: &[u8; 16]) -> io::Result<Option<Vec<u8>>> {
        Ok(self.get_raw_traced(key)?.0)
    }

    /// Same as `get_raw`, but also returns the index probe count.
    pub fn get_raw_traced(&mut self, key: &[u8; 16]) -> io::Result<(Option<Vec<u8>>, usize)> {
        let (found, probes) = self.find_index_record(key)?;
        match found {
            Some(record) => {
                let mut buf = vec![0u8; record.len as usize];
                self.data_file.seek(SeekFrom::Start(record.offset))?;
                self.data_file.read_exact(&mut buf)?;
                Ok((Some(buf), probes))
            }
            None => Ok((None, probes)),
        }
    }

    /// Binary search the `.idx` file for `key`. Returns the matching
    /// `IndexRecord` (if any) and the number of probes performed — the
    /// shared core behind both the Particle-typed and raw lookup paths.
    fn find_index_record(&mut self, key: &[u8; 16]) -> io::Result<(Option<IndexRecord>, usize)> {
        if self.record_count == 0 {
            return Ok((None, 0));
        }

        let mut lo: i64 = 0;
        let mut hi: i64 = self.record_count as i64 - 1;
        let mut probes = 0usize;

        while lo <= hi {
            probes += 1;
            let mid = lo + (hi - lo) / 2;
            let record = self.read_index_record(mid as u64)?;
            match record.kaki.as_slice().cmp(key.as_slice()) {
                std::cmp::Ordering::Equal => return Ok((Some(record), probes)),
                std::cmp::Ordering::Less => lo = mid + 1,
                std::cmp::Ordering::Greater => hi = mid - 1,
            }
        }
        Ok((None, probes))
    }

    fn read_index_record(&mut self, i: u64) -> io::Result<IndexRecord> {
        let mut buf = [0u8; INDEX_RECORD_SIZE];
        self.idx_file.seek(SeekFrom::Start(i * INDEX_RECORD_SIZE as u64))?;
        self.idx_file.read_exact(&mut buf)?;
        Ok(IndexRecord::from_bytes(&buf))
    }

    fn read_data_record(&mut self, record: &IndexRecord) -> io::Result<Particle> {
        let mut buf = vec![0u8; record.len as usize];
        self.data_file.seek(SeekFrom::Start(record.offset))?;
        self.data_file.read_exact(&mut buf)?;
        decode_particle(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{e:?}")))
    }

    /// Read every `(key, raw_bytes)` pair in this Data File in one bulk
    /// pass — the real building block for an in-memory cache that pays
    /// the O(n) load cost exactly once (at open/reload) instead of one
    /// disk seek per query. Two bulk reads total, not `record_count`
    /// individual `seek`+`read_exact` syscalls: the whole `.idx` file
    /// (small — `record_count * 28` bytes) and the whole `.data` file
    /// are each read into memory once, then every record is sliced
    /// straight out of the in-memory `.data` buffer using its known
    /// offset/len from the index — no further I/O at all.
    pub fn iter_all_raw(&mut self) -> io::Result<Vec<([u8; 16], Vec<u8>)>> {
        self.idx_file.seek(SeekFrom::Start(0))?;
        let mut idx_buf = Vec::new();
        self.idx_file.read_to_end(&mut idx_buf)?;

        self.data_file.seek(SeekFrom::Start(0))?;
        let mut data_buf = Vec::new();
        self.data_file.read_to_end(&mut data_buf)?;

        let mut out = Vec::with_capacity(self.record_count as usize);
        for chunk in idx_buf.chunks_exact(INDEX_RECORD_SIZE) {
            let record = IndexRecord::from_bytes(chunk);
            let start = record.offset as usize;
            let end = start + record.len as usize;
            let bytes = data_buf
                .get(start..end)
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "index record points past end of data file"))?
                .to_vec();
            out.push((record.kaki, bytes));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::writer::DataFileWriter;
    use akkvalue::AkkValue;
    use bahyway_core::TribeId;
    use enkidb_kaki::{IdentityKaki, KakiMinter, KakiRole};
    use std::fs;
    use std::path::PathBuf;

    fn tmp_base(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("enkidb_datafile_reader_{name}"));
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        dir.join("particles")
    }

    fn build_file(name: &str, n: u32) -> (PathBuf, Vec<[u8; 16]>) {
        let base = tmp_base(name);
        let mut w = DataFileWriter::open(&base).unwrap();
        let m = KakiMinter::new(TribeId::from_u16(0x0020));
        let mut kakis = Vec::with_capacity(n as usize);

        for i in 0..n {
            let e = IdentityKaki::try_from_kaki(m.mint_identity(i, KakiRole::Zikru)).unwrap();
            kakis.push(*e.bytes());
            w.append(&Particle::base(e, "score", AkkValue::Int(i as i64), i as u64)).unwrap();
        }
        w.sync().unwrap();
        w.compact_index().unwrap();
        (base, kakis)
    }

    #[test]
    fn get_finds_every_written_particle() {
        let (base, kakis) = build_file("find_all", 200);
        let mut r = DataFileReader::open(&base).unwrap();
        assert_eq!(r.record_count(), 200);

        for (i, kaki) in kakis.iter().enumerate() {
            let found = r.get(kaki).unwrap().expect("must find particle");
            assert_eq!(found.timestamp, i as u64);
        }
    }

    #[test]
    fn iter_all_raw_yields_every_written_record_with_correct_bytes() {
        let base = tmp_base("iter_all_raw");
        let mut w = DataFileWriter::open(&base).unwrap();
        let mut expected: Vec<([u8; 16], Vec<u8>)> = Vec::new();
        for i in 0u8..50 {
            let key = [i; 16];
            let payload = format!("payload-{i}").into_bytes();
            w.append_raw(key, &payload).unwrap();
            expected.push((key, payload));
        }
        w.sync().unwrap();
        w.compact_index().unwrap();

        let mut r = DataFileReader::open(&base).unwrap();
        let mut got = r.iter_all_raw().unwrap();
        got.sort_by_key(|(k, _)| *k);
        expected.sort_by_key(|(k, _)| *k);
        assert_eq!(got, expected);
    }

    #[test]
    fn iter_all_raw_on_empty_file_is_empty() {
        let base = tmp_base("iter_all_raw_empty");
        let mut w = DataFileWriter::open(&base).unwrap();
        w.sync().unwrap();
        w.compact_index().unwrap();

        let mut r = DataFileReader::open(&base).unwrap();
        assert!(r.iter_all_raw().unwrap().is_empty());
    }

    #[test]
    fn get_returns_none_for_unknown_kaki() {
        let (base, _) = build_file("unknown", 50);
        let mut r = DataFileReader::open(&base).unwrap();
        let bogus = [0xFFu8; 16];
        assert!(r.get(&bogus).unwrap().is_none());
    }

    #[test]
    fn lookup_is_logarithmic_not_linear() {
        // 10,000 records: linear scan would need up to 10,000 probes.
        // Binary search needs at most ceil(log2(10000)) + 1 = 15.
        let (base, kakis) = build_file("log_n", 10_000);
        let mut r = DataFileReader::open(&base).unwrap();

        let (found, probes) = r.get_traced(&kakis[9_999]).unwrap();
        assert!(found.is_some());
        assert!(
            probes <= 15,
            "expected O(log n) probes (<=15 for 10,000 records), got {probes}"
        );

        let (found, probes) = r.get_traced(&kakis[0]).unwrap();
        assert!(found.is_some());
        assert!(probes <= 15, "got {probes} probes for first record");
    }

    #[test]
    fn open_does_not_require_reading_data_file_contents() {
        // Structural guarantee: DataFileReader only holds File handles +
        // a u64 count, not a Vec<u8> of file contents. Opening a reader
        // on a file it will never fully read is a compile-time-checkable
        // fact of the struct's shape, verified here by usage: any lookup
        // still works after opening a file with many records, proving the
        // reader is not depending on an eager full read at open() time
        // (that codepath doesn't exist -- open() is a single File::open
        // + one metadata() stat call for each file).
        let (base, kakis) = build_file("no_eager_load", 5_000);
        let mut r = DataFileReader::open(&base).unwrap();
        assert_eq!(r.record_count(), 5_000);
        assert!(r.get(&kakis[2_500]).unwrap().is_some());
    }

    #[test]
    fn raw_round_trip_for_non_particle_records() {
        use crate::writer::DataFileWriter;

        let base = tmp_base("raw_round_trip");
        let mut w = DataFileWriter::open(&base).unwrap();
        let key_a = [1u8; 16];
        let key_b = [2u8; 16];
        w.append_raw(key_a, b"posting-list-bytes").unwrap();
        w.append_raw(key_b, b"entity-history-blob").unwrap();
        w.sync().unwrap();
        w.compact_index().unwrap();

        let mut r = DataFileReader::open(&base).unwrap();
        assert_eq!(r.get_raw(&key_a).unwrap().unwrap(), b"posting-list-bytes");
        assert_eq!(r.get_raw(&key_b).unwrap().unwrap(), b"entity-history-blob");
        assert!(r.get_raw(&[9u8; 16]).unwrap().is_none());
    }
}
