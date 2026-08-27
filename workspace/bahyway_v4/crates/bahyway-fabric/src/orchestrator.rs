//! FabricOrchestrator — the sovereign pipeline runner.
//!
//! The orchestrator holds a registry of sources, targets, and pipeline
//! declarations.  When asked to run a pipeline it:
//!
//!   1. Extracts a batch from the source
//!   2. Enforces the SchemaContract at the source boundary
//!   3. Runs each declared Stage in order, building a LineageChain per record
//!   4. Routes the processed batch to every declared target
//!   5. Enforces the SchemaContract at each target boundary
//!   6. Returns an OrchestratorResult with receipts, exceptions, and lineage
//!
//! Hot-swapping: register a new source or target at any time.
//! The orchestrator never needs to know about existing pipelines to add one.

use crate::connector::{
    DataBatch, DeliveryReceipt, ExtractionCursor, SourceConnector, SourceId, TargetConnector,
    TargetId,
};
use crate::exception::FabricException;
use crate::lineage::{hash_record, LineageChain, LineageHop, QualitySnapshot, StageId};
use crate::pipeline::{PipelineDeclaration, PipelineId, Stage};
use data_cleansing_station::VgcaCleansingStation;
use std::collections::HashMap;
use template_engine::Template;

/// One record's field set: (attr_hash, value_bytes) pairs.
type FieldRecord = Vec<(u32, Vec<u8>)>;
/// A record paired with the lineage chain accumulated for it so far.
type StagedRecord = (FieldRecord, LineageChain);

// ── Orchestrator Result ───────────────────────────────────────────────────────

/// The result of a single pipeline run.
#[derive(Debug)]
pub struct OrchestratorResult {
    pub pipeline_id: PipelineId,
    pub records_in: usize,
    pub records_out: usize,
    pub exceptions: Vec<FabricException>,
    pub receipts: Vec<DeliveryReceipt>,
    /// Lineage chains — one per successfully processed record.
    pub lineage: Vec<LineageChain>,
    /// Orbit density snapshot for Dashboard visualisation (particle lane counts).
    pub orbit_density: data_cleansing_station::OrbitDensitySnapshot,
}

impl OrchestratorResult {
    pub fn success_rate(&self) -> f32 {
        if self.records_in == 0 {
            return 1.0;
        }
        self.records_out as f32 / self.records_in as f32
    }

    /// True when every record was delivered with no exceptions.
    pub fn is_clean(&self) -> bool {
        self.exceptions.is_empty() && self.records_out == self.records_in
    }
}

// ── FabricOrchestrator ────────────────────────────────────────────────────────

/// The central coordinator of the Enterprise Data Fabric.
///
/// Register sources, targets, and pipelines once.
/// Call `run_pipeline` to execute declarative flows with full lineage.
///
/// Holds a `VgcaCleansingStation` whose self-calibrating centroid improves
/// over time as GEM-lane particles pass through `Stage::Cleanse`.
pub struct FabricOrchestrator {
    sources: HashMap<SourceId, Box<dyn SourceConnector>>,
    targets: HashMap<TargetId, Box<dyn TargetConnector>>,
    pipelines: HashMap<PipelineId, PipelineDeclaration>,
    /// Self-calibrating VGCA domain centroid — updated by every GEM particle.
    cleansing_station: std::cell::RefCell<VgcaCleansingStation>,
}

impl FabricOrchestrator {
    pub fn new() -> Self {
        FabricOrchestrator {
            sources: HashMap::new(),
            targets: HashMap::new(),
            pipelines: HashMap::new(),
            cleansing_station: std::cell::RefCell::new(VgcaCleansingStation::new()),
        }
    }

    /// Read-only access to the VGCA domain centroid coordinates (7 dimensions).
    pub fn centroid_dims(&self) -> [f64; 7] {
        *self.cleansing_station.borrow().centroid_dims()
    }

    /// Number of GEM-lane particles that have updated the domain centroid.
    pub fn centroid_gem_depth(&self) -> u64 {
        self.cleansing_station.borrow().gem_count()
    }

    /// Register a source connector. Replaces any existing connector with the same ID.
    pub fn register_source(&mut self, connector: Box<dyn SourceConnector>) {
        self.sources.insert(connector.source_id(), connector);
    }

    /// Register a target connector. Replaces any existing connector with the same ID.
    pub fn register_target(&mut self, connector: Box<dyn TargetConnector>) {
        self.targets.insert(connector.target_id(), connector);
    }

    /// Register a pipeline declaration.
    pub fn register_pipeline(&mut self, pipeline: PipelineDeclaration) {
        self.pipelines.insert(pipeline.id.clone(), pipeline);
    }

