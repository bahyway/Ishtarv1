//! SdbPipeline — drives LandingZone → Musarû byte scan → ZipEngine →
//! CSV/TSV parse → SdbStore staging.
//!
//! Every particle is assigned EventCause::KakiBorn and written to the Journal
//! at the moment it enters the stage database.  The ValidationSweep later
//! appends the SdbValidationPass / SdbValidationFail cause when it decides.

use std::fs;
use std::path::Path;

use bahyway_core::TribeId;
use enkidb_journal::{EventCause, Journal, JournalEntry};
use enkidb_kaki::{EventKaki, KakiMinter, KakiRole};
use enkidw::{
    kaki_generator::{generate, parse_csv, parse_tsv, RawRecord},
    zip_extract, LandingFileKind, LandingZone,
};

use crate::sdb_store::{SdbStatus, SdbStore, StagedParticle};
use musaru_security::zip_scan;

// ── Alert ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SdbAlertSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct SdbAlert {
    pub severity: SdbAlertSeverity,
    pub message: String,
}

impl SdbAlert {
    fn info(msg: impl Into<String>) -> Self {
        SdbAlert {
            severity: SdbAlertSeverity::Info,
            message: msg.into(),
        }
    }
    fn warning(msg: impl Into<String>) -> Self {
        SdbAlert {
            severity: SdbAlertSeverity::Warning,
            message: msg.into(),
        }
    }
    fn error(msg: impl Into<String>) -> Self {
        SdbAlert {
            severity: SdbAlertSeverity::Error,
            message: msg.into(),
        }
    }
}

// ── Stats ────────────────────────────────────────────────────────────────────

#[derive(Debug, Default, Clone)]
pub struct SdbPipelineStats {
    pub files_seen: usize,
    pub zips_processed: usize,
    pub records_staged: usize,
    pub records_skipped: usize,
    pub malware_hits: usize,
    pub alerts_emitted: usize,
}

// ── Pipeline ─────────────────────────────────────────────────────────────────

pub struct SdbPipeline {
    landing: LandingZone,
    minter: KakiMinter,
    stats: SdbPipelineStats,
    alerts: Vec<SdbAlert>,
}

impl SdbPipeline {
    pub fn open(landing_path: &Path, tribe_id: TribeId) -> std::io::Result<Self> {
        let landing = LandingZone::new(landing_path)?;
        Ok(SdbPipeline {
            landing,
            minter: KakiMinter::new(tribe_id),
            stats: SdbPipelineStats::default(),
            alerts: Vec::new(),
        })
    }

    /// Poll the landing zone and stage every new file's particles.
    /// Each particle's arrival is journalled with EventCause::KakiBorn.
    pub fn run_once(&mut self, store: &mut SdbStore, journal: &mut Journal) -> &SdbPipelineStats {
        let new_files = self.landing.poll();
        if new_files.is_empty() {
            return &self.stats;
        }

        self.emit(SdbAlert::info(format!(
            "{} file(s) arrived in SDB landing zone",
            new_files.len()
        )));
        self.stats.files_seen += new_files.len();

        for lf in new_files {
            let fname = lf
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            // Archive the raw file out of the landing root *before*
            // processing. LandingZone's dedup ("seen") state is in-memory
            // only — without this, a fresh SdbPipeline (e.g. after a
            // restart) would rediscover every already-staged file still
            // sitting in the landing root and stage it again, duplicating
            // particles that already passed through the pipeline.
            let archived_path = match self.landing.archive(&lf) {
                Ok(p) => p,
                Err(e) => {
                    self.emit(SdbAlert::error(format!(
                        "{fname}: failed to archive source — {e}"
                    )));
                    continue;
                }
            };

            match lf.kind {
                LandingFileKind::Zip => self.process_zip(&archived_path, &fname, store, journal),
                LandingFileKind::Csv => {
                    self.process_records(&archived_path, &fname, true, store, journal)
                }
                LandingFileKind::Tsv => {
                    self.process_records(&archived_path, &fname, false, store, journal)
                }
                LandingFileKind::Way | LandingFileKind::Other => {
                    self.emit(SdbAlert::warning(format!(
                        "{fname}: SDB does not process WAY/other files"
                    )));
                }
            }
        }

        &self.stats
    }

