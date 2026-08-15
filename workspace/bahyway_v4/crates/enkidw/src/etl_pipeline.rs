//! EtlPipeline — full ETL orchestrator for the EnkiDW landing zone (§12.2).
//!
//! Processing sequence per file:
//!   1. LandingZone::poll()   → new files
//!   2. AlertEngine           → emit DW alert for arrivals / failures
//!   3. ZipEngine             → extract if .zip
//!   4. KakiGenerator         → parse TSV/CSV rows → GeneratedEntry
//!   5. PersistedDb::commit() → disk-first CQRS write
//!   6. WayCompiler           → if .way, compile to .akk and log
//!
//! All processing is single-pass and crash-safe via PersistedDb.

use std::fs;
use std::path::Path;

use bahyway_core::TribeId;
use enkidb_kaki::KakiMinter;
use enkidb_persist::PersistedDb;
use enkidb_storage::FsyncPolicy;

use crate::landing_zone::{LandingFileKind, LandingZone};
use crate::zip_engine::{self, ZipError};
use crate::kaki_generator::{self, RawRecord};
use crate::way_file::WayFile;
use crate::way_compiler;

use compare_tribe_schema::{compare_versions, CompareVerdict, FieldMeta, FieldType, SchemaVersion};
use musaru_security::zip_scan as musaru_scan;

/// Severity level for a DW pipeline alert.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DwAlertSeverity { Info, Warning, Error }

/// An alert emitted by the ETL pipeline during processing.
#[derive(Debug, Clone)]
pub struct DwAlert {
    pub severity: DwAlertSeverity,
    pub message:  String,
}

impl DwAlert {
    fn info(msg: impl Into<String>)    -> Self { DwAlert { severity: DwAlertSeverity::Info,    message: msg.into() } }
    fn warning(msg: impl Into<String>) -> Self { DwAlert { severity: DwAlertSeverity::Warning, message: msg.into() } }
    fn error(msg: impl Into<String>)   -> Self { DwAlert { severity: DwAlertSeverity::Error,   message: msg.into() } }
}

/// Statistics accumulated during pipeline execution.
#[derive(Debug, Default, Clone)]
pub struct EtlStats {
    /// Files polled from the landing zone.
    pub files_seen:       usize,
    /// ZIP archives processed (entries extracted).
    pub zips_processed:   usize,
    /// `.way` files compiled.
    pub way_compiled:     usize,
    /// Data records successfully ingested as particles.
    pub records_ingested: usize,
    /// Records or entries that were skipped due to errors.
    pub records_skipped:  usize,
    /// Total DW alerts emitted (info + warning + error).
    pub alerts_emitted:   usize,
}

/// The ETL pipeline — drives the full LandingZone→PersistedDb flow.
pub struct EtlPipeline {
    landing:        LandingZone,
    pdb:            PersistedDb,
    minter:         KakiMinter,
    stats:          EtlStats,
    alerts:         Vec<DwAlert>,
    last_schema:    Option<SchemaVersion>,
    schema_version: u32,
}

impl EtlPipeline {
    /// Open the pipeline: create the landing zone directory and open PersistedDb.
    pub fn open(
        landing_path: &Path,
        data_dir:     &Path,
        tribe_id:     TribeId,
        policy:       FsyncPolicy,
    ) -> std::io::Result<Self> {
        let landing = LandingZone::new(landing_path)?;
        let pdb     = PersistedDb::open(data_dir, tribe_id, policy)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        Ok(EtlPipeline {
            landing,
            pdb,
            minter:         KakiMinter::new(tribe_id),
            stats:          EtlStats::default(),
            alerts:         Vec::new(),
            last_schema:    None,
            schema_version: 0,
        })
    }

    /// Poll the landing zone and process every new file.
    /// Returns a reference to the updated stats.
    pub fn run_once(&mut self) -> &EtlStats {
        let new_files = self.landing.poll();
        if new_files.is_empty() { return &self.stats; }

        self.emit(DwAlert::info(format!("{} new file(s) in landing zone", new_files.len())));
        self.stats.files_seen += new_files.len();

        for lf in new_files {
            let fname = lf.path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            match lf.kind {
                LandingFileKind::Zip => self.process_zip(&lf.path, &fname),
                LandingFileKind::Way => self.process_way(&lf.path, &fname),
                LandingFileKind::Tsv => self.process_records(&lf.path, &fname, false),
                LandingFileKind::Csv => self.process_records(&lf.path, &fname, true),
                LandingFileKind::Other => {
                    self.emit(DwAlert::warning(format!("{fname}: unrecognised file type — skipping")));
                    self.stats.records_skipped += 1;
                }
            }
        }

        &self.stats
    }

    // ── File processors ──────────────────────────────────────────────────────