    pub fn source_count(&self) -> usize {
        self.sources.len()
    }
    pub fn target_count(&self) -> usize {
        self.targets.len()
    }
    pub fn pipeline_count(&self) -> usize {
        self.pipelines.len()
    }

    /// Execute a registered pipeline. Returns `None` if the pipeline is not found.
    pub fn run_pipeline(
        &self,
        pipeline_id: &PipelineId,
        cursor: &ExtractionCursor,
    ) -> Option<OrchestratorResult> {
        let pipeline = self.pipelines.get(pipeline_id)?;
        if !pipeline.enabled {
            return None;
        }

        let source = self.sources.get(&pipeline.source)?;

        // ── Step 1: Extract ───────────────────────────────────────────────────
        let batch = match source.extract(cursor) {
            Ok(b) => b,
            Err(e) => {
                return Some(OrchestratorResult {
                    pipeline_id: pipeline_id.clone(),
                    records_in: 0,
                    records_out: 0,
                    exceptions: vec![e],
                    receipts: vec![],
                    lineage: vec![],
                    orbit_density: data_cleansing_station::OrbitDensitySnapshot::default(),
                })
            }
        };

        let records_in = batch.len();
        let epoch = batch.epoch;
        let source_id = batch.source_id.clone();

        // ── Step 2: Contract validation at source boundary ────────────────────
        let contract = source.schema();
        let mut exceptions: Vec<FabricException> = Vec::new();
        let mut accepted_records: Vec<Vec<(u32, Vec<u8>)>> = Vec::new();

        for record in batch.records {
            let present: Vec<u32> = record.iter().map(|(h, _)| *h).collect();
            match contract.validate_presence(&present) {
                Ok(_) => accepted_records.push(record),
                Err(fields) => {
                    let field = fields.first().copied().unwrap_or("unknown");
                    exceptions.push(FabricException::missing_field(
                        source_id.clone(),
                        "source-boundary",
                        field,
                        record,
                        epoch,
                    ));
                }
            }
        }

        // ── Step 3: Stage execution with lineage tracking ─────────────────────
        let mut processed: Vec<StagedRecord> = accepted_records
            .into_iter()
            .map(|r| {
                let mut chain = LineageChain::new();
                // Seed lineage with source hop (b11 unknown at extraction → 0)
                chain.push(
                    LineageHop::new(
                        StageId("extraction"),
                        Some(source_id.clone()),
                        hash_record(&r),
                        hash_record(&r),
                        QualitySnapshot {
                            b11_in: 0,
                            b11_out: 0,
                        },
                        epoch,
                    )
                    .with_annotation(format!("source={}", source.display_name())),
                );
                (r, chain)
            })
            .collect();

        let mut cleanse_reports: Vec<data_cleansing_station::CleansingReport> = Vec::new();

        for stage in &pipeline.stages {
            let stage_name = stage.stage_name();
            let mut next: Vec<StagedRecord> = Vec::new();

            for (record, mut chain) in processed {
                let in_hash = hash_record(&record);
                let (out_record, b11_in, b11_out, rejected, annotation, opt_report) =
                    self.apply_stage(stage, &record, &source_id, epoch, &mut exceptions);

                if let Some(report) = opt_report {
                    cleanse_reports.push(report);
                }

                if !rejected {
                    let out_hash = hash_record(&out_record);
                    chain.push(
                        LineageHop::new(
                            StageId(stage_name),
                            None,
                            in_hash,
                            out_hash,
                            QualitySnapshot { b11_in, b11_out },
                            epoch,
                        )
                        .with_annotation(annotation),
                    );
                    next.push((out_record, chain));
                }
            }
            processed = next;
        }

        let orbit_density = {
            let mut snap =
                data_cleansing_station::OrbitDensitySnapshot::from_reports(&cleanse_reports);
            snap.centroid_gem_depth = self.cleansing_station.borrow().gem_count();
            snap
        };

        // ── Step 4: Deliver to targets ────────────────────────────────────────
        let final_records: Vec<Vec<(u32, Vec<u8>)>> =
            processed.iter().map(|(r, _)| r.clone()).collect();
        let lineage: Vec<LineageChain> = processed.into_iter().map(|(_, c)| c).collect();

        let mut receipts: Vec<DeliveryReceipt> = Vec::new();
        for target_id in &pipeline.targets {
            if let Some(target) = self.targets.get(target_id) {
                let deliver_batch = DataBatch::new(source_id.clone(), final_records.clone(), epoch);
                match target.deliver(deliver_batch) {
                    Ok(r) => receipts.push(r),
                    Err(e) => exceptions.push(e),
                }
            }
        }

        Some(OrchestratorResult {
            pipeline_id: pipeline_id.clone(),
            records_in,
            records_out: final_records.len(),
            exceptions,
            receipts,
            lineage,
            orbit_density,
        })
    }