    fn process_zip(
        &mut self,
        path: &Path,
        fname: &str,
        store: &mut SdbStore,
        journal: &mut Journal,
    ) {
        let data = match fs::read(path) {
            Ok(d) => d,
            Err(e) => {
                self.emit(SdbAlert::error(format!("{fname}: read error — {e}")));
                return;
            }
        };

        let malware_in_zip = zip_scan(&data).malware_detected;
        if malware_in_zip {
            self.stats.malware_hits += 1;
            self.emit(SdbAlert::error(format!(
                "{fname}: Musarû pre-scan flagged malware signature"
            )));
        }

        self.stats.zips_processed += 1;
        let entries = zip_extract(&data);
        self.emit(SdbAlert::info(format!(
            "{fname}: extracted {} ZIP entries",
            entries.len()
        )));

        for result in entries {
            match result {
                Ok(entry) => {
                    let is_csv = entry.name.to_lowercase().ends_with(".csv");
                    let is_tsv = entry.name.to_lowercase().ends_with(".tsv");
                    if is_csv || is_tsv {
                        let records = if is_csv {
                            parse_csv(&entry.data)
                        } else {
                            parse_tsv(&entry.data)
                        };
                        self.stage_records(&entry.name, records, malware_in_zip, store, journal);
                    } else {
                        self.emit(SdbAlert::warning(format!(
                            "{}/{}: skipped (not CSV/TSV)",
                            fname, entry.name
                        )));
                    }
                }
                Err(e) => {
                    self.emit(SdbAlert::warning(format!("{fname}: ZIP entry error — {e}")));
                    self.stats.records_skipped += 1;
                }
            }
        }
    }

    fn process_records(
        &mut self,
        path: &Path,
        fname: &str,
        is_csv: bool,
        store: &mut SdbStore,
        journal: &mut Journal,
    ) {
        let data = match fs::read(path) {
            Ok(d) => d,
            Err(e) => {
                self.emit(SdbAlert::error(format!("{fname}: read error — {e}")));
                return;
            }
        };
        let records = if is_csv {
            parse_csv(&data)
        } else {
            parse_tsv(&data)
        };
        self.stage_records(fname, records, false, store, journal);
    }

    fn stage_records(
        &mut self,
        source: &str,
        records: Vec<RawRecord>,
        malware_flag: bool,
        store: &mut SdbStore,
        journal: &mut Journal,
    ) {
        let count = records.len();
        let mut ok = 0usize;

        for record in records {
            let ge = generate(&self.minter, &record);

            let event_kaki = match EventKaki::try_from_kaki(self.minter.event(KakiRole::Zikru)) {
                Ok(ek) => ek,
                Err(_) => {
                    self.stats.records_skipped += 1;
                    continue;
                }
            };

            // Default Color_ID(RGB) at staging: (128, 200, 255) = mid-severity, high-quality, fresh
            let color_rgb: [u8; 3] = [128, 200, 255];

            // Write KakiBorn event to the Journal.
            let entry = JournalEntry::new(event_kaki, ge.particle, ge.epoch, ge.eav.clone())
                .with_color_snapshot(color_rgb, 0.0)
                .with_event_cause(EventCause::KakiBorn);
            let _ = journal.append(entry);

            store.stage(StagedParticle {
                kaki_bytes: *ge.particle.bytes(),
                tribe_id: ge.particle.tribe_id().as_u16(),
                epoch: ge.epoch,
                eav: ge.eav,
                color_rgb,
                status: SdbStatus::Pending,
                arrived_tick: 0,
                malware_flag,
            });

            ok += 1;
        }

        self.stats.records_staged += ok;
        self.stats.records_skipped += count - ok;
        if ok > 0 {
            self.emit(SdbAlert::info(format!(
                "{source}: staged {ok}/{count} particles"
            )));
        }
    }

    fn emit(&mut self, alert: SdbAlert) {
        self.stats.alerts_emitted += 1;
        self.alerts.push(alert);
    }

    pub fn take_alerts(&mut self) -> Vec<SdbAlert> {
        std::mem::take(&mut self.alerts)
    }
    pub fn alerts(&self) -> &[SdbAlert] {
        &self.alerts
    }
    pub fn stats(&self) -> &SdbPipelineStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enkidw::build_store_zip;
    use std::fs;
    use std::path::PathBuf;

