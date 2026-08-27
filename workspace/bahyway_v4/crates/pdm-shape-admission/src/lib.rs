//! pdm-shape-admission — commit a stakeholder-defined Tribe Shape's EAV
//! schema into BahyWay's external stores.
//!
//! Closes a real, previously-undecided gap: `docs/06_governance_parzu/
//! MANDATORY_VS_OPTIONAL_ATTRIBUTES.md` names GL-EAV-001's Subject-Area
//! Attribute Registry (Layer 2) as "registry design is sound, zero
//! implementation yet." This crate is that implementation, wired to the
//! shape a stakeholder actually creates in DubSar PDM IDE (with SLA +
//! OntoGraph) — the trigger that did not exist yet when the registry was
//! first designed, and the reason it stayed on paper.
//!
//! What actually gets written, and where, and why this is deliberately
//! NOT a naive four-way fan-out:
//!
//!   1. Staging write ([`admit_into_staging`]) — ONE commit into the
//!      combined EnkiSDB/EnkiODB/EnkiQDB write path. These three are not
//!      three independent stores in the real deployment: the real
//!      `enkisdb-write-server` container owns all three together and
//!      drains Promoted/Quarantined particles into EnkiODB/EnkiQDB's own
//!      Read Nodes (`deploy/podman/Containerfile.enkisdb-write-server`).
//!      Modelling this as three separate atomic writes would invent a
//!      partial-failure mode that does not exist in the real topology —
//!      one `EnkiDb` + one KISPU commit is the honest shape.
//!
//!   2. Golden promotion ([`promote_into_golden`]) — a SEPARATE, later
//!      commit into EnkiDB-golden. `playbook_212`'s own deploy language
//!      ("EnkiDB-core, the BeeMDM ETL...") treats Golden as reached via
//!      ETL promotion, not written simultaneously with staging. Bundling
//!      it into the same atomic commit as step 1 would claim a
//!      simultaneity the real pipeline does not have.
//!
//!   3. EnkiDW is never written here at all. [`project_dw_schema`] reads
//!      EnkiDB-golden's own schema back — consistent with ADR-012 ("the
//!      read node ... holds no journal at all and serves exclusively
//!      from indexed, materialized Data Files") and with the
//!      shape-vs-instance resolution already reached: the warehouse's
//!      shape is a projection, rebuilt by replay, never a second
//!      independent write.
//!
//! Each of the two real writes (staging, golden) is itself all-or-nothing
//! via `enkidb_ingest::kispu::commit` — the audit record and the ledger
//! entry for that store land together or not at all.

#![forbid(unsafe_code)]

use akkvalue::AkkValue;
use bahyway_core::TribeId;
use enkidb_con_engine::NaruJournal;
use enkidb_engine::EnkiDb;
use enkidb_ingest::bridge::attr_value_to_eav_triple;
use enkidb_ingest::kispu::{self, KispuError};
use enkidb_journal::entry::EavTriple;
use enkidb_kaki::{mint::KakiMinter, EventKaki, IdentityKaki, KakiRole};

/// GL-EAV-001's seven real Subject Areas (Layer 2). Not invented here —
/// named exactly as `docs/06_governance_parzu/
/// MANDATORY_VS_OPTIONAL_ATTRIBUTES.md` records them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubjectArea {
    Assets,
    Architecture,
    Algorithms,
    Batches,
    Processes,
    Environments,
    Knowledge,
}

impl SubjectArea {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Assets => "Assets",
            Self::Architecture => "Architecture",
            Self::Algorithms => "Algorithms",
            Self::Batches => "Batches",
            Self::Processes => "Processes",
            Self::Environments => "Environments",
            Self::Knowledge => "Knowledge",
        }
    }
}

