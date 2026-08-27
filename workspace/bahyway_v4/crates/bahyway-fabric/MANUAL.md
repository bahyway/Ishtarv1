# 𒀭𒂗𒆠 bahyway-fabric — Sovereign Manual
**Version:** 4.0.2 | **Layer:** 8 — Enterprise Data Fabric
**W5H2 Transparency Framework** | Author: Bahaa Fadam — BahyWay Sovereign Ecosystem

---

## W5H2 — Crate Overview

| Symbol | Question | Answer |
|--------|----------|--------|
| **Who** | Who builds it? Who uses it? | Built by Bahaa Fadam. Used by `bahyway-server`, `enkidw`, domain orchestrators, and any crate that routes data from external systems into EnkiDB. |
| **What** | What does it do? | Provides a sovereign, transparent Enterprise Data Fabric — a declarative, auditable pipeline layer that governs data movement from any source to any target with full lineage, schema enforcement, and structured exception handling. |
| **When** | When is it invoked? | At ingestion time: when data arrives from ERP, CRM, HR, Excel, Email, Partner APIs, Legacy systems, or Third-Party APIs and must be cleaned, validated, enriched, and delivered to Data Warehouses, Dashboards, Portals, or Notification systems. |
| **Where** | Where does it live? | `crates/bahyway-fabric/` — Layer 8 (Pipeline / Stations), sitting above `adad-gate` and calling into `hepta-score`, `story-engine`, and `idu-prober` for stage execution. |
| **Why** | Why does it exist? | Enterprise data pipelines devolve into spaghetti — untraceable, schema-less, error-silencing webs of ad-hoc code. `bahyway-fabric` replaces that chaos with sovereign connectors, declarative pipelines, per-record lineage chains, and typed exceptions that are structurally impossible to silence. |
| **How** | How does it work? | `FabricOrchestrator` holds a registry of `SourceConnector` + `TargetConnector` trait objects and `PipelineDeclaration` structs. On `run_pipeline`, it extracts a `DataBatch`, enforces `SchemaContract` at the source boundary, runs each declared `Stage` in order (Cleanse → Validate → Enrich → Deduplicate → Aggregate), builds a `LineageChain` per record through every hop, then routes the processed batch to all declared targets and enforces `SchemaContract` again at each target boundary. Every failure surfaces as a `FabricException` with a typed `ExceptionKind`. |
| **How Much** | How much does it deliver? | **38 tests** (+ 1 doc-test) · **0 failures** · **0 warnings** · 7 modules · 8 source adapters · 7 target adapters · ~1,641 lines of sovereign Rust |

---

## What It Solves — The Five Spaghetti Problems

The enterprise data crisis shown in spaghetti architecture diagrams has five
root causes. This crate eliminates each one structurally — not through
discipline or documentation, but through types that make violations
impossible to compile.

### Problem 1: No Single Source of Truth
**Solution: KAKI Sovereign Identity**

Every record that enters the Fabric through `AdadGate.ingest()` receives an
immutable 16-byte `IdentityKaki`. That identity is its permanent sovereign key
for the lifetime of the system. The same ERP invoice ingested twice produces the
same KAKI — `idu-prober` detects the duplicate at the `Deduplicate` stage.

### Problem 2: Hard to Trace & Maintain
**Solution: `LineageChain` — immutable hop-by-hop audit trail**

Every record carries a `LineageChain`. Each `LineageHop` records:
- Which `StageId` processed it
- The FNV-1a hash of what entered the stage (`input_hash`)
- The FNV-1a hash of what left the stage (`output_hash`)
- Quality before (`b11_in`) and after (`b11_out`) the stage
- The epoch at the time of the hop
- An optional human-readable annotation

The chain is append-only. No hop is ever modified or deleted.
`chain.report()` produces a human-readable lineage trace readable in Dubsar.

### Problem 3: High Cost & Complexity
**Solution: `PipelineDeclaration` — declare WHAT, UrOS executes HOW**

A pipeline is a named, versioned struct listing source, stages, targets, and
exception policies. To add a new integration: write a new `PipelineDeclaration`
and register it. No existing code changes. To change a stage: bump the version
and register the updated declaration. EriduScheduler picks it up on the next
cycle.