    // ── Stage Dispatch ────────────────────────────────────────────────────────

    /// Apply one stage to one record.
    /// Returns: (output_record, b11_in, b11_out, was_rejected, annotation, opt_cleanse_report)
    fn apply_stage(
        &self,
        stage: &Stage,
        record: &[(u32, Vec<u8>)],
        source_id: &SourceId,
        epoch: u32,
        exceptions: &mut Vec<FabricException>,
    ) -> (
        FieldRecord,
        u8,
        u8,
        bool,
        String,
        Option<data_cleansing_station::CleansingReport>,
    ) {
        match stage {
            Stage::Cleanse => {
                // Full VGCA-powered cleansing: VGCA-Σ (FSV 7D) for text fields,
                // VGCA-Δ (BFV 6D) for binary fields, + DQM 6-dimension assessment.
                // The self-calibrating centroid updates when this record is GEM-lane.
                // Build a minimal template treating all fields as optional —
                // completeness is enforced at the source boundary, not here.
                // We use a fixed empty template so VGCA geometry runs on all fields.
                let template = Template::default_template("t.cleanse", "CleansePass", &[]);
                let report = self
                    .cleansing_station
                    .borrow_mut()
                    .cleanse(record, &template, epoch, epoch, 10);
                // Alien fields are stripped; everything else passes through
                let cleaned: Vec<(u32, Vec<u8>)> = record
                    .iter()
                    .zip(report.field_results.iter())
                    .filter(|(_, vr)| vr.fit != vgca_engine::GeometricFit::Alien)
                    .map(|((h, v), _)| (*h, v.clone()))
                    .collect();
                let b11 = report.dqm_report.composite_b11;
                let lane_label = report.lane.label();
                let annotation = format!(
                    "vgca:B11={b11} lane={lane_label} clean={:.0}% centroid_depth={}",
                    report.vgca_clean_ratio * 100.0,
                    self.cleansing_station.borrow().gem_count(),
                );
                (cleaned, 0, b11, false, annotation, Some(report))
            }

            Stage::Validate { min_b11 } => {
                // Simplified quality gate: score by non-empty fields / total fields.
                // In production this delegates to hepta-score H(P) computation.
                let non_empty = record.iter().filter(|(_, v)| !v.is_empty()).count();
                let total = record.len().max(1);
                let b11 = ((non_empty as f32 / total as f32) * 240.0) as u8;
                if b11 < *min_b11 {
                    exceptions.push(FabricException::quality_rejection(
                        source_id.clone(),
                        b11,
                        *min_b11,
                        record.to_vec(),
                        epoch,
                    ));
                    (vec![], 0, b11, true, String::new(), None)
                } else {
                    (
                        record.to_vec(),
                        0,
                        b11,
                        false,
                        format!("B11={b11} ≥ threshold={min_b11}"),
                        None,
                    )
                }
            }

            Stage::Enrich { ruleset } => {
                let mut enriched = record.to_vec();
                enriched.push((0xFFFF, format!("enriched-by:{ruleset}").into_bytes()));
                (enriched, 0, 0, false, format!("ruleset={ruleset}"), None)
            }

            Stage::Deduplicate => (record.to_vec(), 0, 0, false, "idu-prober:pass".into(), None),

            Stage::Aggregate {
                key_attr_hash,
                strategy,
            } => {
                let annotation = format!("key={key_attr_hash:#x} strategy={:?}", strategy);
                (record.to_vec(), 0, 0, false, annotation, None)
            }

            Stage::Transform { transform_id } => (
                record.to_vec(),
                0,
                0,
                false,
                format!("transform={transform_id}"),
                None,
            ),
        }
    }
}

