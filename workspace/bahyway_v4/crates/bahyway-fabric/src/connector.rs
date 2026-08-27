//! Connector traits — the typed boundary between the Fabric and the outside world.
//!
//! Implementing `SourceConnector` or `TargetConnector` is the ONLY way to attach
//! a new system (ERP, CRM, Excel, API…) to the Fabric.  Adding a connector never
//! touches existing pipelines — zero spaghetti by construction.

use crate::contract::SchemaContract;
use crate::exception::FabricException;

// ── Identifiers ───────────────────────────────────────────────────────────────

/// Unique identifier for a source connector (e.g. "erp.sap", "crm.salesforce").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceId(pub &'static str);

/// Unique identifier for a target connector (e.g. "dw.snowflake", "notify.email").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TargetId(pub &'static str);

// ── Data Batch ────────────────────────────────────────────────────────────────

/// A batch of raw records extracted from a source.
/// Each record is a list of (attr_hash, raw_bytes) pairs — the same format
/// the AdadGate understands, keeping the Fabric natively KAKI-compatible.
#[derive(Debug, Clone)]
pub struct DataBatch {
    pub source_id: SourceId,
    /// Each record is a Vec of (attr_hash, value) pairs.
    pub records: Vec<Vec<(u32, Vec<u8>)>>,
    /// Monotonic epoch assigned by the source at extraction time.
    pub epoch: u32,
}

impl DataBatch {
    pub fn new(source_id: SourceId, records: Vec<Vec<(u32, Vec<u8>)>>, epoch: u32) -> Self {
        DataBatch {
            source_id,
            records,
            epoch,
        }
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

/// Cursor that tracks the extraction position inside a source,
/// allowing incremental / resumable extraction without re-scanning.
#[derive(Debug, Clone, Default)]
pub struct ExtractionCursor {
    /// Last successfully processed epoch.
    pub last_epoch: u32,
    /// Opaque offset blob interpreted by each connector.
    pub offset: Vec<u8>,
}

// ── Delivery Receipt ─────────────────────────────────────────────────────────

/// Proof that a target received and accepted a batch.
#[derive(Debug, Clone)]
pub struct DeliveryReceipt {
    pub target_id: TargetId,
    pub records_accepted: usize,
    /// Target-assigned confirmation token (e.g. commit ID, message ID).
    pub token: Vec<u8>,
}

// ── Source Connector Trait ───────────────────────────────────────────────────

/// Every data source (ERP, CRM, HR, Excel, API…) implements this trait.
///
/// The Fabric calls `extract` on a schedule managed by EriduScheduler.
/// The connector owns the connection logic; the Fabric owns what happens next.
pub trait SourceConnector: Send + Sync {
    fn source_id(&self) -> SourceId;

    /// The contract this source guarantees to produce.
    fn schema(&self) -> SchemaContract;

    /// Pull the next batch of records since `cursor`.
    fn extract(&self, cursor: &ExtractionCursor) -> Result<DataBatch, FabricException>;

    /// Human-readable display name (shown in Dubsar lineage view).
    fn display_name(&self) -> &'static str;
}

// ── Target Connector Trait ───────────────────────────────────────────────────

/// Every data consumer (Data Warehouse, Dashboard, Portal, Notification…) implements this trait.
///
/// The Fabric calls `deliver` after all pipeline stages have run.
/// The connector owns the delivery logic; the Fabric owns the audit trail.
pub trait TargetConnector: Send + Sync {
    fn target_id(&self) -> TargetId;

    /// The contract this target requires to accept a batch.
    fn schema_expectation(&self) -> SchemaContract;

    /// Deliver a processed batch. Returns a receipt or a structured exception.
    fn deliver(&self, batch: DataBatch) -> Result<DeliveryReceipt, FabricException>;

    /// Human-readable display name.
    fn display_name(&self) -> &'static str;
}
