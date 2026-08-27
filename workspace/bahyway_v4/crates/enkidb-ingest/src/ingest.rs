//! CsvIngester — orchestrates the CSV → Particle → EnkiDb pipeline.

use bahyway_core::TribeId;
use enkidb_con_engine::NaruJournal;
use enkidb_engine::EnkiDb;
use enkidb_kaki::{mint::KakiMinter, IdentityKaki, KakiRole};

use crate::bridge::particle_to_eav_triple;
use crate::csv::{parse_csv, CsvError};
use crate::kispu::{self, KispuError};
use akkvalue::AkkValue;
use enkidb_particles::Particle;

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum IngestError {
    Csv(CsvError),
    Db(bahyway_core::BahywayError),
    /// The KISPU all-or-nothing commit refused a row -- neither its audit
    /// record nor its ledger entry landed (see `kispu::commit`).
    Kispu(KispuError),
}

impl core::fmt::Display for IngestError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Csv(e) => write!(f, "ingest CSV: {e}"),
            Self::Db(e) => write!(f, "ingest DB: {e:?}"),
            Self::Kispu(e) => write!(f, "ingest commit: {e}"),
        }
    }
}

impl From<CsvError> for IngestError {
    fn from(e: CsvError) -> Self {
        Self::Csv(e)
    }
}
impl From<bahyway_core::BahywayError> for IngestError {
    fn from(e: bahyway_core::BahywayError) -> Self {
        Self::Db(e)
    }
}
impl From<KispuError> for IngestError {
    fn from(e: KispuError) -> Self {
        Self::Kispu(e)
    }
}

// ── Report ────────────────────────────────────────────────────────────────────

/// Summary of one completed ingestion run.
#[derive(Debug, Clone)]
pub struct IngestReport {
    /// Number of CSV data rows processed.
    pub rows_processed: usize,
    /// Number of EAV particles written to the DB.
    pub particles_written: usize,
    /// Number of null values skipped (null fields are not stored).
    pub nulls_skipped: usize,
    /// Identity KAKIs minted, one per CSV row.
    pub entity_kakis: Vec<IdentityKaki>,
    /// NĀRU audit records committed alongside the ledger entries above —
    /// always equal to `entity_kakis.len()` on a fully successful run,
    /// since KISPU commits both legs together or neither (`kispu::commit`).
    pub audit_entries_committed: usize,
}

// ── CsvIngester ───────────────────────────────────────────────────────────────

/// Parses CSV text and ingests it into an `EnkiDb`.
///
/// Each CSV row becomes one entity (minted `IdentityKaki`).
/// Each non-null column value becomes one EAV particle under that entity.
///
/// Timestamp is fixed at ingestion time (`born_at`); pass 0 for unit tests.
///
/// Null fields (empty CSV cells) are skipped — they become row-absent nulls
/// per NullSemantics::RowAbsent.
pub struct CsvIngester {
    tribe_id: TribeId,
    csv_text: String,
    born_at: u64,
    audit_capacity: usize,
}

/// Default NĀRU audit journal capacity for one ingest run. Generous
/// enough that ordinary CSV files never hit it; override with
/// `with_audit_capacity` for a run expected to exceed it.
const DEFAULT_AUDIT_CAPACITY: usize = 1_000_000;

impl CsvIngester {
    /// Construct from raw CSV text.
    pub fn from_str(csv_text: impl Into<String>, tribe_id: TribeId) -> Self {
        Self {
            tribe_id,
            csv_text: csv_text.into(),
            born_at: 0,
            audit_capacity: DEFAULT_AUDIT_CAPACITY,
        }
    }

    /// Set the birth timestamp for all minted particles.
    pub fn with_timestamp(mut self, born_at: u64) -> Self {
        self.born_at = born_at;
        self
    }

    /// Override the NĀRU audit journal's fixed capacity for this run.
    pub fn with_audit_capacity(mut self, audit_capacity: usize) -> Self {
        self.audit_capacity = audit_capacity;
        self
    }

    /// Parse + ingest into the given `EnkiDb`.
    ///
    /// On success, returns an `IngestReport` with the minted entity KAKIs.
    /// Each row's ledger entry and NĀRU audit record commit together via
    /// `kispu::commit` — a row that fails to commit lands in neither, so
    /// the DB and the audit journal are never left disagreeing about how
    /// many rows actually made it in (earlier rows in the same run stay
    /// committed; the run overall reports the error).
    pub fn ingest_into(self, db: &mut EnkiDb) -> Result<IngestReport, IngestError> {
        let rows = parse_csv(&self.csv_text)?;
        let minter = KakiMinter::new(self.tribe_id);
        let mut audit = NaruJournal::new(self.audit_capacity);

        let mut report = IngestReport {
            rows_processed: 0,
            particles_written: 0,
            nulls_skipped: 0,
            entity_kakis: Vec::with_capacity(rows.len()),
            audit_entries_committed: 0,
        };

        for row in rows {
            report.rows_processed += 1;

            // Mint one IdentityKaki per row — it is the entity's sovereign identity
            let identity_kaki = IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru))
                .expect("KakiMinter always produces valid Identity KAKIs");

            // Register the particle in the identity + sovereignty indexes
            db.register_particle(&identity_kaki)?;

            // Build EAV triples for all non-null fields
            let mut eav_triples = Vec::new();