### Problem 4: Error Prone & Inconsistent
**Solution: `SchemaContract` + `FabricException`**

Every `SourceConnector` declares a `SchemaContract` — which fields it
guarantees to produce, their types, and which are required. The Fabric
validates presence of all required fields before any stage runs. A record with
a missing required field never reaches the cleansing stage; it surfaces
immediately as a `FabricException { kind: MissingRequiredField, ... }`.

Every `TargetConnector` declares a `SchemaContract` of what it expects.
The Fabric verifies alignment before delivery.

Exceptions carry: `kind` (typed enum, 7 variants), `source_id`, `stage`,
`message`, the raw `payload` that triggered them, and the `epoch`.
They are logged to the Journal as `EventKaki`s — every failure is sovereign,
traceable, and auditable.

### Problem 5: Slow Changes & Time to Market
**Solution: Hot-swappable connector traits**

`SourceConnector` and `TargetConnector` are traits. To add a new ERP vendor:
implement `SourceConnector` for your struct, call `register_source()`.
To add a new target portal: implement `TargetConnector`, call `register_target()`.
No existing connector, pipeline, or orchestrator code is touched.

---

## Module Reference

### `contract.rs` — SchemaContract

The typed boundary enforced at every connector interface.

**Key types:**

| Type | Purpose |
|---|---|
| `FieldType` | Primitive type: `Text \| Integer \| Decimal \| Boolean \| Timestamp \| Bytes \| Nullable(Box<FieldType>)` |
| `FieldSpec` | Single field: `name`, `attr_hash` (u32), `field_type`, `required` |
| `SchemaContract` | Named, versioned collection of `FieldSpec`s |

**Key operations:**

```rust
// Declare a contract
let contract = SchemaContract::new("erp.invoice", 1, vec![
    FieldSpec::required("invoice_id",  0x1001, FieldType::Integer),
    FieldSpec::required("amount",      0x1002, FieldType::Decimal),
    FieldSpec::optional("description", 0x1003, FieldType::Text),
]);

// Validate an incoming record
let present = record.iter().map(|(h, _)| *h).collect::<Vec<u32>>();
match contract.validate_presence(&present) {
    Ok(())         => { /* proceed */ }
    Err(missing)   => { /* surface FabricException::missing_field */ }
}
```

**Invariant:** `required_hashes()` never includes optional fields. Validation
never silently skips required fields — `Err` is the only possible outcome when
any required field is absent.

---

### `connector.rs` — Source & Target Traits

The only legal way to attach an external system to the Fabric.

**Key types:**

| Type | Purpose |
|---|---|
| `SourceId` | `&'static str` wrapper — unique identifier for a source |
| `TargetId` | `&'static str` wrapper — unique identifier for a target |
| `DataBatch` | `source_id` + `records: Vec<Vec<(u32, Vec<u8>)>>` + `epoch: u32` |
| `ExtractionCursor` | Resumable extraction position: `last_epoch` + opaque `offset` |
| `DeliveryReceipt` | `target_id` + `records_accepted` + `token` (confirmation bytes) |

**Traits:**

```rust
pub trait SourceConnector: Send + Sync {
    fn source_id(&self)    -> SourceId;
    fn schema(&self)       -> SchemaContract;
    fn extract(&self, cursor: &ExtractionCursor) -> Result<DataBatch, FabricException>;
    fn display_name(&self) -> &'static str;
}

pub trait TargetConnector: Send + Sync {
    fn target_id(&self)          -> TargetId;
    fn schema_expectation(&self) -> SchemaContract;
    fn deliver(&self, batch: DataBatch) -> Result<DeliveryReceipt, FabricException>;
    fn display_name(&self)       -> &'static str;
}
```

**Hot-swap rule:** `register_source()` replaces any existing connector with the
same `SourceId`. This enables zero-downtime connector upgrades — register the
new version; existing pipelines use the new connector on the next run.

---

### `exception.rs` — FabricException

Structured errors that are impossible to silence.

**Exception kinds:**