    fn tmp_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("enkisdb_pipe_{}", tag));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn make_pipeline(base: &PathBuf) -> (SdbPipeline, SdbStore, Journal) {
        let landing = base.join("landing");
        let tid = TribeId::from_u16(0x0001);
        let pipe = SdbPipeline::open(&landing, tid).unwrap();
        (pipe, SdbStore::new(), Journal::new(64))
    }

    #[test]
    fn tsv_records_staged_and_journalled() {
        let base = tmp_dir("tsv");
        let (mut pipe, mut store, mut jnl) = make_pipeline(&base);

        let tsv = b"name\tepoch\tstate\nAli_Karim\t1\tGolden\nFatima\t2\tFuzzy\n";
        fs::write(base.join("landing").join("records.tsv"), tsv).unwrap();

        pipe.run_once(&mut store, &mut jnl);

        assert_eq!(pipe.stats().records_staged, 2);
        assert_eq!(store.stats().total_staged, 2);
        assert_eq!(jnl.entry_count(), 2);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn zip_with_csv_staged() {
        let base = tmp_dir("zip");
        let (mut pipe, mut store, mut jnl) = make_pipeline(&base);

        let csv = b"name,epoch,state\nBaghdad_Reg,3,Golden\n";
        let zip = build_store_zip("data.csv", csv);
        fs::write(base.join("landing").join("batch.zip"), &zip).unwrap();

        pipe.run_once(&mut store, &mut jnl);

        assert_eq!(pipe.stats().zips_processed, 1);
        assert_eq!(pipe.stats().records_staged, 1);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn malware_signature_flagged() {
        let base = tmp_dir("malware");
        let (mut pipe, mut store, mut jnl) = make_pipeline(&base);

        // Embed EICAR signature inside a ZIP file alongside a TSV
        let tsv = b"name\nMalwareBearer\n";
        let mut zip = build_store_zip("payload.tsv", tsv);
        zip.extend_from_slice(b"EICAR-STANDARD-ANTIVIRUS-TEST-FILE");
        fs::write(base.join("landing").join("suspect.zip"), &zip).unwrap();

        pipe.run_once(&mut store, &mut jnl);

        assert_eq!(pipe.stats().malware_hits, 1);
        // Particle is staged but with malware_flag = true
        let staged = store.all();
        assert_eq!(staged.len(), 1);
        assert!(staged[0].malware_flag);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn no_duplicate_staging_on_second_poll() {
        let base = tmp_dir("dedup");
        let (mut pipe, mut store, mut jnl) = make_pipeline(&base);

        let tsv = b"name\nOnlyOnce\n";
        fs::write(base.join("landing").join("once.tsv"), tsv).unwrap();

        pipe.run_once(&mut store, &mut jnl);
        let first = pipe.stats().records_staged;
        pipe.run_once(&mut store, &mut jnl);
        assert_eq!(pipe.stats().records_staged, first); // not re-staged
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn no_duplicate_staging_after_restart() {
        // A fresh SdbPipeline (simulating a process restart) pointed at the
        // same landing directory must not re-stage a file a prior instance
        // already processed — the raw file must have been archived out of
        // the landing root, not left there relying on in-memory dedup state.
        let base = tmp_dir("restart_dedup");
        let landing_dir = base.join("landing");

        {
            let (mut pipe, mut store, mut jnl) = make_pipeline(&base);
            fs::write(landing_dir.join("once.tsv"), b"name\nOnlyOnce\n").unwrap();
            pipe.run_once(&mut store, &mut jnl);
            assert_eq!(pipe.stats().records_staged, 1);
        }

        let tid = TribeId::from_u16(0x0001);
        let mut pipe2 = SdbPipeline::open(&landing_dir, tid).unwrap();
        let mut store2 = SdbStore::new();
        let mut jnl2 = Journal::new(64);
        pipe2.run_once(&mut store2, &mut jnl2);

        assert_eq!(
            pipe2.stats().records_staged,
            0,
            "restart must not re-stage an already-archived file"
        );
        assert_eq!(store2.stats().total_staged, 0);
        let _ = fs::remove_dir_all(&base);
    }
}