            for (attr_name, value) in &row.fields {
                if matches!(value, AkkValue::Null) {
                    report.nulls_skipped += 1;
                    continue;
                }

                // Create a Particle (for clean API) then convert to EavTriple
                let particle = Particle::base(
                    identity_kaki,
                    attr_name.as_str(),
                    value.clone(),
                    self.born_at,
                );
                eav_triples.push(particle_to_eav_triple(&particle));
                report.particles_written += 1;
            }

            // Mint an EventKaki for this ingestion event
            let event_kaki = enkidb_kaki::EventKaki::try_from_kaki(minter.event(KakiRole::Zikru))
                .expect("KakiMinter always produces valid Event KAKIs");

            let epoch = (self.born_at & 0xFFFF_FFFF) as u32;

            // KISPU: audit record + journal/index write, all-or-nothing.
            kispu::commit(&mut audit, db, event_kaki, identity_kaki, epoch, eav_triples)?;

            report.entity_kakis.push(identity_kaki);
        }

        report.audit_entries_committed = audit.len();
        Ok(report)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::eav_triple_to_value;
    use akkvalue::AkkValue;

    fn make_db(tribe_id: TribeId) -> EnkiDb {
        EnkiDb::new(tribe_id)
    }

    #[test]
    fn ingest_simple_csv() {
        let tribe = TribeId::from_u16(0x0001);
        let mut db = make_db(tribe);

        let csv = "name,age,city\nAlice,30,Najaf\nBob,25,Baghdad";
        let report = CsvIngester::from_str(csv, tribe)
            .with_timestamp(1_700_000_000)
            .ingest_into(&mut db)
            .unwrap();

        assert_eq!(report.rows_processed, 2);
        assert_eq!(report.entity_kakis.len(), 2);
        // Alice: name(text) + age(int) + city(text) = 3 particles
        // Bob:   name(text) + age(int) + city(text) = 3 particles
        assert_eq!(report.particles_written, 6);
        assert_eq!(report.nulls_skipped, 0);
    }

    #[test]
    fn null_fields_are_skipped() {
        let tribe = TribeId::from_u16(0x0001);
        let mut db = make_db(tribe);
        let csv = "a,b,c\n1,,3";
        let report = CsvIngester::from_str(csv, tribe)
            .ingest_into(&mut db)
            .unwrap();
        assert_eq!(report.rows_processed, 1);
        assert_eq!(report.particles_written, 2); // a and c
        assert_eq!(report.nulls_skipped, 1); // b
    }

    #[test]
    fn entity_kakis_are_unique() {
        let tribe = TribeId::from_u16(0x0001);
        let mut db = make_db(tribe);
        let csv = "x\n1\n2\n3";
        let report = CsvIngester::from_str(csv, tribe)
            .ingest_into(&mut db)
            .unwrap();
        assert_eq!(report.entity_kakis.len(), 3);
        // Each kaki must be unique (KakiMinter generates distinct kakis)
        let k0 = report.entity_kakis[0];
        let k1 = report.entity_kakis[1];
        let k2 = report.entity_kakis[2];
        assert_ne!(k0, k1);
        assert_ne!(k1, k2);
        assert_ne!(k0, k2);
    }

    #[test]
    fn value_survives_round_trip_through_journal() {
        let tribe = TribeId::from_u16(0x0001);
        let mut db = make_db(tribe);
        let csv = "temperature,humidity\n23.7,65.0";
        let report = CsvIngester::from_str(csv, tribe)
            .ingest_into(&mut db)
            .unwrap();

        // Read back from the journal and decode the first entry's EAV triples
        let entries = db.journal().read_particle_history(&report.entity_kakis[0]);
        assert!(!entries.is_empty());

        let eav = &entries[0].eav;
        // Two columns → two EAV triples
        assert_eq!(eav.len(), 2);

        // Decode and verify values
        let v0 = eav_triple_to_value(&eav[0]);
        let v1 = eav_triple_to_value(&eav[1]);

        assert_eq!(v0, AkkValue::Float(23.7));
        assert_eq!(v1, AkkValue::Float(65.0));
    }

    #[test]
    fn bool_csv_ingested_correctly() {
        let tribe = TribeId::from_u16(0x0001);
        let mut db = make_db(tribe);
        let csv = "active\ntrue\nfalse";
        let report = CsvIngester::from_str(csv, tribe)
            .ingest_into(&mut db)
            .unwrap();
        assert_eq!(report.rows_processed, 2);
        assert_eq!(report.particles_written, 2);
    }

    #[test]
    fn malformed_csv_returns_error() {
        let tribe = TribeId::from_u16(0x0001);
        let mut db = make_db(tribe);
        // Column mismatch
        let csv = "a,b\n1,2,3";
        assert!(CsvIngester::from_str(csv, tribe)
            .ingest_into(&mut db)
            .is_err());
    }

    #[test]
    fn arabic_names_ingested() {
        let tribe = TribeId::from_u16(0x0001);
        let mut db = make_db(tribe);
        let csv = "name,city\nمحمد,النجف\nعلي,بغداد";
        let report = CsvIngester::from_str(csv, tribe)
            .ingest_into(&mut db)
            .unwrap();
        assert_eq!(report.rows_processed, 2);
        assert_eq!(report.particles_written, 4);
    }

    #[test]
    fn empty_csv_produces_zero_rows() {
        let tribe = TribeId::from_u16(0x0001);
        let mut db = make_db(tribe);
        // Header only, no data rows
        let csv = "name,age";
        let report = CsvIngester::from_str(csv, tribe)
            .ingest_into(&mut db)
            .unwrap();
        assert_eq!(report.rows_processed, 0);
        assert_eq!(report.particles_written, 0);
    }
}