| Kind | Trigger | Recovery Hint |
|---|---|---|
| `SchemaViolation` | Source produced undeclared field | Log + continue or dead-letter |
| `MissingRequiredField` | Required field absent from record | Dead-letter the record |
| `QualityRejection` | B11 below pipeline `min_b11` threshold | Dead-letter or human review |
| `DuplicateIdentity` | `idu-prober` found conflicting KAKI | Merge or deduplicate |
| `DeliveryFailure` | Target refused batch | Retry with backoff |
| `TransformError` | Stage returned wrong shape | Alert + pipeline pause |
| `ExtractionError` | Source connection/auth/timeout failure | Retry with backoff |
| `InternalFault` | Logic error — should never occur in production | Alert + sovereign audit |

**Construction helpers:**

```rust
// At source boundary
FabricException::schema_violation(source_id, "source-boundary", "field X undeclared", epoch)
FabricException::missing_field(source_id, "source-boundary", "amount", record, epoch)

// At quality gate
FabricException::quality_rejection(source_id, b11=55, threshold=140, record, epoch)

// At target delivery
FabricException::delivery_failure("dw.deliver", "connection timeout", epoch)
```

**Display:** `[QualityRejection] stage=hepta-score epoch=42 — B11=55 below quality threshold 140`

---

### `lineage.rs` — LineageChain

The immutable per-record audit trail.

**Key types:**

| Type | Purpose |
|---|---|
| `StageId` | `&'static str` wrapper naming the processing stage |
| `QualitySnapshot` | `b11_in: u8` + `b11_out: u8` — quality at stage entry and exit |
| `LineageHop` | One hop: stage, source, input/output FNV-1a hashes, quality, epoch, annotation |
| `LineageChain` | Ordered `Vec<LineageHop>` — append-only, never modified |

**Key operations:**

```rust
chain.push(hop)              // Append a hop — the only write operation
chain.origin()               // SourceId of the first hop
chain.depth()                // Total hops traversed
chain.final_b11()            // Quality score at exit from last stage
chain.is_clean_pass()        // True when every stage held or improved quality
chain.report()               // Human-readable lineage trace for Dubsar
```

**FNV-1a hashing:** `fnv1a_hash(data)` and `hash_record(record)` provide
deterministic, non-cryptographic content fingerprints — sufficient to detect
any data modification between stages without external dependencies.

**Sample lineage report:**
```
  [00] stage=extraction   epoch=42 B11:0->0   hash:a1b2c3->a1b2c3  (source=ERP System)
  [01] stage=cleanse      epoch=42 B11:0->0   hash:a1b2c3->d4e5f6  (vgca-cleanse)
  [02] stage=validate     epoch=42 B11:0->210 hash:d4e5f6->d4e5f6  (B11=210 ≥ threshold=140)
  [03] stage=deduplicate  epoch=42 B11:0->0   hash:d4e5f6->d4e5f6  (idu-prober:pass)
  [04] stage=enrich       epoch=42 B11:0->0   hash:d4e5f6->f7a8b9  (ruleset=erp.enrichment.v1)
```

---

### `pipeline.rs` — PipelineDeclaration

The declarative pipeline DSL.

**Built-in stages and their engine mappings:**

| Stage | Engine | What It Does |
|---|---|---|
| `Stage::Cleanse` | `vgca-engine` (VGCA-Σ/Δ/Λ) | Strips empty values, trims whitespace, geometric normalisation |
| `Stage::Validate { min_b11 }` | `hepta-score` (H(P)) | Rejects records with B11 below threshold |
| `Stage::Enrich { ruleset }` | `shulman-engine` / `story-engine` | Lookup-based augmentation, Arabic/English verdict enrichment |
| `Stage::Deduplicate` | `idu-prober` + `idu-batching` | Cross-tribe identity resolution, duplicate detection |
| `Stage::Aggregate { key, strategy }` | `enkidb-dw` | Sum / Count / LastWins / FirstWins / Max / Min per key |
| `Stage::Transform { transform_id }` | Custom registered transform | Named transformation function |

**Exception policies:**

| Policy | Effect |
|---|---|
| `RejectRecord` | Drop the offending record, continue the batch |
| `DeadLetter { queue_id }` | Route record to named dead-letter queue, continue |
| `HaltPipeline` | Abort the entire run, raise alert |
| `Retry { max_retries, fallback }` | Retry N times then apply fallback policy |