/// One attribute in a stakeholder-defined Tribe Shape.
///
/// `mandatory` here is DAMA-DMBOK "mandatory within this Subject Area" —
/// distinct from, and layered on top of, the Hepta-7 universal mandatory
/// attributes every particle already carries regardless of Subject Area
/// (`story_engine::projection`). This crate never touches the Hepta-7.
#[derive(Debug, Clone)]
pub struct AttributeDef {
    pub name: String,
    pub subject_area: SubjectArea,
    pub mandatory: bool,
    pub datatype: String,
}

/// The shape a stakeholder builds in DubSar PDM IDE (with SLA + OntoGraph)
/// before any particle of this tribe exists.
#[derive(Debug, Clone)]
pub struct TribeShape {
    pub tribe_id: TribeId,
    pub attributes: Vec<AttributeDef>,
}

/// What one admission commit produced.
#[derive(Debug)]
pub struct AdmissionReport {
    pub shape_kaki: IdentityKaki,
    pub attributes_committed: usize,
}

#[derive(Debug)]
pub enum AdmissionError {
    Kispu(KispuError),
}

impl core::fmt::Display for AdmissionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Kispu(e) => write!(f, "shape admission commit failed: {e}"),
        }
    }
}

impl From<KispuError> for AdmissionError {
    fn from(e: KispuError) -> Self {
        Self::Kispu(e)
    }
}

/// Encode one `TribeShape` as EAV triples: one row per attribute,
/// `shape.attr.<name>` -> `subject_area=<X>|mandatory=<bool>|datatype=<Y>`.
fn shape_eav_triples(shape: &TribeShape) -> Vec<EavTriple> {
    shape
        .attributes
        .iter()
        .map(|a| {
            let attr_name = format!("shape.attr.{}", a.name);
            let value = AkkValue::Text(format!(
                "subject_area={}|mandatory={}|datatype={}",
                a.subject_area.as_str(),
                a.mandatory,
                a.datatype
            ));
            attr_value_to_eav_triple(&attr_name, &value)
        })
        .collect()
}

/// Commit the shape's schema into the combined EnkiSDB/EnkiODB/EnkiQDB
/// write path — one `EnkiDb` because that is how the real
/// `enkisdb-write-server` deployment owns all three (see module docs).
pub fn admit_into_staging(
    shape: &TribeShape,
    staging_db: &mut EnkiDb,
    staging_audit: &mut NaruJournal,
    born_at: u64,
) -> Result<AdmissionReport, AdmissionError> {
    commit_shape(shape, staging_db, staging_audit, born_at)
}

/// Commit the same shape into EnkiDB-golden — a separate, later step
/// (BeeMDM promotion), never bundled atomically with staging (see module
/// docs for why that would misrepresent the real pipeline).
pub fn promote_into_golden(
    shape: &TribeShape,
    golden_db: &mut EnkiDb,
    golden_audit: &mut NaruJournal,
    born_at: u64,
) -> Result<AdmissionReport, AdmissionError> {
    commit_shape(shape, golden_db, golden_audit, born_at)
}

fn commit_shape(
    shape: &TribeShape,
    db: &mut EnkiDb,
    audit: &mut NaruJournal,
    born_at: u64,
) -> Result<AdmissionReport, AdmissionError> {
    let minter = KakiMinter::new(shape.tribe_id);

    // Parzu ("logic, template, axiom, or rule") — a Shape is a template,
    // never a record/entity (Zikru) or an external file (Kishib).
    let shape_kaki = IdentityKaki::try_from_kaki(minter.identity(KakiRole::Parzu))
        .expect("KakiMinter always produces valid Identity KAKIs");
    db.register_particle(&shape_kaki)
        .map_err(|e| AdmissionError::Kispu(KispuError::Db(e)))?;

    let event_kaki = EventKaki::try_from_kaki(minter.event(KakiRole::Parzu))
        .expect("KakiMinter always produces valid Event KAKIs");
    let epoch = (born_at & 0xFFFF_FFFF) as u32;
    let eav = shape_eav_triples(shape);
    let attributes_committed = eav.len();

    kispu::commit(audit, db, event_kaki, shape_kaki, epoch, eav)?;

    Ok(AdmissionReport {
        shape_kaki,
        attributes_committed,
    })
}

