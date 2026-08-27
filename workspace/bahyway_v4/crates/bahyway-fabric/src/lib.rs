//! bahyway-fabric — BahyWay Enterprise Data Fabric
//!
//! The sovereign, transparent answer to enterprise "Spaghetti Processing".
//!
//! # The Five Spaghetti Problems — Solved
//!
//! | Problem                         | This Crate's Answer                                      |
//! |---------------------------------|----------------------------------------------------------|
//! | No single source of truth       | KAKI sovereign identity — every particle born once       |
//! | Hard to trace & maintain        | `LineageLedger` — every hop: source → stage → target     |
//! | High cost & complexity          | `PipelineDeclaration` — declare WHAT, Ur runs HOW    |
//! | Error prone & inconsistent      | `SchemaContract` enforced at every connector boundary    |
//! | Slow changes & time to market   | Hot-swappable connectors — add a source, touch nothing   |
//!
//! # Architecture
//!
//! ```text
//! SOURCES (ERP, CRM, HR, Legacy, Partner, Excel, Email, API)
//!     ↓  [SourceConnector + SchemaContract validation]
//! FABRIC STAGES (Cleanse → Validate → Enrich → Deduplicate → Aggregate)
//!     ↓  [LineageChain built per record through every hop]
//! TARGETS (DataWarehouse, Dashboards, Portals, Notifications, FileExports)
//! ```
//!
//! # Quick Start
//!
//! ```rust
//! use bahyway_fabric::{
//!     orchestrator::FabricOrchestrator,
//!     pipeline::{PipelineDeclaration, Stage},
//!     adapters::{ErpConnector, DataWarehouseTarget, NotificationTarget},
//!     connector::ExtractionCursor,
//!     pipeline::PipelineId,
//! };
//!
//! let mut fabric = FabricOrchestrator::new();
//!
//! // Register sources — hot-swappable, zero impact on existing pipelines
//! fabric.register_source(Box::new(ErpConnector));
//!
//! // Register targets
//! fabric.register_target(Box::new(DataWarehouseTarget));
//! fabric.register_target(Box::new(NotificationTarget));
//!
//! // Declare pipeline — WHAT, not HOW
//! let pipeline = PipelineDeclaration::builder("erp → dw", "erp.sovereign")
//!     .description("Extract ERP records, cleanse, validate quality, load to DW + notify")
//!     .stage(Stage::Cleanse)
//!     .stage(Stage::Validate { min_b11: 140 })
//!     .stage(Stage::Deduplicate)
//!     .stage(Stage::Enrich { ruleset: "erp.v1" })
//!     .target("dw.central")
//!     .target("notify.sovereign")
//!     .build();
//!
//! fabric.register_pipeline(pipeline);
//!
//! // Run — full lineage, structured exceptions, delivery receipts
//! let result = fabric.run_pipeline(
//!     &PipelineId("erp → dw"),
//!     &ExtractionCursor::default(),
//! ).unwrap();
//!
//! assert!(result.exceptions.is_empty() || !result.lineage.is_empty());
//! ```

pub mod adapters;
pub mod connector;
pub mod contract;
pub mod exception;
pub mod lineage;
pub mod orchestrator;
pub mod pipeline;
pub mod pipeline_flow;

/// `lineage`'s sovereign name — NUZI. `LineageLedger`'s immutable,
/// hop-by-hop provenance chain (source -> stage -> stage, hash-proven,
/// append-only) is what NUZI names across BahyWay's documents. Not a
/// crate/type rename: `bahyway-fabric` and `LineageLedger` keep their
/// existing identifiers — same relationship "Tigris" has to `enkiddb`
/// and "Anu" has to `enkidb-indexes`.
pub const NUZI: &str = "nuzi";

pub mod prelude {
    pub use crate::adapters::{
        CrmConnector, DashboardTarget, DataWarehouseTarget, EmailInboxConnector, ErpConnector,
        ExcelFileConnector, ExternalPartnerConnector, ExternalPortalTarget, FileExportTarget,
        HrSystemConnector, LegacySystemConnector, NotificationTarget, OtherApplicationsTarget,
        ReportingToolsTarget, ThirdPartyApiConnector,
    };
    pub use crate::connector::{
        DataBatch, DeliveryReceipt, ExtractionCursor, SourceConnector, SourceId, TargetConnector,
        TargetId,
    };
    pub use crate::contract::{FieldSpec, FieldType, SchemaContract};
    pub use crate::exception::{ExceptionKind, FabricException};
    pub use crate::lineage::{LineageChain, LineageHop, QualitySnapshot, StageId};
    pub use crate::orchestrator::{FabricOrchestrator, OrchestratorResult};
    pub use crate::pipeline::{
        AggregationStrategy, ExceptionPolicy, ExceptionRule, PipelineDeclaration, PipelineId, Stage,
    };
}