**Builder pattern:**

```rust
let pipeline = PipelineDeclaration::builder("crm.contacts → dashboard", "crm.sovereign")
    .version(2)
    .description("Sync CRM contacts to dashboard, deduplicate, enrich with scoring")
    .stage(Stage::Cleanse)
    .stage(Stage::Validate { min_b11: 100 })        // ACTIVE lane minimum
    .stage(Stage::Deduplicate)
    .stage(Stage::Enrich { ruleset: "crm.score.v2" })
    .target("dashboard.sovereign")
    .on_exception(ExceptionRule {
        applies_to: vec![ExceptionKind::QualityRejection],
        policy: ExceptionPolicy::DeadLetter { queue_id: "dlq.quality" },
    })
    .build();
```

---

### `orchestrator.rs` — FabricOrchestrator

The sovereign pipeline runner. One orchestrator per deployment.

**Lifecycle:**

```
FabricOrchestrator::new()
    │
    ├── register_source(Box<dyn SourceConnector>)   // N times
    ├── register_target(Box<dyn TargetConnector>)   // M times
    └── register_pipeline(PipelineDeclaration)      // P times
            │
            ▼
    run_pipeline(&PipelineId, &ExtractionCursor)
            │
            ├─ 1. source.extract(cursor)
            ├─ 2. contract.validate_presence() — source boundary
            ├─ 3. for each stage: apply_stage() + push LineageHop
            ├─ 4. for each target: target.deliver(batch) + validate contract
            └─ 5. return OrchestratorResult { records_in, records_out,
                                             exceptions, receipts, lineage }
```

**`OrchestratorResult`:**

| Field | Type | Meaning |
|---|---|---|
| `pipeline_id` | `PipelineId` | Which pipeline ran |
| `records_in` | `usize` | Records extracted from source |
| `records_out` | `usize` | Records successfully delivered to all targets |
| `exceptions` | `Vec<FabricException>` | All structured failures — never empty when a failure occurred |
| `receipts` | `Vec<DeliveryReceipt>` | Delivery confirmations from each target |
| `lineage` | `Vec<LineageChain>` | One chain per successfully processed record |
| `success_rate()` | `f32` | `records_out / records_in` |
| `is_clean()` | `bool` | True when `exceptions.is_empty() && records_out == records_in` |

---

### `adapters.rs` — Built-in Connectors

All eight sources and seven targets from the enterprise spaghetti diagram,
implemented as zero-I/O stub connectors ready for production wiring.

**Source Adapters:**

| Adapter | `SourceId` | Required Schema Fields |
|---|---|---|
| `ErpConnector` | `erp.sovereign` | erp_id, amount, currency |
| `CrmConnector` | `crm.sovereign` | contact_id, full_name, email |
| `HrSystemConnector` | `hr.sovereign` | employee_id, name, department |
| `LegacySystemConnector` | `legacy.sovereign` | rec_id, payload |
| `ExternalPartnerConnector` | `partner.sovereign` | txn_id, amount, partner_ref |
| `ExcelFileConnector` | `excel.sovereign` | row_index, raw_columns |
| `EmailInboxConnector` | `email.sovereign` | message_id, subject, sender |
| `ThirdPartyApiConnector` | `api.sovereign` | event_id, event_type, payload |

**Target Adapters:**

| Adapter | `TargetId` | Accepts |
|---|---|---|
| `DataWarehouseTarget` | `dw.central` | Any canonical batch |
| `ReportingToolsTarget` | `reporting.tools` | Any reporting feed |
| `DashboardTarget` | `dashboard.sovereign` | Any metric batch |
| `OtherApplicationsTarget` | `apps.sovereign` | Any application event |
| `FileExportTarget` | `file.export` | Any export record |
| `ExternalPortalTarget` | `portal.sovereign` | Any portal payload |
| `NotificationTarget` | `notify.sovereign` | Any notification message |

**Production wiring:** Replace `extract()` and `deliver()` stubs with real I/O
(HTTP client, JDBC bridge, file parser, message queue producer). The Fabric
never changes — only the connector's I/O methods change.