/// EnkiDW never gets its own write. Its schema is a projection read back
/// from EnkiDB-golden's own state — the shape-vs-instance resolution
/// already reached: the warehouse's shape is derived, never an
/// independent write.
pub fn project_dw_schema(
    golden_db: &EnkiDb,
    shape_kaki: &IdentityKaki,
) -> story_engine::projected_state::ProjectedState {
    golden_db.project(shape_kaki)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_shape(tribe_id: TribeId) -> TribeShape {
        TribeShape {
            tribe_id,
            attributes: vec![
                AttributeDef {
                    name: "site_name".to_string(),
                    subject_area: SubjectArea::Assets,
                    mandatory: true,
                    datatype: "Text".to_string(),
                },
                AttributeDef {
                    name: "commissioned_date".to_string(),
                    subject_area: SubjectArea::Assets,
                    mandatory: false,
                    datatype: "Date".to_string(),
                },
            ],
        }
    }

    #[test]
    fn staging_admission_commits_one_row_per_attribute() {
        let tid = TribeId::from_u16(0x0042);
        let shape = sample_shape(tid);
        let mut db = EnkiDb::new(tid);
        let mut audit = NaruJournal::new(8);

        let report = admit_into_staging(&shape, &mut db, &mut audit, 1_700_000_000).unwrap();

        assert_eq!(report.attributes_committed, 2);
        assert_eq!(audit.len(), 1, "one KISPU commit, one audit record");
        assert_eq!(db.event_count(&report.shape_kaki), 1);
    }

    #[test]
    fn staging_and_golden_are_two_separate_commits_not_one_atomic_fan_out() {
        let tid = TribeId::from_u16(0x0043);
        let shape = sample_shape(tid);

        let mut staging_db = EnkiDb::new(tid);
        let mut staging_audit = NaruJournal::new(8);
        let staging_report =
            admit_into_staging(&shape, &mut staging_db, &mut staging_audit, 1).unwrap();

        // Golden has NOT received anything yet -- it is a separate store,
        // and admit_into_staging never touched it.
        let mut golden_db = EnkiDb::new(tid);
        let mut golden_audit = NaruJournal::new(8);
        assert_eq!(golden_db.event_count(&staging_report.shape_kaki), 0);

        let golden_report =
            promote_into_golden(&shape, &mut golden_db, &mut golden_audit, 2).unwrap();

        assert_eq!(golden_report.attributes_committed, 2);
        assert_eq!(golden_audit.len(), 1);
        // Two independent audit trails -- staging's promotion did not
        // touch golden's, and vice versa.
        assert_eq!(staging_audit.len(), 1);
    }

    #[test]
    fn a_refused_staging_commit_leaves_no_partial_row() {
        let tid = TribeId::from_u16(0x0044);
        let shape = sample_shape(tid);
        let mut db = EnkiDb::new(tid);
        // Capacity 0 -- the audit leg refuses immediately (CSR-03).
        let mut audit = NaruJournal::new(0);

        let result = admit_into_staging(&shape, &mut db, &mut audit, 1);

        assert!(result.is_err());
        assert_eq!(audit.len(), 0);
    }

    #[test]
    fn dw_schema_is_a_read_not_a_write() {
        let tid = TribeId::from_u16(0x0045);
        let shape = sample_shape(tid);
        let mut golden_db = EnkiDb::new(tid);
        let mut golden_audit = NaruJournal::new(8);
        let report = promote_into_golden(&shape, &mut golden_db, &mut golden_audit, 1).unwrap();

        // project_dw_schema takes &EnkiDb (shared reference) -- it cannot
        // write to golden_db at all; this is enforced by the compiler,
        // not just by convention.
        let projected = project_dw_schema(&golden_db, &report.shape_kaki);
        assert_eq!(projected.events_seen, 1, "DW's view sees golden's one commit, nothing of its own");
    }
}
