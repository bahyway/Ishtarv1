# 𒀭𒂗𒆠 bahyway-fabric — Sovereign Enterprise Data Fabric
**Version:** 4.0.2 | **Layer:** 8 — Enterprise Fabric | **Status:** Production

---

## What It Solves

Every enterprise suffers the same wound: data flowing from eight sources through
ten tangled processing nodes to seven targets — a spaghetti of untraceable,
schema-less, exception-silencing pipes.  The result:

- *"Where does this data even come from?"* — no one knows
- Silent failures that surface as wrong dashboards three days later
- Every new source requires changing every existing pipeline
- Validation exists only as comments, not enforcement

**`bahyway-fabric` solves all five spaghetti problems at once** — as a
sovereign, transparent Enterprise Data Fabric built on BahyWay's KAKI identity,
H(P) quality, and EriduOS scheduler.

| Spaghetti Problem | Sovereign Answer |
|---|---|
| No single source of truth | KAKI sovereign identity — every particle born once, referenced everywhere |
| Hard to trace & maintain | `LineageChain` — every hop recorded: source → stage → target, immutable |
| High cost & complexity | `PipelineDeclaration` — declare WHAT, EriduOS executes HOW |
| Error prone & inconsistent | `SchemaContract` enforced at every connector boundary before data moves |
| Slow changes & time to market | `SourceConnector` / `TargetConnector` traits — add a system, touch nothing else |

---

## Architecture Position

```
SOURCES                             TARGETS
────────                            ───────
ERP        ─┐                  ┌─▶ Data Warehouse
CRM        ─┤                  ├─▶ Reporting Tools
HR System  ─┤                  ├─▶ Dashboards
Legacy     ─┤   BAHYWAY-FABRIC ├─▶ Other Applications
Partners   ─┤   (this crate)   ├─▶ File Exports
Excel      ─┤                  ├─▶ External Portals
Email      ─┤                  └─▶ Notifications
Third-Party─┘

        ↕ SchemaContract at every boundary
        ↕ LineageChain per record
        ↕ FabricException — never silent
        ↕ PipelineDeclaration — declarative, versioned
```

---

## Quick Start

```rust
use bahyway_fabric::prelude::*;

// 1. Build the orchestrator
let mut fabric = FabricOrchestrator::new();

// 2. Register sources — adding one never touches existing pipelines
fabric.register_source(Box::new(ErpConnector));
fabric.register_source(Box::new(CrmConnector));

// 3. Register targets
fabric.register_target(Box::new(DataWarehouseTarget));
fabric.register_target(Box::new(NotificationTarget));

// 4. Declare a pipeline — WHAT, not HOW
let pipeline = PipelineDeclaration::builder("erp.invoices → dw", "erp.sovereign")
    .description("Ingest ERP invoices, cleanse, validate, deduplicate, load to DW")
    .stage(Stage::Cleanse)
    .stage(Stage::Validate { min_b11: 140 })   // TRIBE lane minimum
    .stage(Stage::Deduplicate)
    .stage(Stage::Enrich { ruleset: "erp.enrichment.v1" })
    .target("dw.central")
    .target("notify.sovereign")
    .build();

fabric.register_pipeline(pipeline);

// 5. Run — full lineage, structured exceptions, delivery receipts
let result = fabric.run_pipeline(
    &PipelineId("erp.invoices → dw"),
    &ExtractionCursor::default(),
).unwrap();

// 6. Inspect lineage — answer "where does this data come from?"
for chain in &result.lineage {
    println!("{}", chain.report());
}

// 7. Handle exceptions — always structured, never silent
for ex in &result.exceptions {
    eprintln!("[{:?}] {}", ex.kind, ex.message);
}
```

---

## Module Map

| Module | Purpose |
|---|---|
| `contract` | `SchemaContract` — typed field declarations enforced at boundaries |
| `connector` | `SourceConnector` + `TargetConnector` traits, `DataBatch`, `DeliveryReceipt` |
| `exception` | `FabricException` — 7 kinds, structured, staged, epoched |
| `lineage` | `LineageChain` / `LineageHop` — immutable hop-by-hop audit trail |
| `pipeline` | `PipelineDeclaration` + `PipelineBuilder` — declarative pipeline DSL |
| `orchestrator` | `FabricOrchestrator` — extract → validate → stage → deliver |
| `adapters` | 8 built-in source adapters + 7 built-in target adapters |

---

## Sovereign Constraints

```
✓ #![forbid(unsafe_code)]     — zero unsafe Rust
✓ No third-party runtime deps  — only internal BahyWay crates
✓ SchemaContract always        — no untyped boundary ever permitted
✓ LineageChain always          — no hop without a record
✓ FabricException always typed — ExceptionKind enum, never String-only error
✓ QUALITY_DIVISOR = 240.0      — ADR-001, never 255
✓ Validate min_b11 default     — TRIBE lane (≥ 140) for enterprise data
```

---

## Dependency Map

```
bahyway-fabric
    ├── bahyway-core    (TribeId, BahywayError, ParticleState)
    ├── bahyway-crc     (CRC-16 for KAKI checksum)
    ├── enkidb-kaki     (Kaki, KakiMinter, IdentityKaki, EventKaki)
    ├── enkidb-journal  (JournalEntry, EavTriple, EventCause)
    ├── adad-gate       (AdadGate — sole KAKI minter)
    ├── hepta-score     (H(P) equation, B11, QualityLane)
    └── story-engine    (StoryEngine — CQRS projection for lineage replay)
```

---

## Test Coverage: 38 tests · 0 failures · 0 warnings

| Module | Tests | Coverage |
|---|---|---|
| `contract` | 4 | SchemaContract validation, required/optional distinction |
| `connector` | — | Trait definitions (tested via orchestrator) |
| `exception` | 3 | All exception kinds, Display format |
| `lineage` | 9 | Chain operations, FNV-1a hash, report output |
| `pipeline` | 7 | Builder pattern, stage configuration, defaults |
| `orchestrator` | 7 | Full pipeline runs, quality rejection, multi-target delivery, lineage depth |
| `adapters` | 9 | All 8 sources + 7 targets — unique IDs, schema presence, delivery |
| `lib.rs` | 1 | Doc-test: full pipeline declaration and run |
| **Total** | **38+1** | **Zero failures** |

---

*𒁾 DUB.SAR — BahyWay.Ecosystem v4.0 | Sovereign Enterprise Data Fabric*