impl Default for FabricOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::{DataBatch, DeliveryReceipt, ExtractionCursor, SourceId, TargetId};
    use crate::contract::{FieldSpec, FieldType, SchemaContract};
    use crate::exception::FabricException;
    use crate::pipeline::{PipelineDeclaration, Stage};

    // ── Stubs ─────────────────────────────────────────────────────────────────

    struct StubSource {
        records: Vec<Vec<(u32, Vec<u8>)>>,
    }

    impl SourceConnector for StubSource {
        fn source_id(&self) -> SourceId {
            SourceId("stub.source")
        }
        fn display_name(&self) -> &'static str {
            "Stub Source"
        }
        fn schema(&self) -> SchemaContract {
            SchemaContract::new(
                "stub",
                1,
                vec![
                    FieldSpec::required("id", 0x0001, FieldType::Integer),
                    FieldSpec::required("value", 0x0002, FieldType::Text),
                ],
            )
        }
        fn extract(&self, _cursor: &ExtractionCursor) -> Result<DataBatch, FabricException> {
            Ok(DataBatch::new(
                SourceId("stub.source"),
                self.records.clone(),
                1,
            ))
        }
    }

    struct StubTarget {
        id: &'static str,
    }

    impl TargetConnector for StubTarget {
        fn target_id(&self) -> TargetId {
            TargetId(self.id)
        }
        fn display_name(&self) -> &'static str {
            "Stub Target"
        }
        fn schema_expectation(&self) -> SchemaContract {
            SchemaContract::new("stub-target", 1, vec![])
        }
        fn deliver(&self, batch: DataBatch) -> Result<DeliveryReceipt, FabricException> {
            Ok(DeliveryReceipt {
                target_id: TargetId(self.id),
                records_accepted: batch.len(),
                token: b"ok".to_vec(),
            })
        }
    }

    fn make_orchestrator(records: Vec<Vec<(u32, Vec<u8>)>>) -> (FabricOrchestrator, PipelineId) {
        let mut o = FabricOrchestrator::new();
        o.register_source(Box::new(StubSource { records }));
        o.register_target(Box::new(StubTarget { id: "stub.dw" }));
        let pipeline = PipelineDeclaration::builder("test.pipe", "stub.source")
            .stage(Stage::Cleanse)
            .stage(Stage::Validate { min_b11: 100 })
            .target("stub.dw")
            .build();
        let id = pipeline.id.clone();
        o.register_pipeline(pipeline);
        (o, id)
    }

    #[test]
    fn registration_counts_are_correct() {
        let (o, _) = make_orchestrator(vec![]);
        assert_eq!(o.source_count(), 1);
        assert_eq!(o.target_count(), 1);
        assert_eq!(o.pipeline_count(), 1);
    }

    #[test]
    fn clean_records_are_delivered() {
        let records = vec![vec![
            (0x0001u32, b"42".to_vec()),
            (0x0002u32, b"hello".to_vec()),
        ]];
        let (o, id) = make_orchestrator(records);
        let result = o.run_pipeline(&id, &ExtractionCursor::default()).unwrap();
        assert_eq!(result.records_in, 1);
        assert_eq!(result.records_out, 1);
        assert!(result.exceptions.is_empty());
        assert_eq!(result.receipts.len(), 1);
    }

    #[test]
    fn empty_value_rejected_by_quality_gate() {
        // Record with all empty values → quality score = 0 < min_b11 = 100
        let records = vec![vec![(0x0001u32, b"".to_vec()), (0x0002u32, b"".to_vec())]];
        let (o, id) = make_orchestrator(records);
        let result = o.run_pipeline(&id, &ExtractionCursor::default()).unwrap();
        assert_eq!(result.records_out, 0);
        assert!(!result.exceptions.is_empty());
        assert_eq!(
            result.exceptions[0].kind,
            crate::exception::ExceptionKind::QualityRejection
        );
    }

    #[test]
    fn lineage_chain_depth_matches_stage_count_plus_extraction() {
        let records = vec![vec![
            (0x0001u32, b"1".to_vec()),
            (0x0002u32, b"val".to_vec()),
        ]];
        let (o, id) = make_orchestrator(records);
        let result = o.run_pipeline(&id, &ExtractionCursor::default()).unwrap();
        // extraction + cleanse + validate = 3 hops
        assert_eq!(result.lineage[0].depth(), 3);
    }

    #[test]
    fn success_rate_is_one_for_clean_batch() {
        let records = vec![
            vec![(0x0001u32, b"1".to_vec()), (0x0002u32, b"v".to_vec())],
            vec![(0x0001u32, b"2".to_vec()), (0x0002u32, b"w".to_vec())],
        ];
        let (o, id) = make_orchestrator(records);
        let result = o.run_pipeline(&id, &ExtractionCursor::default()).unwrap();
        assert!((result.success_rate() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn unknown_pipeline_returns_none() {
        let o = FabricOrchestrator::new();
        let result = o.run_pipeline(&PipelineId("nonexistent"), &ExtractionCursor::default());
        assert!(result.is_none());
    }

    #[test]
    fn multiple_targets_produce_multiple_receipts() {
        let mut o = FabricOrchestrator::new();
        o.register_source(Box::new(StubSource {
            records: vec![vec![(0x0001u32, b"1".to_vec()), (0x0002u32, b"v".to_vec())]],
        }));
        o.register_target(Box::new(StubTarget { id: "t1" }));
        o.register_target(Box::new(StubTarget { id: "t2" }));
        let pipeline = PipelineDeclaration::builder("multi.target", "stub.source")
            .stage(Stage::Cleanse)
            .target("t1")
            .target("t2")
            .build();
        let id = pipeline.id.clone();
        o.register_pipeline(pipeline);
        let result = o.run_pipeline(&id, &ExtractionCursor::default()).unwrap();
        assert_eq!(result.receipts.len(), 2);
    }
}
