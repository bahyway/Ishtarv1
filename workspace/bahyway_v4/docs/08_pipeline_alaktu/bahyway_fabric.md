# bahyway-fabric — Enterprise Data Fabric

> **DubSar Help** | `Pipeline > Fabric` | Layer 8

## Purpose

`bahyway-fabric` is the sovereign Enterprise Data Fabric sitting at the top of
the Layer 8 pipeline stack. It is the structural answer to enterprise spaghetti
processing — the condition where eight sources route through ten tangled
transformation nodes to seven targets with no traceability, no schema
enforcement, and no structured exception handling.

The Fabric does not patch the spaghetti. It replaces it with four sovereign
primitives:

| Primitive | Replaces |
|---|---|
| `SchemaContract` | Implicit, unenforced field assumptions |
| `PipelineDeclaration` | Ad-hoc, hard-coded routing logic |
| `LineageChain` | "Where does this data even come from?" (no answer) |
| `FabricException` | Silent failures that surface three days later |

---

## Position in the ALAKTU Pipeline

```
External World
    │
    ▼
┌─────────────────────────────────────────────────────┐
│                  bahyway-fabric                      │
│                                                     │
│  SourceConnector.extract()                          │
│      │  SchemaContract validated here               │
│      ▼                                              │
│  Stage: Cleanse  (→ vgca-engine)                    │
│  Stage: Validate (→ hepta-score H(P))               │
│  Stage: Enrich   (→ shulman-engine)                 │
│  Stage: Deduplicate (→ idu-prober)                  │
│  Stage: Aggregate (→ enkidb-dw)                     │
│      │  LineageHop appended after each stage        │
│      ▼                                              │
│  TargetConnector.deliver()                          │
│      │  SchemaContract validated here               │
│      ▼                                              │
└─────────────────────────────────────────────────────┘
    │
    ▼
adad-gate → enkidb-journal → permanent-storage
```

---

## Relationship to Other L8 Stations

| Station | Relationship |
|---|---|
| `adad-gate` | Fabric calls `AdadGate.ingest()` to mint KAKI for each arriving record |
| `vgca-validation` | `Stage::Cleanse` delegates geometric normalisation to VGCA engines |
| `data-cleansing-station` | Provides the cleansing ruleset referenced by `Stage::Cleanse` |
| `data-steward-station` | Consumes `OrchestratorResult.lineage` for stewardship review |
| `musaru-security` | Applied after Fabric delivery — security gate on delivered batches |
| `permanent-storage` | Final target for `DataWarehouseTarget` and `FileExportTarget` |

---

## Sources Covered

All eight common enterprise source types are registered as built-in adapters:

| System | `SourceId` | Minimum Required Fields |
|---|---|---|
| ERP | `erp.sovereign` | erp_id, amount, currency |
| CRM | `crm.sovereign` | contact_id, full_name, email |
| HR System | `hr.sovereign` | employee_id, name, department |
| Legacy System | `legacy.sovereign` | rec_id, payload |
| External Partner | `partner.sovereign` | txn_id, amount, partner_ref |
| Excel Files | `excel.sovereign` | row_index, raw_columns |
| Email / Inbox | `email.sovereign` | message_id, subject, sender |
| Third-Party API | `api.sovereign` | event_id, event_type, payload |

## Targets Covered

| System | `TargetId` |
|---|---|
| Data Warehouse | `dw.central` |
| Reporting Tools | `reporting.tools` |
| Dashboards | `dashboard.sovereign` |
| Other Applications | `apps.sovereign` |
| File Exports | `file.export` |
| External Portals | `portal.sovereign` |
| Notifications | `notify.sovereign` |

---

## Exception Flow

```
Stage raises FabricException
    │
    ├─ kind = MissingRequiredField  →  dead-letter record, continue batch
    ├─ kind = QualityRejection      →  dead-letter queue or reject record
    ├─ kind = DuplicateIdentity     →  merge resolution or reject
    ├─ kind = DeliveryFailure       →  retry with backoff, then alert
    ├─ kind = ExtractionError       →  halt extraction, alert operations
    ├─ kind = TransformError        →  halt pipeline, alert architect
    └─ kind = InternalFault         →  sovereign audit event, immediate alert
```

All exceptions carry: `kind`, `source_id`, `stage`, `message`, `payload`, `epoch`.
None are silent. All are logged to the Journal.

---

## See Also

- `crates/bahyway-fabric/MANUAL.md` — Full W5H2 manual
- `08_pipeline_alaktu/submission.md` — Upstream submission stage
- `04_gates/adad_gate.md` — KAKI minting at the ingestion gate
- `14_decisions_adr/adr_005_enterprise_data_fabric.md` — Design decision
- `15_howto/connect_a_new_source.md` — Adding a new source connector
- `15_howto/declare_a_pipeline.md` — Writing a pipeline declaration
- `17_troubleshooting/fabric_exception_diagnosis.md` — Diagnosing exceptions