    fn process_zip(&mut self, path: &Path, fname: &str) {
        let data = match fs::read(path) {
            Ok(d)  => d,
            Err(e) => {
                self.emit(DwAlert::error(format!("{fname}: read error — {e}")));
                self.stats.records_skipped += 1;
                return;
            }
        };

        // ── Musarû security gate: pre-extraction byte scan ───────────────────
        let scan = musaru_scan(&data);
        if scan.malware_detected {
            self.emit(DwAlert::error(format!(
                "{fname}: SECURITY BLOCK — {} (Musarû sig #{})",
                scan.detail,
                scan.matched_sig_index.unwrap_or(0)
            )));
            self.stats.records_skipped += 1;
            return;
        }

        let results = zip_engine::extract(&data);
        self.stats.zips_processed += 1;
        self.emit(DwAlert::info(format!("{fname}: extracted {} ZIP entries", results.len())));

        for result in results {
            match result {
                Ok(entry) => self.process_zip_entry(&entry.name, &entry.data),
                Err(ZipError::UnsupportedMethod { ref name, method }) => {
                    self.emit(DwAlert::warning(format!(
                        "{fname}/{name}: method={method} not supported (use zip -0)"
                    )));
                    self.stats.records_skipped += 1;
                }
                Err(e) => {
                    self.emit(DwAlert::error(format!("{fname}: ZIP error — {e}")));
                    self.stats.records_skipped += 1;
                }
            }
        }
    }

    fn process_zip_entry(&mut self, entry_name: &str, data: &[u8]) {
        let is_csv = entry_name.to_lowercase().ends_with(".csv");
        let is_tsv = entry_name.to_lowercase().ends_with(".tsv");
        let is_way = entry_name.to_lowercase().ends_with(".way");

        if is_tsv || is_csv {
            let records = if is_csv {
                kaki_generator::parse_csv(data)
            } else {
                kaki_generator::parse_tsv(data)
            };
            self.ingest_records(entry_name, records);
        } else if is_way {
            let src = match std::str::from_utf8(data) {
                Ok(s)  => s.to_string(),
                Err(_) => {
                    self.emit(DwAlert::error(format!("{entry_name}: invalid UTF-8 in .way file")));
                    return;
                }
            };
            self.compile_way_source(&src, entry_name);
        } else {
            self.emit(DwAlert::warning(format!("{entry_name}: skipped (not TSV/CSV/WAY)")));
        }
    }

    fn process_way(&mut self, path: &Path, fname: &str) {
        let src = match fs::read_to_string(path) {
            Ok(s)  => s,
            Err(e) => {
                self.emit(DwAlert::error(format!("{fname}: cannot read — {e}")));
                return;
            }
        };
        self.compile_way_source(&src, fname);
    }

    fn compile_way_source(&mut self, src: &str, fname: &str) {
        match WayFile::parse(src).and_then(|w| way_compiler::compile(&w).map_err(|e| e)) {
            Ok(res) => {
                self.stats.way_compiled += 1;
                self.emit(DwAlert::info(format!(
                    "{fname}: .way compiled → {} AAOL chars",
                    res.akk_source.len()
                )));
            }
            Err(e) => {
                self.emit(DwAlert::error(format!("{fname}: .way compile error — {e}")));
                self.stats.records_skipped += 1;
            }
        }
    }

    fn process_records(&mut self, path: &Path, fname: &str, is_csv: bool) {
        let data = match fs::read(path) {
            Ok(d)  => d,
            Err(e) => {
                self.emit(DwAlert::error(format!("{fname}: read error — {e}")));
                self.stats.records_skipped += 1;
                return;
            }
        };
        let records = if is_csv {
            kaki_generator::parse_csv(&data)
        } else {
            kaki_generator::parse_tsv(&data)
        };
        self.ingest_records(fname, records);
    }

    fn ingest_records(&mut self, source: &str, records: Vec<RawRecord>) {
        if records.is_empty() { return; }
        let count = records.len();

        // ── Schema sniff + compare ───────────────────────────────────────────
        let headers = &records[0].headers;
        if !headers.is_empty() {
            let tribe_name  = strip_ext(source);
            self.schema_version += 1;
            let new_schema = headers_to_schema(headers, &tribe_name, self.schema_version);
            if let Some(ref old_schema) = self.last_schema {
                let report = compare_versions(old_schema, &new_schema, &tribe_name, self.schema_version);
                if report.verdict == CompareVerdict::SchemaMismatch {
                    if report.has_breaking_changes() {
                        self.emit(DwAlert::error(format!("{source}: {}", report.summary())));
                        self.stats.records_skipped += count;
                        return;
                    }
                    self.emit(DwAlert::warning(format!("{source}: {}", report.summary())));
                }
            } else {
                self.emit(DwAlert::info(format!(
                    "{source}: first-run schema captured — {} column(s): {}",
                    headers.len(),
                    headers.join(", ")
                )));
            }
            self.last_schema = Some(new_schema);
        }
        // ────────────────────────────────────────────────────────────────────

        let mut ok = 0usize;

        for record in records {
            let ge = kaki_generator::generate(&self.minter, &record);
            if let Err(e) = self.pdb.register_particle(&ge.particle) {
                self.emit(DwAlert::error(format!("{source}: register error — {e}")));
                self.stats.records_skipped += 1;
                continue;
            }
            match self.pdb.commit(ge.event_kaki, ge.particle, ge.epoch, ge.eav) {
                Ok(())  => ok += 1,
                Err(e)  => {
                    self.emit(DwAlert::error(format!("{source}: commit error — {e}")));
                    self.stats.records_skipped += 1;
                }
            }
        }

        self.stats.records_ingested += ok;
        self.stats.records_skipped  += count - ok;
        if ok > 0 {
            self.emit(DwAlert::info(format!("{source}: ingested {ok}/{count} records")));
        }
    }

