//! PipelineDeclaration — declare WHAT, UrOS executes HOW.
//!
//! A pipeline is a named, versioned description of:
//!   - one source to extract from
//!   - an ordered list of processing stages
//!   - one or more targets to deliver to
//!   - an exception policy for failures
//!
//! Adding a new pipeline never modifies existing ones.
//! Changing a stage is a new version, not an in-place edit.
//! This is how you get "slow changes & time to market" down to hours.

use crate::connector::{SourceId, TargetId};
use crate::exception::ExceptionKind;

// ── Pipeline & Stage Identifiers ──────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PipelineId(pub &'static str);

/// The built-in processing stages available to every pipeline.
/// Each maps directly to an existing BahyWayv4 engine:
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stage {
    /// VGCA-based geometric cleansing (vgca-engine).
    Cleanse,
    /// H(P) quality validation — records below threshold are rejected (hepta-score).
    Validate { min_b11: u8 },
    /// Business logic enrichment — lookup + augment (shulman-engine / story-engine).
    Enrich { ruleset: &'static str },
    /// Cross-tribe de-duplication via IDU prober (idu-prober).
    Deduplicate,
    /// Aggregation reduction (sum / count / last-wins per key).
    Aggregate {
        key_attr_hash: u32,
        strategy: AggregationStrategy,
    },
    /// Custom transformation with a named transform function ID.
    Transform { transform_id: &'static str },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AggregationStrategy {
    Sum,
    Count,
    LastWins,
    FirstWins,
    Max,
    Min,
}

impl Stage {
    pub fn stage_name(&self) -> &'static str {
        match self {
            Stage::Cleanse => "cleanse",
            Stage::Validate { .. } => "validate",
            Stage::Enrich { .. } => "enrich",
            Stage::Deduplicate => "deduplicate",
            Stage::Aggregate { .. } => "aggregate",
            Stage::Transform { .. } => "transform",
        }
    }
}

// ── Exception Policy ──────────────────────────────────────────────────────────

/// What the Fabric should do when a stage raises a FabricException.
#[derive(Debug, Clone)]
pub enum ExceptionPolicy {
    /// Reject the offending record and continue processing the rest of the batch.
    RejectRecord,
    /// Send the exception to the dead-letter queue and continue.
    DeadLetter { queue_id: &'static str },
    /// Halt the entire pipeline run and raise an alert.
    HaltPipeline,
    /// Attempt retry up to `max_retries` times before falling back to `fallback`.
    Retry {
        max_retries: u8,
        fallback: Box<ExceptionPolicy>,
    },
}

/// Scope: which exception kinds this policy covers.
#[derive(Debug, Clone)]
pub struct ExceptionRule {
    pub applies_to: Vec<ExceptionKind>,
    pub policy: ExceptionPolicy,
}

// ── Pipeline Declaration ──────────────────────────────────────────────────────

/// A complete, versioned pipeline declaration.
///
/// Construct via `PipelineDeclaration::builder()`.
#[derive(Debug, Clone)]
pub struct PipelineDeclaration {
    pub id: PipelineId,
    /// Semantic version — bump when stages or routing change.
    pub version: u16,
    pub source: SourceId,
    pub stages: Vec<Stage>,
    pub targets: Vec<TargetId>,
    pub exception_rules: Vec<ExceptionRule>,
    /// Optional: human description (shown in Dubsar pipeline map).
    pub description: &'static str,
    pub enabled: bool,
}

impl PipelineDeclaration {
    pub fn builder(id: &'static str, source: &'static str) -> PipelineBuilder {
        PipelineBuilder::new(id, source)
    }
}

// ── Pipeline Builder ──────────────────────────────────────────────────────────

pub struct PipelineBuilder {
    id: PipelineId,
    version: u16,
    source: SourceId,
    stages: Vec<Stage>,
    targets: Vec<TargetId>,
    exception_rules: Vec<ExceptionRule>,
    description: &'static str,
}

impl PipelineBuilder {
    pub fn new(id: &'static str, source: &'static str) -> Self {
        PipelineBuilder {
            id: PipelineId(id),
            version: 1,
            source: SourceId(source),
            stages: Vec::new(),
            targets: Vec::new(),
            exception_rules: Vec::new(),
            description: "",
        }
    }

    pub fn version(mut self, v: u16) -> Self {
        self.version = v;
        self
    }
    pub fn description(mut self, d: &'static str) -> Self {
        self.description = d;
        self
    }

    pub fn stage(mut self, s: Stage) -> Self {
        self.stages.push(s);
        self
    }
    pub fn target(mut self, t: &'static str) -> Self {
        self.targets.push(TargetId(t));
        self
    }

    pub fn on_exception(mut self, rule: ExceptionRule) -> Self {
        self.exception_rules.push(rule);
        self
    }

    pub fn build(self) -> PipelineDeclaration {
        PipelineDeclaration {
            id: self.id,
            version: self.version,
            source: self.source,
            stages: self.stages,
            targets: self.targets,
            exception_rules: self.exception_rules,
            description: self.description,
            enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn erp_to_dw_pipeline() -> PipelineDeclaration {
        PipelineDeclaration::builder("erp.invoices → dw", "erp.sap")
            .version(1)
            .description("Extract ERP invoices, cleanse, validate quality, load to Data Warehouse")
            .stage(Stage::Cleanse)
            .stage(Stage::Validate { min_b11: 140 })
            .stage(Stage::Deduplicate)
            .stage(Stage::Enrich {
                ruleset: "erp.enrichment.v1",
            })
            .target("dw.central")
            .target("notify.alerts")
            .on_exception(ExceptionRule {
                applies_to: vec![ExceptionKind::QualityRejection],
                policy: ExceptionPolicy::DeadLetter {
                    queue_id: "dlq.quality",
                },
            })
            .build()
    }

    #[test]
    fn pipeline_has_correct_source() {
        let p = erp_to_dw_pipeline();
        assert_eq!(p.source, SourceId("erp.sap"));
    }

    #[test]
    fn pipeline_has_four_stages() {
        let p = erp_to_dw_pipeline();
        assert_eq!(p.stages.len(), 4);
    }

    #[test]
    fn pipeline_has_two_targets() {
        let p = erp_to_dw_pipeline();
        assert_eq!(p.targets.len(), 2);
    }

    #[test]
    fn validate_stage_carries_min_b11() {
        let p = erp_to_dw_pipeline();
        let validate = p
            .stages
            .iter()
            .find(|s| s.stage_name() == "validate")
            .unwrap();
        assert_eq!(*validate, Stage::Validate { min_b11: 140 });
    }

    #[test]
    fn pipeline_is_enabled_by_default() {
        assert!(erp_to_dw_pipeline().enabled);
    }

    #[test]
    fn stage_names_are_correct() {
        assert_eq!(Stage::Cleanse.stage_name(), "cleanse");
        assert_eq!(Stage::Deduplicate.stage_name(), "deduplicate");
        assert_eq!(Stage::Enrich { ruleset: "x" }.stage_name(), "enrich");
    }

    #[test]
    fn builder_bumps_version() {
        let p = PipelineDeclaration::builder("test", "src")
            .version(3)
            .build();
        assert_eq!(p.version, 3);
    }
}