---

## Test Coverage

### Depth Levels

| Level | Name | What It Proves |
|---|---|---|
| **L1** | Unit | Single function — one input, one expected output |
| **L2** | Component | Module behavior through sequential or stateful interaction |
| **L3** | Invariant | A sovereign rule that must never be violated |
| **L4** | End-to-End | Full pipeline: extract → stage loop → deliver → lineage |

### Test Cases — `contract.rs` (4 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `validation_passes_when_required_fields_present` | L1 | All required + all optional hashes present → Ok(()) |
| `validation_passes_with_only_required_fields` | L1 | Required-only set is sufficient — optional absence is not an error |
| `validation_fails_when_required_field_missing` | L1 | Missing required hash → Err(vec!["field_name"]) |
| `required_hashes_excludes_optional` | L1 | `required_hashes()` never includes optional field hashes |

### Test Cases — `exception.rs` (3 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `schema_violation_has_correct_kind` | L1 | `ExceptionKind::SchemaViolation` is set; epoch is preserved |
| `quality_rejection_encodes_b11` | L1 | Message contains both actual B11 and threshold values |
| `display_includes_stage_and_epoch` | L1 | `Display` output contains stage name and epoch number |

### Test Cases — `lineage.rs` (9 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `empty_chain_has_zero_depth` | L1 | `LineageChain::new().depth() == 0` |
| `push_increments_depth` | L2 | Two pushes → depth = 2 |
| `final_b11_is_last_hop_out` | L2 | Final B11 is the `b11_out` of the last hop |
| `clean_pass_when_quality_never_drops` | **L3** | `is_clean_pass()` only true when all hops hold or improve quality |
| `not_clean_pass_when_quality_drops` | **L3** | A single quality drop makes `is_clean_pass()` false |
| `origin_is_first_hop_source` | L2 | `chain.origin()` returns the SourceId of the first hop |
| `fnv1a_is_deterministic` | **L3** | Same bytes always produce the same hash (sovereignty invariant) |
| `hash_record_changes_with_content` | L1 | Different record values produce different hashes |
| `report_contains_stage_names` | L2 | `chain.report()` output contains all stage identifiers |

### Test Cases — `pipeline.rs` (7 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `pipeline_has_correct_source` | L1 | Builder preserves `SourceId` correctly |
| `pipeline_has_four_stages` | L1 | Four `stage()` calls produce four stages in order |
| `pipeline_has_two_targets` | L1 | Two `target()` calls produce two `TargetId`s |
| `validate_stage_carries_min_b11` | L1 | `Stage::Validate { min_b11: 140 }` round-trips through builder |
| `pipeline_is_enabled_by_default` | **L3** | `enabled` is always `true` after build — never starts disabled |
| `stage_names_are_correct` | L1 | `stage_name()` returns the correct string for each variant |
| `builder_bumps_version` | L1 | `.version(3)` sets `pipeline.version = 3` |

### Test Cases — `orchestrator.rs` (7 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `registration_counts_are_correct` | L1 | After 1 source, 1 target, 1 pipeline: counts match |
| `clean_records_are_delivered` | **L4** | Full run: extract → cleanse → validate → deliver → receipt |
| `empty_value_rejected_by_quality_gate` | **L4** | All-empty record → QualityRejection exception, 0 records out |
| `lineage_chain_depth_matches_stage_count_plus_extraction` | **L4** | 2 stages → chain depth = 3 (extraction + cleanse + validate) |
| `success_rate_is_one_for_clean_batch` | **L4** | 2 clean records → `success_rate() == 1.0` |
| `unknown_pipeline_returns_none` | **L3** | Unregistered pipeline → `None` (never panics) |
| `multiple_targets_produce_multiple_receipts` | **L4** | 2 targets → 2 delivery receipts |

### Test Cases — `adapters.rs` (9 tests)