    fn emit(&mut self, alert: DwAlert) {
        self.stats.alerts_emitted += 1;
        self.alerts.push(alert);
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    /// Drain accumulated alerts (clears the internal list).
    pub fn take_alerts(&mut self) -> Vec<DwAlert> {
        std::mem::take(&mut self.alerts)
    }

    /// Peek at alerts without consuming them.
    pub fn alerts(&self) -> &[DwAlert] { &self.alerts }

    pub fn stats(&self)       -> &EtlStats   { &self.stats }
    pub fn pdb(&self)         -> &PersistedDb { &self.pdb }
    pub fn pdb_mut(&mut self) -> &mut PersistedDb { &mut self.pdb }
}

fn headers_to_schema(headers: &[String], tribe_name: &str, version: u32) -> SchemaVersion {
    let fields = headers.iter().enumerate()
        .map(|(i, name)| FieldMeta::new(name.trim(), FieldType::Text, None, (i + 1) as u16, false, true))
        .collect();
    SchemaVersion::new(version, tribe_name, version, fields)
}

fn strip_ext(fname: &str) -> String {
    match fname.rfind('.') {
        Some(pos) => fname[..pos].to_string(),
        None      => fname.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use crate::zip_engine::build_store_zip;

    fn tmp_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("enkidw_etl_{}", tag));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn make_pipeline(base: &PathBuf) -> EtlPipeline {
        let landing  = base.join("landing");
        let data_dir = base.join("data");
        let tid      = TribeId::from_u16(0x0001);
        EtlPipeline::open(&landing, &data_dir, tid, FsyncPolicy::Never).unwrap()
    }

    #[test]
    fn ingest_tsv_from_landing_zone() {
        let base = tmp_dir("tsv");
        let mut pipe = make_pipeline(&base);

        let tsv = b"name\tepoch\tstate\nAli_Karim\t1\tGolden\nFatima\t2\tFuzzy\n";
        fs::write(base.join("landing").join("records.tsv"), tsv).unwrap();

        pipe.run_once();
        assert_eq!(pipe.stats().records_ingested, 2);
        assert_eq!(pipe.stats().records_skipped,  0);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn ingest_csv_from_landing_zone() {
        let base = tmp_dir("csv");
        let mut pipe = make_pipeline(&base);

        let csv = b"name,epoch,state\nBaghdad_Reg,3,Golden\n";
        fs::write(base.join("landing").join("data.csv"), csv).unwrap();

        pipe.run_once();
        assert_eq!(pipe.stats().records_ingested, 1);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn ingest_zip_with_tsv() {
        let base = tmp_dir("zip");
        let mut pipe = make_pipeline(&base);

        let tsv = b"name\tepoch\tstate\nOmar_Jawad\t1\tDead\n";
        let zip = build_store_zip("data.tsv", tsv);
        fs::write(base.join("landing").join("batch.zip"), &zip).unwrap();

        pipe.run_once();
        assert_eq!(pipe.stats().zips_processed,   1);
        assert_eq!(pipe.stats().records_ingested,  1);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn compile_way_file() {
        let base = tmp_dir("way");
        let mut pipe = make_pipeline(&base);

        let way_src = "tribe 0x0001\nclass civil.registry\nrole Zikru\n";
        fs::write(base.join("landing").join("najaf.way"), way_src).unwrap();

        pipe.run_once();
        assert_eq!(pipe.stats().way_compiled, 1);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn second_poll_sees_no_duplicates() {
        let base = tmp_dir("dedup");
        let mut pipe = make_pipeline(&base);

        let tsv = b"name\tepoch\nAli\t1\n";
        fs::write(base.join("landing").join("once.tsv"), tsv).unwrap();

        pipe.run_once();
        let first_count = pipe.stats().records_ingested;
        pipe.run_once(); // same file — should not be re-ingested
        assert_eq!(pipe.stats().records_ingested, first_count);
        let _ = fs::remove_dir_all(&base);
    }
}