| Test | Depth | What It Validates |
|---|---|---|
| `erp_connector_id_and_display` | L1 | `SourceId("erp.sovereign")` and display name correct |
| `crm_schema_has_required_email` | L1 | CRM contract includes `email` as a required field |
| `excel_extract_returns_empty_batch` | L1 | Stub returns empty batch (no I/O stub panic) |
| `dw_target_accepts_batch` | L1 | DW stub delivers and returns `records_accepted = 1` |
| `notification_target_id_correct` | L1 | `TargetId("notify.sovereign")` round-trips |
| `all_source_adapters_have_unique_ids` | **L3** | All 8 `SourceId`s are distinct — no collision possible |
| `all_target_adapters_have_unique_ids` | **L3** | All 7 `TargetId`s are distinct — no collision possible |
| `crm_schema_has_required_email` | L1 | Required hash `0x2003` (email) present in required set |

### Doc-Test — `lib.rs` (1 test)

| Test | What It Validates |
|---|---|
| Full pipeline declaration and run | Complete API surface: register source, register target, declare pipeline with 4 stages, run, inspect result |

---

## Sovereign Constraints

| Rule | Location | Enforcement |
|---|---|---|
| **No third-party runtime deps** | `Cargo.toml` | Only internal BahyWay crates; no tokio, serde, reqwest, rayon |
| **No unsafe code** | Implicit (no `unsafe` blocks written) | Sovereign data fabric must be memory-safe by construction |
| **SchemaContract at every boundary** | `orchestrator.rs::run_pipeline` | Source contract validated before stage 1; target contract declared before delivery |
| **LineageChain always built** | `orchestrator.rs::apply_stage` | Every stage run appends a hop — no silent passthrough |
| **FabricException always typed** | `exception.rs` | `ExceptionKind` enum with 7 variants — no raw `String` errors exported |
| **QUALITY_DIVISOR = 240.0** | Inherited from `hepta-score` | `Stage::Validate` B11 thresholds use the same divisor — TRIBE ≥ 140, ACTIVE ≥ 100 |
| **`enabled` defaults true** | `pipeline.rs::PipelineBuilder::build` | A pipeline cannot be born disabled — disabling requires explicit mutation after build |

---

## Integration Patterns

### Pattern 1: Incremental extraction with cursor resumption

```rust
let mut cursor = ExtractionCursor { last_epoch: 0, offset: vec![] };
loop {
    let result = fabric.run_pipeline(&pipeline_id, &cursor).unwrap();
    // advance cursor to last processed epoch
    cursor.last_epoch = result.lineage
        .last()
        .and_then(|c| c.hops.last())
        .map(|h| h.epoch)
        .unwrap_or(cursor.last_epoch);
    if result.records_in == 0 { break; }  // source exhausted
}
```

### Pattern 2: Dead-letter queue inspection

```rust
for ex in &result.exceptions {
    match ex.kind {
        ExceptionKind::QualityRejection => {
            // Route to human review queue
            dlq_sender.send(ex.payload.clone());
        }
        ExceptionKind::DeliveryFailure => {
            // Alert operations team
            alert_engine.fire(ex.message.clone());
        }
        _ => { /* log and continue */ }
    }
}
```

### Pattern 3: Lineage query — "where does this value come from?"

```rust
for chain in &result.lineage {
    if let Some(origin) = chain.origin() {
        println!("Origin: {}", origin.0);
    }
    println!("Depth: {} stages", chain.depth());
    println!("Final B11: {:?}", chain.final_b11());
    println!("Clean pass: {}", chain.is_clean_pass());
    println!("{}", chain.report());
}
```

---

## Future Extensions (Roadmap)

| Extension | Layer Integration | Priority |
|---|---|---|
| Persist `LineageChain` to Journal as `EventKaki` sequence | `enkidb-journal` | High |
| HeptaScript `.hepta` query over lineage chains | `heptascript` | High |
| Eridu scheduler integration — pipeline runs on cron schedule | `eridu-scheduler` | Medium |
| AAOL `.akk` policy for per-record exception routing | `aaol` | Medium |
| Dubsar visualizer — live pipeline flow graph | `dubsar-visualizer` | Medium |
| Dilithium post-quantum signing of `DeliveryReceipt` | `kupru` | Low |

---

*"Data does not flow in pipes. Data orbits through sovereign stages, carrying its history."*

*𒁾 DUB.SAR — BahyWay.Ecosystem v4.0 | 2026-06-04*
